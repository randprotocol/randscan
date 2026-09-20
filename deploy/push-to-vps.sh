#!/usr/bin/env bash
# Run locally: push-to-vps.sh <ip> [domain]  — rsyncs the repo to /root/randscan and runs vps-setup.sh there.
# The server must already run a rand-node with RPC on 127.0.0.1:8545 (see ../fullnode/deploy).
set -euo pipefail
IP=$1; DOMAIN=${2:-randscan.org}
cd "$(dirname "$0")/.."
KEY=${SSH_KEY:-~/.ssh/id_ed25519}
SSH_OPTS="-i $KEY -o StrictHostKeyChecking=accept-new -o ConnectTimeout=15"

# The viewing-key module is built here, not on the server (vps-setup.sh has no wasm toolchain), and
# both key kinds — a viewing key and a transaction key — open through it. Refuse to ship the
# frontend without its binary, or with one older than the crate it is built from.
WASM=frontend/public/viewing/randscan_viewing_bg.wasm
REBUILD="(cd crates/randscan-viewing && wasm-pack build --release --target web --out-dir ../../frontend/public/viewing --out-name randscan_viewing)"
[ -s "$WASM" ] || { echo "missing $WASM; build it first: $REBUILD" >&2; exit 1; }
newer=$(find crates/randscan-viewing/src crates/randscan-viewing/Cargo.toml -newer "$WASM" -print -quit)
[ -z "$newer" ] || { echo "$WASM is older than $newer; rebuild it: $REBUILD" >&2; exit 1; }
rsync -az --delete -e "ssh $SSH_OPTS" \
    --exclude target --exclude .git --exclude frontend/node_modules --exclude frontend/.next \
    ./ root@$IP:/root/randscan/
ssh $SSH_OPTS root@$IP "bash /root/randscan/deploy/vps-setup.sh $DOMAIN"
