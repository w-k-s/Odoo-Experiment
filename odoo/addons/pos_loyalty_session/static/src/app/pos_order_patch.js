import { patch } from "@web/core/utils/patch";
import { PosOrder } from "@point_of_sale/app/models/pos_order";

patch(PosOrder.prototype, {
    // https://www.odoo.com/forum/help-1/how-to-include-a-custom-posorder-field-in-orderreceipt-frontend-template-odoo-18-294122
    export_for_printing(baseUrl, headerData) {
        const result = super.export_for_printing(baseUrl, headerData);
        result.loyalty_session_code = this.loyalty_session_code || "";
        return result;
    },
});
