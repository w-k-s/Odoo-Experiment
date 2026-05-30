{
    "name": "POS Loyalty Session",
    "version": "18.0.1.0.0",
    "summary": "Capture a loyalty session code on each POS order",
    "category": "Sales/Point of Sale",
    "license": "LGPL-3",
    "depends": ["point_of_sale"],
    "assets": {
        "point_of_sale._assets_pos": [
            "pos_loyalty_session/static/src/app/**/*",
        ],
    },
    "installable": True,
}
