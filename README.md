# Excentra

**High-performance crypto exchange infrastructure**

Excentra is a full-stack centralized exchange built in Rust. It features a custom in-memory matching engine, real-time WebSocket streaming, JWT authentication with refresh token rotation, PostgreSQL persistence, and a React/Next.js trading interface.

![Excentra Trading Interface](./docs/screenshot.png)

---

## Features

**Trading Engine**
- Price-time priority (FIFO) matching with partial fill support
- Limit and market order types
- Self-trade prevention — orders are never matched against the same user
- Balance holds — funds locked on placement, released on cancel or fill
- Order book seeded with live prices from CoinGecko at startup

**API & Streaming**
- REST API with paginated order and trade history
- WebSocket streaming: order book, trade feed, 24h ticker, and private order status
- Rate limiting on authentication endpoints
- Structured JSON logging via `tracing`
- Health endpoint reporting DB status, uptime, WebSocket connections, and orders processed

**Auth**
- JWT access tokens + refresh token rotation via httpOnly cookies
- argon2 password hashing
- Password reset via email (SMTP)
- Role-based access control (user / admin)

**Admin**
- User management: role promotion, account suspension
- Dynamic trading pair and asset management — add new pairs without restart
- System metrics: active connections, 24h volume per pair, total trades, uptime

**Frontend**
- Real-time order book with depth visualization
- Candlestick price chart with 1m / 5m / 15m / 1h / 1D timeframes
- Order form with live best bid/ask pre-fill
- Open orders panel with cancel support
- Portfolio dashboard with balance overview and order/trade history

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
│   │ (JWTMiddlw) │  │  /api/v1/*   │  │       /ws        │    │
│   └─────────────┘  └──────┬───────┘  └────────┬─────────┘    │
└──────────────────────────┼───────────────────┼───────────────┘
                           │                   │
          ┌────────────────▼──┐     ┌──────────▼─────────────┐
          │   Service Layer   │     │    WebSocket Manager    │
          │  OrderService     │────▶│  (subscription routing) │
          │  BalanceService   │     └──────────┬─────────────┘
          │  AuthService      │                │
          │  TickerService    │      broadcast::Sender<WsEvent>
          └────────┬──────────┘                │
                   │         MatchResult        │
     ┌─────────────▼─────────────┐             │
     │     Matching Engine       │             │
     │  Exchange (multi-pair)    │             │
     │  OrderBook (per-pair)     │             │
     │  BTreeMap<Price, VecDeque>│             │
     └─────────────┬─────────────┘             │
                   │                           │
                   └───────────────────────────┘
              Service layer broadcasts trades,
              book snapshots, and order status
                   │
     ┌─────────────▼─────────────┐
     │       PostgreSQL          │
     │  users · orders · trades  │
     │  balances · assets        │
     │  trading_pairs            │
     └───────────────────────────┘
```

### Matching Engine

Each trading pair has its own in-memory `OrderBook`:

- **Bids:** `BTreeMap<Reverse<Decimal>, VecDeque<Order>>` — highest price first
- **Asks:** `BTreeMap<Decimal, VecDeque<Order>>` — lowest price first
- **Cancel index:** `HashMap<Uuid, (Decimal, OrderSide)>` — maps order ID to its price level and side, enabling O(1) cancel lookup without scanning the book

The matcher walks the opposite side of the book filling at resting order prices (FIFO within each price level). Partial fills are supported. Matched trades are persisted to PostgreSQL and broadcast over WebSocket. On restart, the book rebuilds from open orders in the database.

### WebSocket Channels

Connect to `ws://localhost:5098/ws`

| Channel | Description | Auth required |
|---|---|---|
| `orderbook:{symbol}` | Full snapshot on subscribe, then live updates | No |
| `trades:{symbol}` | Live trade feed | No |
| `ticker:{symbol}` | 24h stats updates | No |
| `orders:{user_id}` | Private order status updates | Yes |

**Subscribe:**
```json
{ "action": "subscribe", "channel": "orderbook:BTC/USDT" }
```

**Authenticate (for private channels):**
```json
{ "action": "auth", "token": "your-jwt-token" }
```

### Balance Model

When an order is placed, funds move from `available` → `held`. On fill, held funds transfer atomically to the counterparty via a PostgreSQL transaction. On cancel, they return to available. This prevents double-spend without a separate ledger.

---

## Tech Stack

| Layer | Technology | Why |
|---|---|---|
| Language | Rust | Memory safety, performance, strong type system |
| HTTP Framework | axum | Ergonomic, tower-compatible, async-first |
| Async Runtime | tokio | Industry standard for async Rust |
| Database | PostgreSQL + sqlx | Compile-time query checking, async support |
| Auth | JWT + argon2 | Stateless auth; argon2 is the gold standard for password hashing |
| Financial Math | rust_decimal | Exact decimal arithmetic — no floating-point drift |
| Serialization | serde + serde_json | Universal Rust serialization |
| Logging | tracing + tracing-subscriber | Structured, async-aware logging |
| API Docs | utoipa + Scalar | Auto-generated OpenAPI spec from code annotations |
| Price Seeding | CoinGecko API | Free real market prices, no API key required |
| Frontend | Next.js + TypeScript | React framework with type-safe API client |
| Styling | Tailwind CSS | Utility-first, consistent design tokens |

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

Services started:
- **API** → `http://localhost:5098`
- **Frontend** → `http://localhost:3000`
- **Mailpit** (local email UI) → `http://localhost:8025`
- **PostgreSQL** → `localhost:5433`

Migrations run automatically on first start. The order book is seeded with live prices from CoinGecko.

---

### Option B — Manual

**Prerequisites:**
- [Rust](https://rustup.rs/) (stable)
- [PostgreSQL](https://www.postgresql.org/) (v14+)
- [Node.js](https://nodejs.org/) (v20+) + [pnpm](https://pnpm.io/)
- [sqlx-cli](https://github.com/launchbadge/sqlx/tree/main/sqlx-cli)

```bash
cargo install sqlx-cli --no-default-features --features rustls,postgres
```

**1. Clone and configure**

```bash
git clone https://github.com/manlikeHB/excentra.git
cd excentra
cp .env.example .env
```

Edit `.env`:

```env
DATABASE_URL=postgres://postgres:password@localhost:5432/excentra
JWT_SECRET=your-secret-key-here
API_VERSION=v1
PORT=5098
RUST_LOG=info,tower_http=debug
SMTP_HOST=localhost
SMTP_PORT=1025
SMTP_FROM=noreply@excentra.dev
FRONTEND_URL=http://localhost:3000
```

**2. Start the backend**

```bash
cargo run
```

Migrations run automatically on startup.

```
INFO excentra: Server listening port=5098
```

**3. Start the frontend**

```bash
cd frontend
cp .env.example .env.local  # set NEXT_PUBLIC_API_URL and NEXT_PUBLIC_WS_URL
pnpm install
pnpm dev
```

Frontend available at `http://localhost:3000`.

**4. Explore the API**

Interactive API documentation is available at `http://localhost:5098/docs`. Click **Authorize** and paste your JWT token to test protected endpoints.

The raw OpenAPI spec is at `http://localhost:5098/api-docs/openapi.json`.

---

## Key Technical Decisions

### Prices as strings, not floats

All prices and quantities are `rust_decimal::Decimal` internally and serialized as JSON strings. IEEE 754 floating-point cannot represent most decimal fractions exactly — using floats for financial data is a latent correctness bug that compounds across arithmetic operations.

### BTreeMap for the order book

`BTreeMap` keeps prices sorted automatically: O(log n) to find the best price, O(k) to walk k levels during matching. A `HashMap` would give O(1) lookup but no ordering — unusable for a price-time priority engine. Within each price level, `VecDeque` provides O(1) FIFO access.

### In-memory matching, persistent everything else

Matching happens in memory for speed. Every placement, fill, and cancellation is immediately persisted to PostgreSQL inside a transaction. On restart, the book rebuilds from open orders in the database — the engine is stateless, the database is the source of truth.

### Broadcast channel for WebSocket fan-out

A single `tokio::broadcast` channel carries all engine events to all connected WebSocket clients. The WebSocket manager filters by subscription before forwarding — a BTC/USDT subscriber never receives ETH/USDT events. This keeps the hot path (matching) decoupled from the number of connected clients.

### System user for order book seeding

Seed orders placed at startup are backed by a real system user with pre-seeded balances. The engine code path is identical for seed and real orders — no special-casing, no divergent logic.

### In-memory rate limiting

Rate limiting is applied per IP address using an in-memory `DashMap` with a fixed sliding window per route. Covered endpoints include login, register, place order, order book, ticker, trades, deposit, withdraw, forgot-password, and reset-password. Limits and windows are compile-time constants — moving them to runtime config or Redis would be the natural next step for a distributed deployment.

### MutexGuard scope and async safety

Holding a `MutexGuard` across an `.await` is a deadlock risk in async Rust. Guards are always dropped (by limiting scope or using temporaries) before any async calls.

---

## API Reference

Full interactive documentation at `/docs` when the server is running. Key endpoint groups:

| Group | Endpoints |
|---|---|
| Auth | `POST /auth/register` · `POST /auth/login` · `POST /auth/logout` · `POST /auth/refresh` · `POST /auth/forgot-password` · `POST /auth/reset-password` |
| Orders | `POST /orders` · `GET /orders` · `GET /orders/:id` · `DELETE /orders/:id` |
| Market Data | `GET /orderbook/:symbol` · `GET /trades/:symbol` · `GET /ticker` · `GET /ticker/:symbol` · `GET /pairs/active` |
| Balances | `GET /balances` · `POST /balances/deposit` · `POST /balances/withdraw` |
| Users | `GET /users/me` · `PATCH /users/me` |
| Admin | `GET /admin/users` · `PATCH /admin/users/:id/role` · `PATCH /admin/users/:id/suspend` · `GET /admin/stats` · `POST /pairs` · `POST /assets` |
| System | `GET /health` |

---

## Blockchain Integration

Ethereum Sepolia testnet integration is planned — on-chain deposits with per-user HD wallet addresses, a blockchain listener service, and signed withdrawal transactions via Alchemy.

---

## Future Work

- **Fee system:** Maker/taker fee model with collection accounts
- **Stop-loss / take-profit:** Triggered order types
- **Margin trading:** Leveraged positions and a liquidation engine
- **Order book snapshots:** Faster restarts under high order volume
- **More trading pairs:** Already supported dynamically via the admin panel

---

## License

MIT