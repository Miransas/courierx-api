# CourierX

Self-hosted transactional email API. A Resend alternative you can run on your own infrastructure.

> **Status:** Alpha. Under active development. Not production-ready yet.

## Why

Transactional email providers charge $20+/month for volumes that cost cents at the SMTP layer. CourierX wraps AWS SES (and eventually self-hosted Postfix) behind a clean HTTP API and dashboard, so you pay infrastructure costs, not subscription fees.

## Features

**Working now**
- `POST /v1/emails` — queue an email for delivery
- API key authentication (Argon2-hashed)
- Postgres-backed queue (no Redis dependency)

**Planned**
- AWS SES delivery worker
- Web dashboard (send logs, API key management, metrics)
- Webhooks (delivered, bounced, complained)
- Domain verification (SPF, DKIM, DMARC helper)
- JavaScript and Go SDKs
- Self-hosted Postfix backend (Phase 2)

## Architecture

CourierX is split across multiple repositories:

| Repo | Language | Purpose |
|------|----------|---------|
| `courierx-api` | Rust | HTTP API server (this repo) |
| `courierx-worker` | Rust | Queue consumer, SES delivery |
| `courierx-web` | Next.js | Dashboard at console.courierx.io |
| `courierx-sdk-js` | TypeScript | `@courierx/node` |
| `courierx-cli` | Go | Command-line client |
| `courierx-docs` | Nextra | docs.courierx.io |

## Quick start

Requirements: Rust stable, PostgreSQL 14+.

```bash
git clone https://github.com/miransas/courierx-api
cd courierx-api
cp .env.example .env
# edit .env with your DATABASE_URL
cargo run
```

The server listens on `:8080`. Verify it's up:

```bash
curl localhost:8080/health
# {"status":"ok"}
```

### Sending an email

Once you've inserted an API key into the `api_keys` table:

```bash
curl -X POST localhost:8080/v1/emails \
  -H "Authorization: Bearer cx_live_<your_key>" \
  -H "Content-Type: application/json" \
  -d '{
    "from": "hello@yourdomain.com",
    "to": "user@example.com",
    "subject": "Hello from CourierX",
    "html": "<p>It works.</p>"
  }'
# {"id":"<uuid>","status":"queued"}
```

The email sits in the `emails` table until the worker picks it up.

## Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `DATABASE_URL` | — | Postgres connection string (required) |
| `PORT` | `8080` | HTTP listen port |
| `RUST_LOG` | `info` | Tracing filter |

## Development

```bash
cargo check          # fast type-check
cargo clippy         # lints
cargo fmt            # format
cargo test           # run tests
```

Migrations run automatically on startup.

## License

MIT — see [LICENSE](./LICENSE).

---

Part of [Miransas](https://miransas.com).