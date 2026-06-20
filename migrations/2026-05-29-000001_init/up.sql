-- Loyalty programs: the rule set (e.g. "UAE Bakery Rewards").
CREATE TABLE loyalty_programs (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
-- Members enrolled in a program.
CREATE TABLE loyalty_members (
    id TEXT PRIMARY KEY,
    program_id TEXT NOT NULL REFERENCES loyalty_programs(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    email TEXT,
    -- Opaque id of the linked contact in the CRM (Odoo res.partner id, as text).
    external_contact_id TEXT,
    points INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_loyalty_members_program_id ON loyalty_members (program_id);
CREATE UNIQUE INDEX uq_loyalty_members_email ON loyalty_members (email);
-- A session is the short-lived code a member presents at the till.
CREATE TABLE loyalty_sessions (
    id TEXT PRIMARY KEY,
    member_id TEXT NOT NULL REFERENCES loyalty_members(id) ON DELETE CASCADE,
    status TEXT NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at TIMESTAMPTZ
);
CREATE INDEX idx_loyalty_sessions_member_id ON loyalty_sessions (member_id);