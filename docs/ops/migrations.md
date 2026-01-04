
# Database Migrations (Ops)

This document defines the database migration strategy for M0Club (M0-CORE).
It covers schema ownership, migration tooling, workflows for dev/staging/prod, rollback strategies, and operational safety.

M0Club typically uses a relational database (Postgres) for:
- feature store metadata
- market registry cache
- publishing state (commit/reveal tracking)
- API serving (latest outputs, query indices)
- audit logs (optional)

This guide is compatible with either:
- Rust migrations (sqlx, refinery, or diesel)
- Node-based migrations (knex, prisma, drizzle)
- or a dedicated migrations service

The repo should standardize on one method and enforce it in CI.

---

## 1. Goals

- Make schema changes safe, reversible where possible, and auditable.
- Support zero-downtime (or minimal downtime) deployments.
- Ensure migrations are deterministic and consistent across environments.
- Provide clear procedures for running and verifying migrations.
- Prevent schema drift across services.

Non-goals:
- Providing vendor-specific managed DB configuration.
- Guaranteeing that every migration is fully reversible (some are inherently destructive).

---

## 2. Ownership and Boundaries

### 2.1 Schema ownership
Define clear ownership for tables:
- `features_*` tables owned by feature store module
- `registry_*` tables owned by registry cache module
- `publish_*` tables owned by submitter/reconciler
- `api_*` views and indices owned by API gateway

Avoid multiple services writing to the same tables without coordination.

### 2.2 Versioning policy
- migrations are applied in strict order
- each migration has a unique id and timestamp prefix
- migrations are immutable once merged to main
- changes require a new migration, never rewriting an existing one

---

## 3. Migration Tooling

### 3.1 Recommended approach (v1)
Use `sqlx` for Rust-first repos:
- store migrations under `services/migrations/sql/`
- build a small binary `m0-migrations` to apply them

Alternative:
- use a Node migration tool if the repo is Node-heavy

### 3.2 Directory structure
Recommended:
- `services/migrations/`
  - `sql/`
    - `2026-01-01_000001_init.sql`
    - `2026-01-02_000002_add_feature_index.sql`
  - `README.md`
  - `migrate.ts` or `src/main.rs` (runner)

### 3.3 Migration naming
Use UTC timestamp + sequence + short name:
- `YYYYMMDDHHMMSS_<name>.sql`
Example:
- `20260104010101_init_schema.sql`

---

## 4. Migration Workflow

### 4.1 Local development
1) Start Postgres (docker compose).
2) Run migrations.
3) Start services.

Example:
```bash
docker compose -f infrastructure/docker-compose.local.yml up -d postgres
export M0_POSTGRES_URL=postgres://m0:m0@127.0.0.1:5432/m0
pnpm db:migrate
```

If Rust runner:
```bash
cargo run -p m0-migrations -- up --database-url "$M0_POSTGRES_URL"
```

### 4.2 Staging
- migrations run as a Kubernetes Job on each deploy (or manually by operator)
- require approval gates if desired
- monitor DB health during migration

### 4.3 Production
- apply migrations with a controlled process:
  - run in a maintenance window if needed
  - or use online-safe migrations (see section 6)
- always take a backup or ensure PITR is active before applying

---

## 5. Verification and Observability

After applying migrations:
- verify schema version table updated
- run service health checks
- run a short smoke test:
  - API query for latest outputs
  - engine write to feature store
  - submitter read/write publish state

Store migration logs:
- include migration id
- duration
- errors

Add metrics:
- migration run duration
- migration failure count

---

## 6. Zero-Downtime Migration Patterns

To avoid downtime, use additive changes first.

### 6.1 Add columns safely
- add nullable columns with defaults applied later
- backfill in batches
- then enforce NOT NULL if needed

Example pattern:
1) `ALTER TABLE ADD COLUMN new_col ... NULL;`
2) Backfill job updates rows in batches
3) Add constraint / default after backfill

### 6.2 Create indices concurrently (Postgres)
Use:
- `CREATE INDEX CONCURRENTLY ...`
This avoids long locks but cannot run inside a transaction.

Ensure migration tooling supports non-transactional migrations for these cases.

### 6.3 Avoid table rewrites
Be careful with:
- changing column types
- adding defaults to large tables (can rewrite)
Instead:
- create a new column
- backfill
- swap reads/writes
- drop old column later

### 6.4 Split deployments
For breaking schema changes:
- deploy code that supports both old and new schema (compat phase)
- apply migration
- deploy code that fully uses new schema
- remove old paths later

---

## 7. Rollback Strategy

### 7.1 Application rollback vs schema rollback
Prefer rolling back application code over rolling back schema.
Schema rollbacks can be destructive and are often unsafe.

### 7.2 Reversible migrations
If possible, provide down migrations:
- `up.sql` and `down.sql`
However, down migrations may be incomplete for destructive changes.

### 7.3 Emergency recovery
If migration breaks production:
- stop writes if necessary (disable engine publishing)
- restore from backup/PITR
- re-deploy last known good application
- apply a forward-fix migration (preferred) rather than down migration

---

## 8. Schema Version Tracking

Use a version table such as:
- `schema_migrations`

Fields:
- `id` (migration file name)
- `applied_at` timestamp
- `checksum` (optional)
- `duration_ms` (optional)
- `applied_by` (optional)

Migration runner must:
- lock migrations table to prevent concurrent apply
- ensure idempotency (already applied -> no-op)

---

## 9. Concurrency and Locking

Rules:
- only one migration runner instance should apply at a time
- use advisory locks in Postgres for safety
- block other runners until lock released

Example:
- `pg_advisory_lock(hashtext('m0-migrations'))`

---

## 10. Data Backfills

Backfills should be separate from schema migrations when large.

Recommended:
- a dedicated backfill job (K8s Job/CronJob)
- safe batch sizes
- resumable progress table

Backfill principles:
- avoid long locks
- throttle writes
- record progress

---

## 11. CI and Safety Gates

CI should:
- run migrations on a fresh DB
- run service integration tests
- enforce linting of SQL files
- ensure migration ordering and uniqueness

Safety gates:
- block migrations that contain dangerous patterns without approval
- require manual approval for destructive migrations (DROP, TRUNCATE)

---

## 12. Example Runbook

### Apply migrations in staging
```bash
kubectl -n m0-staging apply -f infrastructure/k8s/jobs/m0-migrations.yaml
kubectl -n m0-staging logs job/m0-migrations -f
```

### Apply migrations in production (manual)
1) Confirm backup / PITR
2) Scale down non-critical writers
3) Run migration job
4) Verify health
5) Scale back writers
6) Monitor for 1 hour

---

## 13. Common Failure Modes

- migration lock timeout
- index creation too slow
- table lock blocks online traffic
- schema drift due to manual edits
- missing permissions

Mitigations:
- use concurrent index creation
- run heavy operations off-peak
- lock down DB permissions
- enforce migration-only changes

---

## Links

- Website: https://m0club.com/
- X (Twitter): https://x.com/M0Clubonx
