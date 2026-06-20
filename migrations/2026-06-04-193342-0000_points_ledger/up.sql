-- Your SQL goes here
ALTER TABLE loyalty_programs
ADD COLUMN points_per_currency_minor_unit INTEGER NOT NULL DEFAULT 1;
CREATE TABLE points_transactions (
    id TEXT PRIMARY KEY,
    member_id TEXT NOT NULL,
    program_id TEXT NOT NULL REFERENCES loyalty_programs(id) ON DELETE CASCADE,
    source_system TEXT NOT NULL,
    source_order TEXT NOT NULL,
    -- pos_order id (as text)
    delta INTEGER NOT NULL,
    -- +earn / -redeem
    amount_total NUMERIC(12, 2) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT fk_points_transaction_member_id FOREIGN KEY(member_id) REFERENCES loyalty_members(id) ON DELETE CASCADE,
    CONSTRAINT fk_points_transaction_program_id FOREIGN KEY(program_id) REFERENCES loyalty_programs(id) ON DELETE CASCADE,
    CONSTRAINT uq_points_transactions_source_order UNIQUE(source_system, source_order)
);
CREATE INDEX idx_points_tx_member ON points_transactions (member_id);