import type { TransactionKind } from '@/types';

export const TOKEN_SYMBOL = 'SHRUGG';
export const TOKEN_DECIMALS = 9;

// ---------------------------------------------------------------------------
// Class names
// ---------------------------------------------------------------------------

export function cn(...classes: (string | undefined | null | false)[]): string {
  return classes.filter(Boolean).join(' ');
}

// ---------------------------------------------------------------------------
// Time — all chain timestamps are milliseconds since the epoch
// ---------------------------------------------------------------------------

/** Relative age of a millisecond timestamp, e.g. "12s ago". */
export function formatTimestamp(timestampMs: number): string {
  if (!Number.isFinite(timestampMs)) return '—';

  const diff = Date.now() - timestampMs;
  const seconds = Math.floor(Math.abs(diff) / 1000);
  const minutes = Math.floor(seconds / 60);
  const hours = Math.floor(minutes / 60);
  const days = Math.floor(hours / 24);

  if (diff < 0) return 'just now';
  if (seconds < 60) return seconds <= 1 ? 'just now' : `${seconds}s ago`;
  if (minutes < 60) return minutes === 1 ? '1 min ago' : `${minutes} mins ago`;
  if (hours < 24) return hours === 1 ? '1 hour ago' : `${hours} hours ago`;
  if (days < 7) return days === 1 ? '1 day ago' : `${days} days ago`;

  return new Date(timestampMs).toLocaleDateString('en-US', {
    month: 'short',
    day: 'numeric',
    year: 'numeric',
  });
}

/** Absolute rendering of a millisecond timestamp. */
export function formatDateTime(timestampMs: number): string {
  if (!Number.isFinite(timestampMs)) return '—';
  return new Date(timestampMs).toLocaleString('en-US', {
    month: 'short',
    day: 'numeric',
    year: 'numeric',
    hour: 'numeric',
    minute: '2-digit',
    second: '2-digit',
    hour12: true,
  });
}

/** Human duration for a millisecond span, e.g. "1.20s" or "2m 5s". */
export function formatDurationMs(ms: number): string {
  if (!Number.isFinite(ms) || ms <= 0) return '—';
  if (ms < 1000) return `${Math.round(ms)}ms`;
  if (ms < 60_000) return `${(ms / 1000).toFixed(2)}s`;
  const minutes = Math.floor(ms / 60_000);
  const seconds = Math.round((ms % 60_000) / 1000);
  return `${minutes}m ${seconds}s`;
}

// ---------------------------------------------------------------------------
// Amounts — decimal strings of smallest units
// ---------------------------------------------------------------------------

function toBigInt(value: string | number | bigint): bigint | null {
  try {
    if (typeof value === 'bigint') return value;
    if (typeof value === 'number') return BigInt(Math.trunc(value));
    const trimmed = value.trim();
    if (trimmed === '') return null;
    return BigInt(trimmed);
  } catch {
    return null;
  }
}

/**
 * Convert smallest units to a decimal string with up to `decimals` places,
 * trailing zeros trimmed. Integer part is grouped with commas.
 */
export function formatUnits(
  value: string | number | bigint | null | undefined,
  decimals: number = TOKEN_DECIMALS
): string {
  if (value === null || value === undefined) return '0';
  const units = toBigInt(value);
  if (units === null) return String(value);

  const negative = units < BigInt(0);
  const abs = negative ? -units : units;
  const divisor = BigInt(10) ** BigInt(decimals);
  const whole = abs / divisor;
  const fraction = abs % divisor;

  const wholeStr = whole.toLocaleString('en-US');
  let out = wholeStr;

  if (fraction > BigInt(0)) {
    const fracStr = fraction.toString().padStart(decimals, '0').replace(/0+$/, '');
    if (fracStr.length > 0) out = `${wholeStr}.${fracStr}`;
  }

  return negative ? `-${out}` : out;
}

/** Same as formatUnits, suffixed with the token symbol. */
export function formatAmount(
  value: string | number | bigint | null | undefined,
  decimals: number = TOKEN_DECIMALS,
  symbol: string = TOKEN_SYMBOL
): string {
  return `${formatUnits(value, decimals)} ${symbol}`;
}

/**
 * Stake is SHRUGG that left the pool into the validator register, so it is units with the
 * token's nine decimals. `suffix` is appended after the symbol ("1,000 SHRUGG total stake").
 */
export function formatStake(
  value: string | number | bigint | null | undefined,
  suffix = ''
): string {
  const base = formatAmount(value ?? 0);
  return suffix ? `${base} ${suffix}` : base;
}

/**
 * An amount in a bridged asset's own smallest unit (the registry index says which asset; index
 * 0 or null is SHRUGG and gets the usual nine-decimal rendering). Bridged units have no fixed
 * decimals on this chain.
 */
export function formatAssetAmount(
  units: string | number | bigint | null | undefined,
  assetIndex: number | null | undefined
): string {
  if (units === null || units === undefined) return '—';
  if (assetIndex === null || assetIndex === undefined || assetIndex === 0) {
    return formatAmount(units);
  }
  return `${formatUnits(units, 0)} units of asset #${assetIndex}`;
}

// ---------------------------------------------------------------------------
// Numbers
// ---------------------------------------------------------------------------

export function formatNumber(n: number | bigint | null | undefined): string {
  if (n === null || n === undefined) return '—';
  if (typeof n === 'number' && !Number.isFinite(n)) return '—';
  return n.toLocaleString('en-US');
}

export function formatCompactNumber(n: number): string {
  if (!Number.isFinite(n)) return '—';
  if (Math.abs(n) < 1000) return n.toString();
  if (Math.abs(n) < 1_000_000) return `${(n / 1000).toFixed(1)}K`;
  if (Math.abs(n) < 1_000_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
  return `${(n / 1_000_000_000).toFixed(1)}B`;
}

export function formatPercentage(value: number, decimals: number = 2): string {
  if (!Number.isFinite(value)) return '—';
  return `${value.toFixed(decimals)}%`;
}

/** Byte counts, used for zk proof sizes. */
export function formatBytes(bytes: number | null | undefined): string {
  if (bytes === null || bytes === undefined || !Number.isFinite(bytes)) return '—';
  if (bytes < 1024) return `${formatNumber(bytes)} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(2)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
}

/** Uptime in seconds rendered as e.g. "3h 12m", "5m 3s", "2d 4h". */
export function formatConnectedTime(seconds: number | null | undefined): string {
  if (seconds === null || seconds === undefined || !Number.isFinite(seconds) || seconds < 0) {
    return '—';
  }

  const total = Math.floor(seconds);
  if (total < 60) return `${total}s`;

  const days = Math.floor(total / 86_400);
  const hours = Math.floor((total % 86_400) / 3600);
  const minutes = Math.floor((total % 3600) / 60);
  const secs = total % 60;

  if (days > 0) return `${days}d ${hours}h`;
  if (hours > 0) return `${hours}h ${minutes}m`;
  return `${minutes}m ${secs}s`;
}

// ---------------------------------------------------------------------------
// Hashes and addresses
// ---------------------------------------------------------------------------

export function shortenHash(
  hash: string | null | undefined,
  startChars: number = 8,
  endChars: number = 6
): string {
  if (!hash) return '—';
  if (hash.length <= startChars + endChars + 3) return hash;
  return `${hash.slice(0, startChars)}…${hash.slice(-endChars)}`;
}

const HEX64 = /^(0x)?[0-9a-fA-F]{64}$/;
const BASE58 = /^[1-9A-HJ-NP-Za-km-z]{32,44}$/;
const DECIMAL = /^\d+$/;

export function isHash(value: string): boolean {
  return HEX64.test(value.trim());
}

export function isAddress(value: string): boolean {
  const v = value.trim();
  // A 64-hex string is a hash, not an address, even though it is base58-shaped.
  return !HEX64.test(v) && BASE58.test(v);
}

export function isHeight(value: string): boolean {
  return DECIMAL.test(value.trim());
}

export type SearchQueryType = 'height' | 'hash' | 'address' | 'unknown';

export function getSearchQueryType(query: string): SearchQueryType {
  const trimmed = query.trim();
  if (trimmed === '') return 'unknown';
  if (isHeight(trimmed)) return 'height';
  if (isHash(trimmed)) return 'hash';
  if (isAddress(trimmed)) return 'address';
  return 'unknown';
}

// ---------------------------------------------------------------------------
// Transaction kinds
// ---------------------------------------------------------------------------

/** Kinds the API accepts as a `?kind=` filter (`other` is display-only). */
export const TRANSACTION_KINDS: TransactionKind[] = [
  'transfer',
  'mint',
  'deploy',
  'call',
  'bond',
  'unbond',
  'withdraw',
  'bridge_attest',
  'bridge_burn',
];

const KIND_LABELS: Record<TransactionKind, string> = {
  transfer: 'Transfer (shielded)',
  mint: 'Mint (faucet)',
  deploy: 'Deploy',
  call: 'Call (confidential)',
  bond: 'Bond',
  unbond: 'Unbond',
  withdraw: 'Withdraw',
  bridge_attest: 'Bridge in (attestation)',
  bridge_burn: 'Bridge out (burn)',
  other: 'Other',
};

const KIND_SHORT_LABELS: Record<TransactionKind, string> = {
  transfer: 'Transfer',
  mint: 'Mint',
  deploy: 'Deploy',
  call: 'Call',
  bond: 'Bond',
  unbond: 'Unbond',
  withdraw: 'Withdraw',
  bridge_attest: 'Bridge in',
  bridge_burn: 'Bridge out',
  other: 'Other',
};

const KIND_BADGE_CLASSES: Record<TransactionKind, string> = {
  transfer: 'badge badge-transfer',
  mint: 'badge badge-mint',
  deploy: 'badge badge-deploy',
  call: 'badge badge-call',
  bond: 'badge badge-stake',
  unbond: 'badge badge-stake',
  withdraw: 'badge badge-stake',
  bridge_attest: 'badge badge-bridge',
  bridge_burn: 'badge badge-bridge',
  other: 'badge badge-neutral',
};

/** Kinds whose `amount` is in a bridged asset's own unit rather than SHRUGG units. */
export function amountIsBridged(kind: string): boolean {
  return kind === 'bridge_attest' || kind === 'bridge_burn';
}

// ---------------------------------------------------------------------------
// Bridge
// ---------------------------------------------------------------------------

/**
 * Bridged assets used to carry 8 decimals on the account chain; on the shielded chain a
 * bridged amount is in the asset's own smallest unit (see formatAssetAmount). Kept for the
 * legacy rendering of a two-part "tokens (units)" string.
 */
export const BRIDGED_DECIMALS = 8;

const BRIDGE_CHAIN_NAMES: Record<number, string> = {
  1: 'Rand',
  2: 'Ethereum',
  3: 'BSC',
  4: 'Tron',
  5: 'Solana',
};

/** "Ethereum (2)" for a known bridge chain id, "chain 9" otherwise. */
export function formatBridgeChain(id: number | null | undefined): string {
  if (id === null || id === undefined) return '—';
  const name = BRIDGE_CHAIN_NAMES[id];
  return name ? `${name} (${id})` : `chain ${id}`;
}

/** Bridged units as a decimal number of tokens (8 decimals) with the raw units alongside. */
export function formatBridgedAmount(units: string | null | undefined): string {
  if (units === null || units === undefined) return '—';
  return `${formatUnits(units, BRIDGED_DECIMALS)} tokens (${formatUnits(units, 0)} units)`;
}

/**
 * Bridge destinations are 32-byte hex; a 20-byte EVM/Tron address arrives left-padded with
 * 24 zero hex digits. Strip that padding for display and keep the full value for copying.
 */
export function formatBridgeAddress(hex: string): string {
  const clean = hex.toLowerCase().replace(/^0x/, '');
  if (clean.length === 64 && clean.startsWith('0'.repeat(24))) {
    return `0x${clean.slice(24)}`;
  }
  return clean;
}

export function getKindLabel(kind: string): string {
  return KIND_LABELS[kind as TransactionKind] ?? kind;
}

export function getKindShortLabel(kind: string): string {
  return KIND_SHORT_LABELS[kind as TransactionKind] ?? kind;
}

export function getKindBadgeClass(kind: string): string {
  return KIND_BADGE_CLASSES[kind as TransactionKind] ?? 'badge badge-neutral';
}

// ---------------------------------------------------------------------------
// Nodes
// ---------------------------------------------------------------------------

interface GeoLike {
  city: string | null;
  region: string | null;
  country: string | null;
}

/** "Singapore, SG" style location line; null geo becomes "Unknown location". */
export function formatLocation(geo: GeoLike | null | undefined): string {
  if (!geo) return 'Unknown location';
  const parts = [geo.city, geo.region, geo.country].filter(
    (part): part is string => typeof part === 'string' && part.trim() !== ''
  );
  // A city that repeats as its own region (city states) reads badly twice.
  const deduped = parts.filter((part, index) => parts.indexOf(part) === index);
  return deduped.length > 0 ? deduped.join(', ') : 'Unknown location';
}

/** "1.2.3.4:9000", falling back gracefully when either half is missing. */
export function formatEndpoint(
  ip: string | null | undefined,
  port: number | null | undefined
): string {
  if (!ip) return '—';
  return port === null || port === undefined ? ip : `${ip}:${port}`;
}

const NODE_ROLE_BADGE_CLASSES: Record<string, string> = {
  validator: 'badge badge-transfer',
  observer: 'badge badge-mint',
  peer: 'badge badge-neutral',
};

export function getNodeRoleBadgeClass(role: string): string {
  return NODE_ROLE_BADGE_CLASSES[role] ?? 'badge badge-neutral';
}

export function getNodeRoleLabel(role: string): string {
  return role.charAt(0).toUpperCase() + role.slice(1);
}

// ---------------------------------------------------------------------------
// Clipboard
// ---------------------------------------------------------------------------

export async function copyToClipboard(text: string): Promise<boolean> {
  try {
    await navigator.clipboard.writeText(text);
    return true;
  } catch {
    return false;
  }
}
