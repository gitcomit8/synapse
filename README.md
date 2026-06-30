# Synapse Backend

Rust/Axum backend for Synapse - WhatsApp for Work.

## Setup

1. Copy `.env.example` to `.env` and fill in your credentials
2. Run migrations: `sqlx migrate run`
3. Start server: `cargo run`

## Project Structure

```
src/
├── main.rs          # Entry point
├── config.rs        # Configuration loading
├── db/              # Database pool and migrations
├── routes/          # API route handlers
│   ├── auth.rs
│   ├── workspace.rs
│   ├── channel.rs
│   ├── message.rs
│   └── board.rs
├── ws/              # WebSocket handling
│   ├── handler.rs
│   └── manager.rs
└── middleware/      # Middleware (auth, etc.)
    └── auth.rs
```

## API

- `GET /health` - Health check
- `POST /api/v1/auth/send-otp` - Send phone OTP
- `POST /api/v1/auth/verify-otp` - Verify OTP and get tokens
- `GET /api/v1/auth/me` - Get current user

## Docs

- `docs/specs/` — Feature matrix, roadmap, development plan, API specs
- `docs/standards/` — Engineering standards, API design guidelines
- `docs/diagrams/` — Architecture diagrams, DB schema

## Tech Stack

- **Language:** Rust (stable)
- **Framework:** Axum
- **Database:** PostgreSQL (Supabase)
- **ORM:** SQLx
- **Cache:** Redis (Upstash)
- **Storage:** Cloudflare R2
- **Auth:** Firebase Phone OTP
