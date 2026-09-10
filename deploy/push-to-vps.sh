#!/usr/bin/env bash
# Run locally: push-to-vps.sh <ip> [domain]  — rsyncs the repo to /root/randscan and runs vps-setup.sh there.
# The server must already run a shrugg-node with RPC on 127.0.0.1:8545 (see ../fullnode/deploy).
set -euo pipefail
IP=$1; DOMAIN=${2:-randscan.org}
cd "$(dirname "$0")/.."
KEY=${SSH_KEY:-~/.ssh/id_ed25519}
SSH_OPTS="-i $KEY -o StrictHostKeyChecking=accept-new -o ConnectTimeout=15"
rsync -az --delete -e "ssh $SSH_OPTS" \
    --exclude target --exclude .git --exclude frontend/node_modules --exclude frontend/.next \
    ./ root@$IP:/root/randscan/
ssh $SSH_OPTS root@$IP "bash /root/randscan/deploy/vps-setup.sh $DOMAIN"
