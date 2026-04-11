# Excentra Exchange

A production-grade centralized cryptocurrency exchange backend built in Rust. Features a custom in-memory matching engine, real-time WebSocket streaming, JWT authentication, PostgreSQL persistence, and live price seeding via CoinGecko.

> **Status:** Phases 1–10 complete. Backend API, WebSocket streaming, admin panel, and Docker setup done. Ethereum Sepolia testnet integration planned.
---

## Overview

Excentra models how real exchanges like Binance or Kraken work under the hood — user accounts, live order books, and a matching engine that fills buy and sell orders in real time using price-time priority (FIFO).

**Supported pairs:** BTC/USDT · ETH/USDT · SOL/USDT

**Core capabilities:**
- Limit and market order placement with immediate matching
- Price-time priority (FIFO) matching engine with partial fill support
- Self-trade prevention (STP) — limit orders checked against user's own resting orders
- Wash trading protection in the matching loop
- Real-time order book, trade feed, and 24h ticker via WebSocket
- Balance management with holds — funds locked on placement, released on cancel
- JWT authentication with refresh token rotation via httpOnly cookies
- Role-based access control (user / admin)
- Paginated order and trade history with status and pair filtering
- Admin panel — user management, role promotion, account suspension, system metrics
- Order book seeded with live prices from CoinGecko at startup

---

## Getting Started

### Option A — Docker (recommended)

**Prerequisites:** [Docker](https://www.docker.com/)

```bash
git clone https://github.com/manlikeHB/excentra.git
cd excentra
cp .env.example .env   # fill in your values
docker compose up
```

Migrations run automatically on first start. Server available at `http://localhost:3000`.

---

### Option B — Manual

**Prerequisites:**
- [Rust](https://rustup.rs/) (stable)
- [PostgreSQL](https://www.postgresql.org/) (v14+)
- [sqlx-cli](https://github.com/launchbadge/sqlx/tree/main/sqlx-cli)

```bash
cargo install sqlx-cli --no-default-features --features rustls,postgres
```

**1. Clone the repo**
```bash
git clone https://github.com/manlikeHB/excentra.git
cd excentra
```

**2. Configure environment**
```bash
cp .env.example .env
```

`.env`:
```env
DATABASE_URL=postgres://postgres:password@localhost:5432/excentra
JWT_SECRET=your-secret-key-here
API_VERSION=v1
PORT=3000
RUST_LOG=info,tower_http=debug
```

**3. Run the server**
```bash
cargo run
```

Migrations run automatically on startup. On first run the server will also fetch live prices from CoinGecko and seed the order book.

You should see:
```
INFO excentra: Server listening port=3000
```

**4. Explore the API**

Interactive API documentation is available at `http://localhost:3000/docs`. Explore all endpoints, view request and response schemas, and make live requests directly from the browser. Click **Authorize** and paste your JWT token to test protected endpoints.

The raw OpenAPI spec is also available at `http://localhost:3000/api-docs/openapi.json` for import into Postman or client generation.

---

## Architecture

```
┌───────────────────────────────────────────────────────────────┐
│                         Clients                               │
│              (REST via HTTP · WebSocket)                      │
└────────────────────────┬──────────────────────────────────────┘
                         │
┌────────────────────────▼──────────────────────────────────────┐
│                      axum HTTP Server                         │
│   ┌─────────────┐  ┌──────────────┐  ┌──────────────────┐    │
│   │  Auth Layer │  │  REST Routes │  │  WebSocket Route  │    │
│   │ (JWT Middleware) │  /api/v1/* │  │  /ws             │    │
│   └─────────────┘  └──────┬───────┘  └────────┬─────────┘    │
└──────────────────────────┼───────────────────┼───────────────┘
                           │                   │
          ┌────────────────▼──┐     ┌──────────▼─────────────┐
          │   Service Layer   │     │    WebSocket Manager    │
          │  OrderService     │     │  (subscription routing) │
          │  TradeService     │     └──────────┬─────────────┘
          │  BalanceService   │                │
          │  AuthService      │      broadcast::Sender<WsEvent>
          └────────┬──────────┘                │
                   │                           │
     ┌─────────────▼─────────────┐             │
     │     Matching Engine       │─────────────┘
     │  Exchange (multi-pair)    │   emits trades, book updates
     │  OrderBook (per-pair)     │
     │  BTreeMap<Price, VecDeque>│
     └─────────────┬─────────────┘
                   │
     ┌─────────────▼─────────────┐
     │       PostgreSQL          │
     │  users · orders · trades  │
     │  balances · trading_pairs │
     └───────────────────────────┘
```

### Matching Engine

Each trading pair has its own in-memory `OrderBook`:

- **Bids:** `BTreeMap<Reverse<Decimal>, VecDeque<Order>>` — highest price first
- **Asks:** `BTreeMap<Decimal, VecDeque<Order>>` — lowest price first
- **Cancel index:** `HashMap<Uuid, Decimal>` — O(1) cancel lookup

The matcher walks the opposite side of the book, filling at resting order prices (FIFO within each price level). Partial fills are supported. Matched trades are persisted to PostgreSQL and broadcast over WebSocket.

### WebSocket Architecture

A `tokio::broadcast` channel carries events from the engine to all connected clients. The WebSocket manager routes events to subscribers based on their active subscriptions — only BTC/USDT subscribers receive BTC/USDT order book updates.

### Balance Holds

When a user places an order, funds move from `available` → `held`. On match, held funds transfer to the counterparty. On cancel, they return to available. This prevents double-spend without a separate ledger.

---

## Tech Stack

| Layer | Technology | Why |
|---|---|---|
| Language | Rust | Memory safety, performance, strong type system |
| HTTP Framework | axum | Ergonomic, tower-compatible, async-first |
| Async Runtime | tokio | Industry standard for async Rust |
| Database | PostgreSQL + sqlx | Compile-time query checking, async support |
| Auth | JWT + argon2 | Stateless auth; argon2 is the gold standard for password hashing |
| Financial Math | rust_decimal | Exact decimal arithmetic |
| Serialization | serde + serde_json | Universal Rust serialization |
| Logging | tracing + tracing-subscriber | Structured, async-aware logging |
| Price Seeding | CoinGecko API | Free, real market prices, no API key required |

---

## Key Technical Decisions

### Prices as strings, not floats

All prices and quantities are `rust_decimal::Decimal` internally and serialized as JSON strings. IEEE 754 floating point cannot represent most decimal fractions exactly — using floats for financial data is a latent correctness bug.

### BTreeMap for the order book

`BTreeMap` keeps prices sorted automatically: O(log n) to find the best price, O(k) to walk k price levels during matching. A `HashMap` would give O(1) lookup but no ordering — unusable for a matching engine. Within each price level, `VecDeque` provides O(1) FIFO access for time-priority matching.

### In-memory matching, persistent everything else

Matching happens in memory for speed. Every placement, fill, and cancellation is immediately persisted to PostgreSQL. On restart, the order book rebuilds from open orders in the database — the engine is stateless, the database is the source of truth.

### System user for order book seeding

Seed orders placed at startup are backed by a real system user with pre-seeded balances. The engine code path is identical for seed and real orders — no special-casing, no divergent logic.

### MutexGuard scope and async safety

Holding a `MutexGuard` across an `.await` is a deadlock risk in async Rust. Guards are always dropped (by limiting scope or using temporaries) before any async calls. This was a non-obvious constraint in async Rust lifetime management that shaped several service-layer patterns.

---

## Roadmap

| Phase | Description | Status |
|---|---|---|
| 1 | Project setup, domain types, database schema | ✅ Complete |
| 2 | Matching engine (in-memory, FIFO) | ✅ Complete |
| 3 | Database layer (sqlx queries) | ✅ Complete |
| 4 | Auth (JWT + argon2 + refresh tokens) | ✅ Complete |
| 5 | REST API | ✅ Complete |
| 6 | WebSocket streaming | ✅ Complete |
| 7 | Ticker service + CoinGecko price seeding | ✅ Complete |
| 8 | Observability (tracing, health check) | ✅ Complete |
| 9 | Ethereum Sepolia testnet integration | 📋 Planned |
| 10 | Admin panel (backend API) | ✅ Complete |
| 11 | Docker + docker-compose | ✅ Complete |
| 12 | Frontend (React / Next.js) | 🔄 Planned |
| 13 | Deployment | 📋 Planned |

### Blockchain Integration (Phase 9)

Deposit and withdrawal on Ethereum Sepolia testnet:
- HD wallet with per-user deposit addresses (BIP-44)
- Blockchain listener service watches for incoming transactions
- Balance credited after N block confirmations
- Signed withdrawal transactions broadcast via Alchemy

---

## Future Work

- **Fee system:** Maker/taker model with fee collection
- **Stop-loss / take-profit:** Triggered order types
- **Margin trading:** Leveraged positions and a liquidation engine
- **Order book snapshots:** Faster restarts under high load
- **Rate limiting:** Per-user API limits
- **More trading pairs:** Dynamic pair addition without restart

---

## License

MIT
