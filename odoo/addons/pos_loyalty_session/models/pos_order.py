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

        # TODO: Request client credentials
        base_url = (
            self.env["ir.config_parameter"]
            .sudo()
            .get_param("loyalty.base_url", DEFAULT_LOYALTY_BASE_URL)
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
            partner = partner.create(
                {
                    "name": name,
                    "email": email or False,
                    "phone": member.get("phone") or False,
                }
            )

        return {
            "session_code": data.get("session_id", code),
            "partner_id": partner.id,
        }
