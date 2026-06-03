# Diesel migrations

Each subdirectory here is one migration: a paired `up.sql` (apply) and
`down.sql` (revert). They run **in lexical order**, which is why every dir is
prefixed with a timestamp (`YYYY-MM-DD-HHMMSS_name`).

> In this project migrations are **embedded into the binary** and run
> automatically on startup (`embed_migrations!` → `MIGRATIONS` in `src/db.rs`).
> You do **not** run `diesel migration run` against production — you generate
> the migration here, regenerate `schema.rs`, and the app applies it on boot.

## Editing a migration → rebuild the Docker image

Because migrations are embedded **at compile time**, editing any `*.sql` has no
effect on a running `loyalty-backend` container until its image is rebuilt — the
old binary still carries the old SQL. After changing a migration, rebuild and
recreate the DB from scratch:

```bash
docker compose build loyalty-backend          # re-embed the migrations
docker compose rm -sf loyalty-backend loyalty-postgres
rm -rf backend/pg-data                         # wipe the DB volume
docker compose up -d loyalty-postgres          # wait until healthy
docker compose up -d loyalty-backend           # reruns migrations on boot
```

## One-time setup

```bash
cargo install diesel_cli --no-default-features --features postgres
# DATABASE_URL points diesel at a *dev* database (e.g. the compose Postgres on :5433)
export DATABASE_URL=postgres://USER:PASS@localhost:5433/DBNAME
```

`diesel.toml` is already configured: migrations live in `migrations/`, and the schema is printed to `src/schema.rs`.

## Adding a migration

1. **Generate the empty migration pair:**

   ```bash
   diesel migration generate <name>      # e.g. points_ledger
   ```

   This creates `migrations/<timestamp>_<name>/{up,down}.sql`.

2. **Write `up.sql`** — the forward change (e.g. `CREATE TABLE …`,
   `ALTER TABLE … ADD COLUMN …`).

3. **Write `down.sql`** — the exact inverse that returns the schema to its
   prior state (e.g. `DROP TABLE …`, `ALTER TABLE … DROP COLUMN …`). Keep it
   correct: it's how you revert and how `redo` is tested.

4. **Apply against your dev DB and regenerate the schema:**

   ```bash
   diesel migration run        # applies pending up.sql; reprints src/schema.rs
   ```

   Commit the resulting `src/schema.rs` changes alongside the migration.

5. **Verify the down works** (apply → revert → re-apply in one shot):

   ```bash
   diesel migration redo
   ```

6. **Update the Rust models** in `src/loyalty_engine/models.rs` to match any
   new/changed columns (the `table!` macros in `schema.rs` are regenerated for
   you in step 4; the structs are not).

## Handy commands

```bash
diesel migration list       # show applied / pending
diesel migration run        # apply all pending up.sql
diesel migration revert     # revert the most recent migration (down.sql)
diesel migration redo       # revert + re-apply the most recent (tests down.sql)
diesel print-schema         # reprint schema.rs without running anything
```

## Conventions in this repo

- **Timestamp-prefix every dir** so ordering is deterministic.
- **Never edit a migration that has already shipped/been applied elsewhere** —
  add a new migration instead. Editing applied SQL desyncs environments.
- **Keep `down.sql` a true inverse** of `up.sql`.
- **Commit `schema.rs` with the migration** so the embedded set and the
  generated schema never drift.
