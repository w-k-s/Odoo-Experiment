import logging
import os

import requests

from odoo import _, api, fields, models
from odoo.exceptions import UserError

_logger = logging.getLogger(__name__)

# Default base URL of the loyalty backend on the Compose network.
# Override at runtime with the `loyalty.base_url` system parameter
# (Settings ▸ Technical ▸ System Parameters) without touching code.
DEFAULT_LOYALTY_BASE_URL = "http://loyalty-backend:8000"
DEFAULT_LOYALTY_AUTH_URL = "https://wks-bakery.eu.auth0.com/oauth/token"


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

        base_url = (
            self.env["ir.config_parameter"]
            .sudo()
            .get_param("loyalty.base_url", DEFAULT_LOYALTY_BASE_URL)
        )

        auth_url = (
            self.env["ir.config_parameter"]
            .sudo()
            .get_param("loyalty.auth_url", DEFAULT_LOYALTY_AUTH_URL)
        )

        url = f"{base_url.rstrip('/')}/loyalty/sessions/{code}"

        client_id = os.environ.get("LOYALTY_AUTH_CLIENT_ID")
        client_secret = os.environ.get("LOYALTY_AUTH_CLIENT_SECRET")
        audience = os.environ.get("LOYALTY_AUTH_AUDIENCE")
        missing = [
            name
            for name, value in (
                ("LOYALTY_AUTH_CLIENT_ID", client_id),
                ("LOYALTY_AUTH_CLIENT_SECRET", client_secret),
                ("LOYALTY_AUTH_AUDIENCE", audience),
            )
            if not value
        ]
        if missing:
            _logger.error(
                "Loyalty auth misconfigured; missing env vars: %s", ", ".join(missing)
            )
            raise UserError(
                _(
                    "Loyalty service is not configured. Please contact your administrator."
                )
            )

        try:
            token_response = requests.post(
                url=auth_url,
                json={
                    "client_id": client_id,
                    "client_secret": client_secret,
                    "audience": audience,
                    "grant_type": "client_credentials",
                },
                timeout=5,
            )

            token_response.raise_for_status()
            token_json = token_response.json()

            access_token = token_json.get("access_token")

            resp = requests.get(
                url, headers={"Authorization": f"Bearer {access_token}"}, timeout=5
            )
            resp.raise_for_status()
            data = resp.json()
        except requests.exceptions.Timeout:
            raise UserError(_("Loyalty service timed out. Please try again."))
        except requests.exceptions.RequestException as exc:
            _logger.warning("Loyalty lookup failed for %s: %s", code, exc)
            raise UserError(_("Could not resolve loyalty session '%s'.") % code)

        member = data.get("member") or {}
        member_id = (member.get("id") or "").strip()

        # Find or create the customer. Email is our match key; adjust if your
        # loyalty backend returns a stronger external identifier.
        partner = self.env["res.partner"]
        if member_id:
            partner = partner.search([("ref", "=", member_id)], limit=1)

        return {
            "session_code": data.get("session_id", code),
            "partner_id": partner.id,
        }
