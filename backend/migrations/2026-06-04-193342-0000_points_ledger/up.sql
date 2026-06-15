-- Your SQL goes here
ALTER TABLE loyalty_programs
ADD COLUMN points_per_currency_minor_unit INTEGER NOT NULL DEFAULT 1;
CREATE TABLE points_transactions (
    id TEXT PRIMARY KEY,
    member_id TEXT NOT NULL REFERENCES loyalty_members(id) ON DELETE CASCADE,
    source_order TEXT NOT NULL,
    -- pos_order id (as text)
    delta INTEGER NOT NULL,
    -- +earn / -redeem
    amount_total NUMERIC(12, 2) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (source_order) -- idempotency key
);
CREATE INDEX idx_points_tx_member ON points_transactions (member_id);