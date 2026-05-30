# Pepecoin Ord Indexer

ℹ️ Fork based on [apezord/ord-dogecoin](https://github.com/apezord/ord-dogecoin), adapted for **Pepecoin** (Pepinals, DRC-20, Pepemaps).

## Key differences

‼️ DISCLAIMER: OUR CODE MAY STILL HAVE BUGS

This indexer targets Pepecoin mainnet (RPC port **33873**) and includes Pepemap and DRC-20 indexing.

## API documentation

OpenAPI spec: [openapi.yaml](openapi.yaml). View it in the [Swagger Editor](https://editor.swagger.io/) via Import URL:

`https://raw.githubusercontent.com/reallyshadydev/pepecoin-ord-indexer/main/openapi.yaml`

## Prerequisites

1. Run a fully synced **Pepecoin Core** node with `-txindex`.
2. Configure RPC access (see [RPC credentials](#rpc-credentials) below).
3. Set `SUBSIDIES_PATH` and `STARTING_SATS_PATH` to the JSON files in this repo.

## RPC credentials

The indexer talks to `pepecoind` over JSON-RPC. **Never commit real usernames, passwords, or cookie files.**

### Option 1: Username & password in the RPC URL

Set `rpcuser` and `rpcpassword` in your Pepecoin config (`pepecoin.conf`), then pass them via `--rpc-url`:

```shell
# pepecoin.conf (example — use your own values)
txindex=1
rpcuser=your_rpc_user
rpcpassword=your_rpc_password
rpcport=33873
rpcbind=127.0.0.1
rpcallowip=127.0.0.1
```

```shell
export RPC_URL="http://your_rpc_user:your_rpc_password@127.0.0.1:33873"
ord --rpc-url="$RPC_URL" ...
```

For Docker, put the same value in `.env` (copy from `.env.example`):

```shell
RPC_URL=http://your_rpc_user:your_rpc_password@127.0.0.1:33873
```

### Option 2: Cookie file (recommended for local nodes)

If `pepecoind` generates `~/.pepecoin/.cookie`, point the indexer at it:

```shell
ord \
  --rpc-url=127.0.0.1:33873 \
  --cookie-file="$HOME/.pepecoin/.cookie" \
  ...
```

Or set the Pepecoin data directory (flag name is legacy):

```shell
ord --dogecoin-data-dir="$HOME/.pepecoin" ...
```

### Default ports

| Network | RPC port |
|---------|----------|
| Mainnet | `33873` |
| Testnet | `44873` |
| Signet  | `38332` |
| Regtest | `18332` |

## How to run

```shell
export RUST_LOG=info
export SUBSIDIES_PATH=/path/to/pepecoin-ord-indexer/subsidies.json
export STARTING_SATS_PATH=/path/to/pepecoin-ord-indexer/starting_sats.json
export RPC_URL="http://your_rpc_user:your_rpc_password@127.0.0.1:33873"

mkdir -p /mnt/ord-node/indexer-data-main

# Index only
ord \
  --rpc-url="$RPC_URL" \
  --data-dir=/mnt/ord-node/indexer-data-main \
  --nr-parallel-requests=16 \
  --first-inscription-height=186920 \
  --first-dune-height=186920 \
  --index-dunes \
  --index-transactions \
  --index-drc20 \
  --index-pepemaps \
  index

# Index + HTTP server
ord \
  --rpc-url="$RPC_URL" \
  --data-dir=/mnt/ord-node/indexer-data-main \
  --nr-parallel-requests=16 \
  --first-inscription-height=186920 \
  --first-dune-height=186920 \
  --index-dunes \
  --index-transactions \
  --index-drc20 \
  --index-pepemaps \
  server
```

`--index-transactions` stores transaction data (needed for DRC-20 / Pepemap indexing and better API performance).

`--nr-parallel-requests` controls parallel RPC calls during indexing — `16` is a reasonable default.

With all settings enabled, the database can require hundreds of GB when fully synced.

## Required env vars

At the repo root you'll find `subsidies.json` and `starting_sats.json`. Point ord at them before starting:

```shell
export SUBSIDIES_PATH=/path/to/pepecoin-ord-indexer/subsidies.json
export STARTING_SATS_PATH=/path/to/pepecoin-ord-indexer/starting_sats.json
```

## Docker

### Prerequisites

1. Linux host (or similar)
2. Synced `pepecoind` with `-txindex` and RPC configured (see above)
3. Docker + Docker Compose
4. Clone this repo and copy `.env.example` → `.env` with your `RPC_URL`

### Build

```shell
docker build -t pepecoin-ord-indexer .
```

### Run

```shell
docker compose up -d
```

### Stop cleanly

Use a timeout so the index database closes properly:

```shell
docker compose stop -t 600
docker compose down
```

## Original README

See [READMEFROMAPEZORD.md](READMEFROMAPEZORD.md) for upstream `ord` documentation.
