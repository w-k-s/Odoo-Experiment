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
            console.error(error)
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
