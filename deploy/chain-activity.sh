#!/usr/bin/env bash
# One unit of activity on the shielded testnet, then a check that randscan.org indexed it.
# Each run does ONE step and advances a rotation kept in STATE_DIR:
#   transfer  a proved 1.5 SHRUGG send from wallet 1 to wallet 2 (faucet-mints first when
#             wallet 1 is running low, so the loop never starves)
#   deploy    a fresh private_payment guest (a new threshold each time gives a new program id)
#   call      a proved call of the last program this loop deployed (input transcript published)
# Meant to be fired every few minutes by a scheduler; a lock makes overlapping runs a no-op
# (two proofs against the same wallet would race for the same notes).
#
# Usage: deploy/chain-activity.sh [transfer|deploy|call]   (no argument = next in rotation)
# Env:   BIN, WALLETS (or W1/W2 key files), RPC, RPC_FALLBACK, EXPLORER, STATE_DIR, ROTATION, NICE (e.g. "nice -n 10" when the
#        prover shares a box with a validator; proofs are ~1.3 MB at constraint set 5 and take
#        1.5–4 min, so run this where the node is local: a 2.6 MB hex submit over a slow uplink
#        outruns the wallet's 15 s RPC timeout).
set -uo pipefail
BIN=${BIN:-$HOME/Github/randprotocol/fullnode/bin-03c9fb9}
WALLETS=${WALLETS:-$HOME/Github/randprotocol/fullnode/wallets}
RPC=${RPC:-http://127.0.0.1:8545}
EXPLORER=${EXPLORER:-https://randscan.org/api/v1}
STATE_DIR=${STATE_DIR:-${TMPDIR:-/tmp}/chain-activity}
NICE=${NICE:-}
mkdir -p "$STATE_DIR"
W1=${W1:-$WALLETS/shielded-1.key.json}
W2=${W2:-$WALLETS/shielded-2.key.json}
LOG=$STATE_DIR/activity.log

log() { printf '%s %s\n' "$(date -u +%FT%TZ)" "$*" | tee -a "$LOG"; }
rpc() { curl -s --max-time 10 "$RPC" -H 'content-type: application/json' -d "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"$1\",\"params\":$2}"; }
submitted() { grep "submitted $2 " <<<"$1" | awk '{print $3}' | head -1; }
# Poll the explorer until it serves the transaction with the expected kind.
explorer_tx() {
  local hash=$1 want=$2 i body
  for i in $(seq 1 30); do
    body=$(curl -s --max-time 10 "$EXPLORER/transactions/$hash")
    if grep -q "\"kind\":\"$want\"" <<<"$body"; then echo "$body"; return 0; fi
    sleep 3
  done
  echo "$body"; return 1
}

if ! mkdir "$STATE_DIR/lock" 2>/dev/null; then
  log "skip: a previous step is still running (lock $STATE_DIR/lock)"; exit 0
fi
trap 'rmdir "$STATE_DIR/lock"' EXIT

# A node that is catching up serves a stale anchor; fall back to RPC_FALLBACK (say, an SSH
# tunnel to a synced droplet: `ssh -f -N -L 18545:127.0.0.1:8545 root@<node>`) when the
# primary is down or syncing.
RPC_FALLBACK=${RPC_FALLBACK:-http://127.0.0.1:18545}
status=$(rpc shrugg_status '[]')
if [ -z "$status" ] || grep -q '"syncing":true' <<<"$status"; then
  alt=$(RPC=$RPC_FALLBACK rpc shrugg_status '[]')
  if [ -n "$alt" ] && ! grep -q '"syncing":true' <<<"$alt"; then
    log "primary node $RPC is $([ -z "$status" ] && echo down || echo syncing); using $RPC_FALLBACK"
    RPC=$RPC_FALLBACK
  fi
fi
chain=$(rpc shrugg_chainId '[]' | grep -o '"result":[0-9]*' | cut -d: -f2)
head=$(rpc shrugg_getHead '[]' | grep -o '"height":[0-9]*' | cut -d: -f2)
[ -n "$chain" ] || { log "FAIL: no node at $RPC"; exit 1; }

n=$(cat "$STATE_DIR/n" 2>/dev/null || echo 0); n=$((n+1)); echo "$n" > "$STATE_DIR/n"
# ROTATION can drop a step, e.g. "transfer deploy" while a node build refuses calls.
read -r -a rotation <<<"${ROTATION:-transfer deploy call}"
step=${1:-${rotation[$(( (n-1) % ${#rotation[@]} ))]}}
log "step $n $step (chain $chain, head $head)"
t0=$(date +%s)

case $step in
  transfer)
    bal=$("$BIN/shrugg" balance --key "$W1" --rpc "$RPC" 2>/dev/null | grep -o 'balance: [0-9.]*' | awk '{print $2}')
    if [ -n "$bal" ] && [ "${bal%%.*}" -lt 20 ]; then
      out=$("$BIN/shrugg" faucet --key "$W1" --rpc "$RPC" --amount 100 2>&1); h=$(submitted "$out" mint)
      if [ -n "$h" ] && explorer_tx "$h" mint >/dev/null; then log "step $n mint ok $h (wallet 1 had $bal SHRUGG)"; else log "step $n mint FAIL ${h:-nohash}: $(tail -1 <<<"$out")"; fi
    fi
    to=$("$BIN/shrugg" --key "$W2" address | tail -1)
    out=$($NICE "$BIN/shrugg" send "$to" 1.5 --key "$W1" --rpc "$RPC" 2>&1); h=$(submitted "$out" transfer)
    if [ -n "$h" ] && body=$(explorer_tx "$h" transfer); then
      log "step $n transfer ok $h block $(grep -o '"height":[0-9]*' <<<"$body" | head -1 | cut -d: -f2) ($(( $(date +%s) - t0 )) s)"
    else log "step $n transfer FAIL ${h:-nohash}: $(tail -1 <<<"$out")"; exit 1; fi ;;
  deploy)
    pj="$STATE_DIR/pp-$n.json"
    "$BIN/shrugg" program build --guest private_payment --arg $((100 + n)) --out "$pj" >/dev/null 2>&1
    out=$($NICE "$BIN/shrugg" program deploy "$pj" --key "$W1" --rpc "$RPC" 2>&1); h=$(submitted "$out" deploy)
    pid=$(grep -o 'program id: [0-9a-f]*' <<<"$out" | awk '{print $3}')
    if [ -n "$h" ] && explorer_tx "$h" deploy >/dev/null; then
      echo "$pid" > "$STATE_DIR/program"
      log "step $n deploy ok $h program $pid ($(( $(date +%s) - t0 )) s)"
    else log "step $n deploy FAIL ${h:-nohash}: $(tail -1 <<<"$out")"; exit 1; fi ;;
  call)
    pid=$(cat "$STATE_DIR/program" 2>/dev/null)
    if [ -z "$pid" ]; then log "step $n call skipped: nothing deployed yet by this loop"; exit 0; fi
    # 400 + 250 against a threshold of 100 + <deploy step>: outputs [1, 0, 650 - threshold, ...]
    out=$($NICE "$BIN/shrugg" call "$pid" --input 400 --input 250 --input 0 --input 0 --key "$W1" --rpc "$RPC" 2>&1); h=$(submitted "$out" call)
    if [ -n "$h" ] && body=$(explorer_tx "$h" call) && grep -q '"h_in":"[0-9a-f]' <<<"$body"; then
      log "step $n call ok $h program $pid outputs $(grep -o '"outputs":\[[^]]*\]' <<<"$body" | head -1) $(grep -o '"input_envelope_len":[0-9]*' <<<"$body") ($(( $(date +%s) - t0 )) s)"
    else log "step $n call FAIL ${h:-nohash}: $(tail -1 <<<"$out")"; exit 1; fi ;;
  *) log "unknown step $step"; exit 2 ;;
esac

ex=$(curl -s --max-time 10 "$EXPLORER/health")
log "step $n explorer $(grep -o '"status":"[a-z]*"' <<<"$ex") lag $(grep -o '"lag":[0-9-]*' <<<"$ex" | cut -d: -f2) chain $(curl -s --max-time 10 "$EXPLORER/stats" | grep -o '"chain_id":[0-9]*' | cut -d: -f2)"
