# CourierX API

The HTTP API for [CourierX](https://courierx.io) — an open-source, self-hosted email service.

Written in Rust with Axum and SQLx. Accepts `POST /v1/emails` and writes to a Postgres-backed queue, where [courierx-worker](https://github.com/Miransas/courierx-worker) picks them up for delivery.

## Stack

- **Language:** Rust (edition 2021)
- **Framework:** Axum 0.7
- **Database:** Postgres 16+ via SQLx 0.8
- **Auth:** Argon2id password hashing + JWT
- **Migrations:** `sqlx::migrate!` (compile-time, embedded)

## Endpoints

| Method | Path           | Description                              | Auth         |
|--------|----------------|------------------------------------------|--------------|
| GET    | `/health`      | Liveness probe                           | None         |
| POST   | `/v1/emails`   | Queue a transactional email              | API key      |

The `POST /v1/emails` body is **Resend-compatible** — same request and response shape, so existing Resend SDKs work by changing only the base URL.

### Example

```bash
curl -X POST http://localhost:8080/v1/emails \
  -H "Authorization: Bearer cx_live_..." \
  -H "Content-Type: application/json" \
  -d '{
    "from": "you@yourdomain.com",
    "to": "user@example.com",
    "subject": "Welcome",
    "html": "<p>Hello.</p>"
  }'
```

Response (202 Accepted):

```json
{ "id": "4f8a2c91-7b3d-4e6a-9c5f-1a8b2d4e6f01", "status": "queued" }
```

## Local development

### Prerequisites

- Rust 1.85+ (`rustup default stable`)
- Postgres 16+ running locally
- `sqlx-cli` installed: `cargo install sqlx-cli --no-default-features --features postgres`

### Setup

```bash
git clone https://github.com/Miransas/courierx-api.git
cd courierx-api

# Create the database
createdb courierx

# Copy the example env, then edit as needed
cp .env.example .env

# Run migrations
sqlx migrate run

# Start the API
cargo run
```

The server listens on `http://localhost:8080` by default. Configure via `.env`:

```env
DATABASE_URL=postgres://courierx:courierx@localhost:5432/courierx
PORT=8080
JWT_SECRET=change-me-in-production
RUST_LOG=courierx_api=debug,tower_http=debug
```

### Generating a development API key

```bash
cargo run --example gen_api_key
```

This prints a usable `cx_live_...` key and its Argon2 hash. Insert the row into `api_keys` to start sending requests.

## Schema

Three tables: `workspaces`, `api_keys`, `emails`. Migrations live in `migrations/`. The API is the source of truth for the schema — the worker reads the same Postgres database but does not run migrations.

## Other repos in the CourierX project

- [courierx-web](https://github.com/sardorazimov/courierx-web) — the marketing site at courierx.io
- [courierx-worker](https://github.com/Miransas/courierx-worker) — the queue consumer that delivers emails
- `courierx-console` — the dashboard (private until launch)

## Brand

CourierX is built under the [Miransas](https://miransas.com) brand by [@sardorazimov](https://github.com/sardorazimov).

## License

MIT — see [LICENSE](./LICENSE)
