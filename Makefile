# Convenience targets for the Odoo loyalty stack.
# Override the module on the CLI, e.g. `make odoo-upgrade MODULE=point_of_sale`.
MODULE ?= pos_loyalty_session

.PHONY: odoo-upgrade odoo-assets odoo-restart

## Reload an addon after Python changes: upgrade the module, then restart Odoo.
odoo-upgrade:
	docker compose run --rm odoo odoo -c /etc/odoo/odoo.conf -d odoo \
		-u $(MODULE) --stop-after-init
	docker compose restart odoo

## Reload after JS/XML-only changes: drop cached asset bundles, then restart.
odoo-assets:
	docker compose exec -T odoo-postgres psql -U odoo -d odoo \
		-c "delete from ir_attachment where url like '/web/assets/%';"
	docker compose restart odoo

## Just restart the live Odoo server.
odoo-restart:
	docker compose restart odoo 

backend: backend/crates/**
	cd backend; cargo build; cd ..;

backend-lint: backend/crates/**
	cd backend; cargo clippy; cd ..;

## Restart the backend container without rebuilding the image (picks up env/config changes only).
backend-restart:
	docker compose up -d --force-recreate loyalty-backend

## Rebuild the Docker image from source and restart the container (picks up code changes).
backend-rebuild:
	docker compose up -d --build loyalty-backend

backend-generate-schema: 
	diesel migration run --database-url=postgres://loyalty:loyalty_password@localhost:5433/loyalty