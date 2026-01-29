# RandScan

A blockchain explorer for the Rand Protocol built with a Rust backend and Next.js frontend.

## Architecture

- **Backend**: Rust (Axum web framework, SQLx for PostgreSQL)
- **Frontend**: Next.js 14 with TypeScript and Tailwind CSS
- **Database**: PostgreSQL 16

## Prerequisites

- Rust 1.75+
- Node.js 20+
- PostgreSQL 16
- A running Solana/Rand RPC node

## Running Locally (Terminal)

### 1. Database Setup

Start PostgreSQL and create the database:

```bash
# Using Docker for PostgreSQL
docker run -d \
  --name randscan-postgres \
  -e POSTGRES_USER=randscan \
  -e POSTGRES_PASSWORD=randscan \
  -e POSTGRES_DB=randscan \
  -p 5432:5432 \
  postgres:16-alpine
```

### 2. Backend (Rust API)

```bash
# Copy environment file
cp .env.example .env

# Edit .env to configure your RPC_URL and other settings
# Default DATABASE_URL: postgres://randscan:randscan@localhost:5432/randscan

# Build and run the API server
cargo run --release --bin randscan-api
```

The API server will start on `http://localhost:3000`.

#### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `DATABASE_URL` | PostgreSQL connection string | `postgres://randscan:randscan@localhost:5432/randscan` |
| `RPC_URL` | Solana/Rand RPC endpoint | `http://localhost:8899` |
| `API_HOST` | API bind address | `0.0.0.0` |
| `API_PORT` | API port | `3000` |
| `RUST_LOG` | Log level | `info` |

### 3. Frontend (Next.js)

```bash
cd frontend

# Install dependencies
npm install

# Copy environment file
cp .env.local.example .env.local

# Edit .env.local if needed (defaults work for local development)
# NEXT_PUBLIC_API_URL=http://localhost:3000
# NEXT_PUBLIC_WS_URL=ws://localhost:3000

# Run development server
npm run dev
```

The frontend will start on `http://localhost:3001`.

#### Frontend Scripts

| Command | Description |
|---------|-------------|
| `npm run dev` | Start development server on port 3001 |
| `npm run build` | Build for production |
| `npm run start` | Start production server |
| `npm run lint` | Run ESLint |

## Running with Docker Compose

The easiest way to run the entire stack:

```bash
# Build and start all services
docker compose up --build

# Or run in detached mode
docker compose up -d --build

# View logs
docker compose logs -f

# Stop all services
docker compose down

# Stop and remove volumes (clears database)
docker compose down -v
```

### Services

| Service | Port | Description |
|---------|------|-------------|
| `postgres` | 5432 | PostgreSQL database |
| `api` | 3000 | Rust backend API |
| `frontend` | 3001 | Next.js frontend |

### Accessing the Application

- **Frontend**: http://localhost:3001
- **API**: http://localhost:3000

### Docker Compose Environment

The docker-compose.yml configures:
- PostgreSQL with persistent volume storage
- API server connected to PostgreSQL
- Frontend with API/WebSocket URLs pointing to the backend

To customize, you can override environment variables or modify `docker-compose.yml`.

## Project Structure

```
randscan/
├── crates/
│   ├── randscan-api/       # REST API server
│   ├── randscan-core/      # Core types and utilities
│   ├── randscan-db/        # Database models and queries
│   ├── randscan-indexer/   # Blockchain indexer
│   ├── randscan-ws/        # WebSocket server
│   └── randscan-frontend/  # (Deprecated) Leptos frontend
├── frontend/               # Next.js frontend
│   ├── src/
│   │   ├── app/           # App Router pages
│   │   ├── components/    # React components
│   │   ├── hooks/         # Custom hooks
│   │   ├── lib/           # Utilities
│   │   └── types/         # TypeScript types
│   └── ...
├── migrations/             # SQL migrations
├── docker-compose.yml
├── Dockerfile             # Backend Dockerfile
└── Cargo.toml             # Rust workspace
```

## License

MIT
