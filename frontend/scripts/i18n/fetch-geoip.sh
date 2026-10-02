#!/usr/bin/env bash
# Fetch DB-IP Country Lite (CC BY 4.0) for this month, falling back to last month, into $1
# (default frontend/geoip/dbip-country-lite.mmdb). Used by deploy/vps-setup.sh on the box and by
# hand here. The site reads it in process to pick a first visit's language; nothing is stored.
set -euo pipefail
out=${1:-"$(cd "$(dirname "$0")/../.." && pwd)/geoip/dbip-country-lite.mmdb"}
mkdir -p "$(dirname "$out")"
this=$(date -u +%Y-%m)
last=$(date -u -d '-1 month' +%Y-%m 2>/dev/null || date -u -v-1m +%Y-%m)
for m in "$this" "$last"; do
    url="https://download.db-ip.com/free/dbip-country-lite-$m.mmdb.gz"
    if curl -fsSL "$url" | gunzip > "$out.tmp" 2>/dev/null && [ -s "$out.tmp" ]; then
        mv "$out.tmp" "$out"
        echo "fetched $url -> $out"
        exit 0
    fi
done
echo "fetch-geoip: no database for $this or $last" >&2
rm -f "$out.tmp"
exit 1
