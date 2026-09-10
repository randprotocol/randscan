#!/usr/bin/env bash
# Runs ON the server as root. Usage: vps-setup.sh [domain]
# Installs PostgreSQL, Node 20 and Caddy, builds the API and the frontend from /root/randscan,
# installs systemd services and a Caddyfile. Safe to re-run (rebuild + restart, data kept).
set -euo pipefail
DOMAIN=${1:-randscan.org}
SRC=/root/randscan
ENV_DIR=/etc/randscan
DB_PASS_FILE=$ENV_DIR/db.pass

export DEBIAN_FRONTEND=noninteractive

# --- packages -------------------------------------------------------------------------------
apt-get update -qq
apt-get install -y -qq postgresql postgresql-contrib curl gnupg ca-certificates debian-keyring debian-archive-keyring apt-transport-https rsync >/dev/null

if ! command -v node >/dev/null || [ "$(node -v | cut -c2-3)" -lt 20 ]; then
    curl -fsSL https://deb.nodesource.com/setup_20.x | bash - >/dev/null
    apt-get install -y -qq nodejs >/dev/null
fi

if ! command -v caddy >/dev/null; then
    curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/gpg.key' | gpg --dearmor -o /usr/share/keyrings/caddy-stable-archive-keyring.gpg
    curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/debian.deb.txt' > /etc/apt/sources.list.d/caddy-stable.list
    apt-get update -qq
    apt-get install -y -qq caddy >/dev/null
fi

# --- database -------------------------------------------------------------------------------
mkdir -p $ENV_DIR
if [ ! -f $DB_PASS_FILE ]; then
    tr -dc 'A-Za-z0-9' </dev/urandom | head -c 32 > $DB_PASS_FILE
fi
DB_PASS=$(cat $DB_PASS_FILE)
systemctl enable --now postgresql
# (no `| grep -q` here: with pipefail a SIGPIPE from grep -q aborts the script)
if [ -z "$(sudo -u postgres psql -tAc "SELECT 1 FROM pg_roles WHERE rolname='randscan'")" ]; then
    sudo -u postgres psql -v ON_ERROR_STOP=1 -c "CREATE ROLE randscan LOGIN PASSWORD '$DB_PASS'"
fi
sudo -u postgres psql -v ON_ERROR_STOP=1 -c "ALTER ROLE randscan PASSWORD '$DB_PASS'" >/dev/null
if [ -z "$(sudo -u postgres psql -tAc "SELECT 1 FROM pg_database WHERE datname='randscan'")" ]; then
    sudo -u postgres psql -v ON_ERROR_STOP=1 -c "CREATE DATABASE randscan OWNER randscan"
fi

PUBLIC_IP=$(curl -4 -s --max-time 10 https://api.ipify.org || hostname -I | awk '{print $1}')
cat > $ENV_DIR/api.env <<ENV
DATABASE_URL=postgres://randscan:$DB_PASS@127.0.0.1:5432/randscan
RPC_URL=http://127.0.0.1:8545
NODE_PUBLIC_IP=$PUBLIC_IP
API_HOST=127.0.0.1
API_PORT=3000
POLL_INTERVAL_MS=1000
BATCH_SIZE=200
RUST_LOG=info,sqlx=warn,tower_http=warn
ENV
chmod 600 $ENV_DIR/api.env

# --- build ----------------------------------------------------------------------------------
source /root/.cargo/env
cd $SRC
echo "building randscan-api..."
cargo build --release --bin randscan-api > /tmp/randscan-build.log 2>&1 || { tail -40 /tmp/randscan-build.log; exit 1; }
grep -E "^\s+Finished" /tmp/randscan-build.log || true
test -x target/release/randscan-api
install -m 755 target/release/randscan-api /usr/local/bin/randscan-api

echo "building frontend..."
cd $SRC/frontend
npm ci --no-audit --no-fund >/dev/null
NEXT_PUBLIC_API_URL="" NEXT_TELEMETRY_DISABLED=1 npm run build >/dev/null
rm -rf /opt/randscan-frontend
mkdir -p /opt/randscan-frontend
cp -r .next/standalone/. /opt/randscan-frontend/
mkdir -p /opt/randscan-frontend/.next
cp -r .next/static /opt/randscan-frontend/.next/static
[ -d public ] && cp -r public /opt/randscan-frontend/public

# --- services -------------------------------------------------------------------------------
install -m 644 $SRC/deploy/randscan-api.service /etc/systemd/system/randscan-api.service
install -m 644 $SRC/deploy/randscan-frontend.service /etc/systemd/system/randscan-frontend.service
sed "s/__DOMAIN__/$DOMAIN/g" $SRC/deploy/Caddyfile > /etc/caddy/Caddyfile
caddy validate --config /etc/caddy/Caddyfile --adapter caddyfile >/dev/null

systemctl daemon-reload
systemctl enable randscan-api randscan-frontend caddy >/dev/null
systemctl restart randscan-api
systemctl restart randscan-frontend
systemctl reload caddy || systemctl restart caddy

# --- firewall -------------------------------------------------------------------------------
if command -v ufw >/dev/null; then
    ufw allow 80/tcp >/dev/null
    ufw allow 443/tcp >/dev/null
fi

sleep 4
systemctl is-active randscan-api randscan-frontend caddy || true
curl -s http://127.0.0.1:3000/api/v1/health; echo
curl -s -o /dev/null -w "frontend http %{http_code}\n" http://127.0.0.1:3001/
