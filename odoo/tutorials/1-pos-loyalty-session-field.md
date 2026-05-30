# Phase 5 — POS ↔ Loyalty Integration

**Goal:** at the till, the cashier enters the **loyalty session code** the member
created in the PWA (Phase 4). The POS then:

1. stores the code in a custom `loyalty_session_code` field on `pos.order`,
2. calls the **loyalty backend** (Phase 3) to resolve the session → member details,
3. **links the customer** to the order using Odoo's built-in POS customer feature
   (creating the `res.partner` if they don't exist yet),

and all of this persists to the `pos_order` table in Postgres (which Phase 6's
Debezium connector will later capture).

By the end you'll have a small, committable Odoo module:
`odoo/addons/pos_loyalty_session/`.

### Prerequisites

- The **loyalty backend** (Phase 3) is running and reachable from the Odoo
  container, exposing:
  - `GET /loyalty/sessions/{id}` → returns the session and its member, e.g.
    ```json
    { "session_id": "LOY-7F3A9", "member": { "name": "Sara K", "email": "sara@example.com", "phone": "+9715..." } }
    ```
- Assumed reachable at `http://loyalty-backend:8000` on the Compose network.
  We'll make this configurable, so adjust to your service name/port.

> All file paths below are **host** paths. The container sees them at
> `/mnt/addons/...` because `docker-compose.yaml` mounts `./odoo/addons:/mnt/addons`,
> and `odoo/etc/odoo.conf` has `addons_path = /mnt/addons,...`.

---

## How POS data flows (read this first)

The POS is an OWL/JavaScript app in the browser, backed by Python models. For this
phase you touch **four** layers:

1. **Python model** — declare the column on `pos.order` so it exists in Postgres.
2. **Data loading** — tell the POS which fields to ship to the browser
   (`_load_pos_data_fields`). A field the frontend doesn't know about won't be
   read or written back.
3. **Python method** — a server-side method that calls the loyalty backend and
   finds/creates the `res.partner`. Doing the HTTP call **server-side** (not from
   the browser) avoids CORS and keeps the loyalty service URL/secrets off the
   client.
4. **Frontend (OWL/JS)** — a button + popup to capture the code, which calls the
   Python method, sets the field, and links the returned customer to the order.

Because the field is registered in step 2, the POS's client-side ORM
(`related_models`) automatically **serialises it back** to the server when the
order is saved — you don't write any custom RPC for persistence.

---

## Step 1 — Module skeleton

Create this directory tree under `odoo/addons/`:

```
pos_loyalty_session/
├── __init__.py
├── __manifest__.py
├── models/
│   ├── __init__.py
│   └── pos_order.py
└── static/
    └── src/
        └── app/
            ├── loyalty_session_button.js
            ├── loyalty_session_button.xml
            ├── control_buttons.xml
            └── pos_order_patch.js
```

---

## Step 2 — The manifest

`odoo/addons/pos_loyalty_session/__manifest__.py`

```python
{
    "name": "POS Loyalty Session",
    "version": "18.0.1.0.0",
    "summary": "Capture a loyalty session code on each POS order",
    "category": "Sales/Point of Sale",
    "license": "LGPL-3",
    "depends": ["point_of_sale"],
    "assets": {
        # This bundle is what the POS client loads. Globbing the app folder
        # picks up every .js and .xml we add under static/src/app.
        "point_of_sale._assets_pos": [
            "pos_loyalty_session/static/src/app/**/*",
        ],
    },
    "installable": True,
}
```

Key point: POS frontend assets go in the **`point_of_sale._assets_pos`** bundle,
not the usual `web.assets_backend`. If you put them in the wrong bundle the button
silently never appears.

---

## Step 3 — Python package init files

`odoo/addons/pos_loyalty_session/__init__.py`

```python
from . import models
```

`odoo/addons/pos_loyalty_session/models/__init__.py`

```python
from . import pos_order
```

---

## Step 4 — The Python model

`odoo/addons/pos_loyalty_session/models/pos_order.py`

```python
import logging

import requests

from odoo import _, api, fields, models
from odoo.exceptions import UserError

_logger = logging.getLogger(__name__)

# Default base URL of the loyalty backend on the Compose network.
# Override at runtime with the `loyalty.base_url` system parameter
# (Settings ▸ Technical ▸ System Parameters) without touching code.
DEFAULT_LOYALTY_BASE_URL = "http://loyalty-backend:8000"


class PosOrder(models.Model):
    _inherit = "pos.order"

    loyalty_session_code = fields.Char(
        string="Loyalty Session Code",
        help="Loyalty session code presented by the member at checkout. "
             "Resolved against the loyalty backend to link the customer, and "
             "captured downstream via CDC (Debezium).",
    )

    # Don't do this in odoo 18 and above: https://www.odoo.com/forum/help-1/how-to-include-a-custom-posorder-field-in-orderreceipt-frontend-template-odoo-18-294122
    @api.model
    def _load_pos_data_fields(self, config_id):
        # Append our field to the list the POS frontend loads, so it is read
        # into the browser and written back when the order is saved.
        result = super()._load_pos_data_fields(config_id)
        if "loyalty_session_code" not in result:
            result.append("loyalty_session_code")
        return result

    @api.model
    def lookup_loyalty_session(self, code):
        """Resolve a loyalty session code to a customer.

        Called from the POS frontend. Performs the HTTP call server-side
        (no CORS, URL stays off the client), finds or creates the matching
        res.partner, and returns a small dict the POS can act on.
        """
        code = (code or "").strip()
        if not code:
            return {"session_code": "", "partner_id": False}

        base_url = self.env["ir.config_parameter"].sudo().get_param(
            "loyalty.base_url", DEFAULT_LOYALTY_BASE_URL
        )
        url = f"{base_url.rstrip('/')}/loyalty/sessions/{code}"

        try:
            resp = requests.get(url, timeout=5)
            resp.raise_for_status()
            data = resp.json()
        except requests.exceptions.Timeout:
            raise UserError(_("Loyalty service timed out. Please try again."))
        except requests.exceptions.RequestException as exc:
            _logger.warning("Loyalty lookup failed for %s: %s", code, exc)
            raise UserError(_("Could not resolve loyalty session '%s'.") % code)

        member = data.get("member") or {}
        email = (member.get("email") or "").strip()
        name = member.get("name") or email or _("Loyalty Member")

        # Find or create the customer. Email is our match key; adjust if your
        # loyalty backend returns a stronger external identifier.
        partner = self.env["res.partner"]
        if email:
            partner = partner.search([("email", "=", email)], limit=1)
        if not partner:
            partner = partner.create({
                "name": name,
                "email": email or False,
                "phone": member.get("phone") or False,
            })

        return {
            "session_code": data.get("session_id", code),
            "partner_id": partner.id,
        }
```

What each piece does:
- The `fields.Char` creates the actual `loyalty_session_code` column on `pos_order`.
- `_load_pos_data_fields` makes the POS client aware of it (read in + written back).
- `lookup_loyalty_session` is the server-side bridge to the loyalty backend; it
  returns the canonical session code plus the `res.partner` id to link.

> **Why find-or-create on email?** Phase 5 deliberately keeps it simple. If your
> loyalty backend issues a stable member/customer ID, prefer matching on a custom
> `ref`/external-id field instead of email to avoid duplicates.

---

## Step 5 — Initialise the field on the frontend order

`odoo/addons/pos_loyalty_session/static/src/app/pos_order_patch.js`

```javascript
import { patch } from "@web/core/utils/patch";
import { PosOrder } from "@point_of_sale/app/models/pos_order";

patch(PosOrder.prototype, {
    setup(vals) {
        super.setup(...arguments);
        // Ensure the property always exists so the UI never shows `undefined`
        // and the value round-trips to the backend.
        this.loyalty_session_code = vals.loyalty_session_code || "";
    },
});
```

`PosOrder` lives at `@point_of_sale/app/models/pos_order` and its `setup(vals)`
receives the values loaded from Python — exactly where we read our field.

---

## Step 6 — The control button component

`odoo/addons/pos_loyalty_session/static/src/app/loyalty_session_button.js`

```javascript
import { Component } from "@odoo/owl";
import { _t } from "@web/core/l10n/translation";
import { useService } from "@web/core/utils/hooks";
import { usePos } from "@point_of_sale/app/store/pos_hook";
import { makeAwaitable } from "@point_of_sale/app/store/make_awaitable_dialog";
import { TextInputPopup } from "@point_of_sale/app/utils/input_popups/text_input_popup";
import { ControlButtons } from "@point_of_sale/app/screens/product_screen/control_buttons/control_buttons";

export class LoyaltySessionButton extends Component {
    static template = "pos_loyalty_session.LoyaltySessionButton";
    static props = {};

    setup() {
        this.pos = usePos();
        this.dialog = useService("dialog");
        this.notification = useService("notification");
    }

    get currentOrder() {
        return this.pos.get_order();
    }

    async onClick() {
        const order = this.currentOrder;
        const code = await makeAwaitable(this.dialog, TextInputPopup, {
            title: _t("Loyalty Session Code"),
            startingValue: order.loyalty_session_code || "",
            placeholder: _t("Scan or enter loyalty code"),
        });
        // `code` is undefined if the cashier cancelled the popup.
        if (code === undefined) {
            return;
        }

        const trimmed = code.trim();
        if (!trimmed) {
            // Cashier cleared the code: drop the link too.
            order.loyalty_session_code = "";
            return;
        }

        // Resolve the session on the server (HTTP call happens in Python).
        let result;
        try {
            result = await this.pos.data.call(
                "pos.order",
                "lookup_loyalty_session",
                [trimmed]
            );
        } catch (error) {
            // UserError from Python surfaces here.
            this.notification.add(
                error?.data?.message || _t("Loyalty lookup failed."),
                { type: "danger" }
            );
            return;
        }

        order.loyalty_session_code = result.session_code;

        // Link the customer using Odoo's built-in POS partner mechanism.
        if (result.partner_id) {
            // Load the partner into the POS client models, then set it.
            const [partner] = await this.pos.data.read("res.partner", [
                result.partner_id,
            ]);
            if (partner) {
                order.set_partner(partner);
                this.notification.add(
                    _t("Customer linked: %s", partner.name),
                    { type: "success" }
                );
            }
        }
    }
}

// Register our component so the inherited template below can render it.
ControlButtons.components = {
    ...ControlButtons.components,
    LoyaltySessionButton,
};
```

These imports and calls are verified against the Odoo 18 image you're running:
- `TextInputPopup` → `@point_of_sale/app/utils/input_popups/text_input_popup`
  (props: `title`, `startingValue`, `placeholder`, `getPayload`, `close`).
- `makeAwaitable` → `@point_of_sale/app/store/make_awaitable_dialog` (turns the
  popup into a promise that resolves with the entered value).
- `usePos` → `@point_of_sale/app/store/pos_hook`.
- `this.pos.data.call(model, method, args)` → calls a Python `@api.model` method.
- `this.pos.data.read("res.partner", [id])` → loads record(s) into the POS client
  models and returns them (needed before `set_partner`).
- `order.set_partner(partner)` → Odoo's built-in way to attach a customer to the
  order (sets `partner_id`, re-applies pricelist/fiscal position).

---

## Step 7 — The button's template

`odoo/addons/pos_loyalty_session/static/src/app/loyalty_session_button.xml`

```xml
<?xml version="1.0" encoding="UTF-8"?>
<templates id="template" xml:space="preserve">
    <t t-name="pos_loyalty_session.LoyaltySessionButton">
        <button class="btn btn-secondary btn-lg py-5" t-on-click="onClick">
            <i class="fa fa-gift me-1" role="img" aria-label="Loyalty code" title="Loyalty code" />
            <t t-if="currentOrder?.loyalty_session_code">
                <t t-esc="currentOrder.loyalty_session_code" />
            </t>
            <t t-else="">Loyalty Code</t>
        </button>
    </t>
</templates>
```

The button shows the current code when set, otherwise the label "Loyalty Code" —
handy for confirming persistence visually.

---

## Step 8 — Inject the button into the POS control panel

`odoo/addons/pos_loyalty_session/static/src/app/control_buttons.xml`

```xml
<?xml version="1.0" encoding="UTF-8"?>
<templates id="template" xml:space="preserve">
    <t t-name="pos_loyalty_session.ControlButtons"
       t-inherit="point_of_sale.ControlButtons"
       t-inherit-mode="extension">
        <xpath expr="//button[hasclass('o_pricelist_button')]" position="before">
            <LoyaltySessionButton />
        </xpath>
    </t>
</templates>
```

This uses OWL template inheritance (`t-inherit` / `xpath`) to splice our button in
just before the Pricelist button, inside the "Actions" panel of the Product Screen.
(On a full-size screen the buttons live behind the **Actions** button; on small
screens they're shown directly.)

---

## Step 9 — Install the module

The module is new, so Odoo needs to discover and install it.

```bash
# From the project root
cd /Users/waqqassheikh/Developer/Odoo-Experiment

# Install the module (updates the app list + installs in one shot)
docker compose run --rm odoo odoo \
  -c /etc/odoo/odoo.conf -d odoo \
  -i pos_loyalty_session --stop-after-init

# Restart the live server so it serves the freshly installed module + assets
docker compose restart odoo
```

> Alternative (UI): enable Developer Mode, go to **Apps → Update Apps List**, search
> "POS Loyalty Session", click **Install**.

If you prefer this to be installed automatically on a clean rebuild, add it to the
`odoo-init` command in `docker-compose.yaml`:
`-i base,point_of_sale,pos_loyalty_session`.

---

## Step 10 — Try it in the POS

1. Make sure the **loyalty backend** is up and a session exists (create one via the
   Phase 4 PWA, or `curl` the backend directly). Note its session code.
2. Open <http://localhost:8069> and go to **Point of Sale**.
3. Open a session, start a new order, add a product.
4. Click **Actions** (or find the buttons directly on a small screen) and click
   **Loyalty Code**.
5. Enter the session code and confirm. You should see:
   - the button now shows the (canonical) code,
   - a "Customer linked: …" notification,
   - the customer appearing on the order (the same slot as the manual
     **Customer** button).
6. **Validate / pay** the order to save it.

> **Testing without the backend yet?** Temporarily point `loyalty.base_url` at a
> stub (e.g. a tiny `python -m http.server` serving a static JSON), or test the
> field-only path first and add the backend once Phase 3 is ready.

---

## Step 11 — Verify it persisted to Postgres

This is the proof that Phase 6's CDC will have something to capture — both the
session code **and** the linked customer:

```bash
docker compose exec -T odoo-postgres \
  psql -U odoo -d odoo \
  -c "SELECT o.id, o.pos_reference, o.loyalty_session_code, o.partner_id, p.name
        FROM pos_order o
        LEFT JOIN res_partner p ON p.id = o.partner_id
       ORDER BY o.id DESC LIMIT 5;"
```

You should see your session code and the resolved customer on the latest order row.

---

## Step 12 — Iterating on JS/XML changes

When you edit **Python**, you must upgrade the module:

```bash
docker compose run --rm odoo odoo -c /etc/odoo/odoo.conf -d odoo \
  -u pos_loyalty_session --stop-after-init
docker compose restart odoo
```

When you edit only **JS/XML assets**, restart is usually enough; if the browser
caches old assets, hard-refresh (Cmd+Shift+R) or append `?debug=assets` to the URL
to bust the bundle.

---

## Step 13 — Commit

All of these are source files safe to commit (no secrets, no build artifacts):

```bash
git add odoo/addons/pos_loyalty_session odoo/tutorials/1-pos-loyalty-session-field.md
git commit -m "Phase 5: POS loyalty session integration (field + customer link)"
```

> If `git status` shows `odoo/pg-data/`, make sure it's git-ignored — it's the
> Postgres data volume, not source.

---

## What you built (recap)

| Layer | File | Responsibility |
|-------|------|----------------|
| DB column | `models/pos_order.py` (`fields.Char`) | Persists the session code |
| Data loading | `models/pos_order.py` (`_load_pos_data_fields`) | Ships field to/from POS client |
| Backend bridge | `models/pos_order.py` (`lookup_loyalty_session`) | Server-side HTTP call + find/create partner |
| Order init | `static/src/app/pos_order_patch.js` | Field always defined on the order |
| UI logic | `static/src/app/loyalty_session_button.js` | Popup → lookup → set code + link customer |
| UI markup | `static/src/app/loyalty_session_button.xml` | The button |
| UI placement | `static/src/app/control_buttons.xml` | Injects button into POS |

**Next (Phase 6):** once the order is posted, Debezium captures the `pos_order`
row (now carrying `loyalty_session_code` and `partner_id`) from the Postgres WAL
and streams it to Kafka for the KFunc transform service.
