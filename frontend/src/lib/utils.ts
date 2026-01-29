// Format timestamp to relative time
export function formatTimestamp(timestamp: number): string {
  const now = Date.now();
  const ts = timestamp * 1000; // Convert to milliseconds if in seconds
  const diff = now - ts;

  const seconds = Math.floor(diff / 1000);
  const minutes = Math.floor(seconds / 60);
  const hours = Math.floor(minutes / 60);
  const days = Math.floor(hours / 24);

  if (seconds < 60) {
    return seconds <= 1 ? 'just now' : `${seconds}s ago`;
  }
  if (minutes < 60) {
    return minutes === 1 ? '1 min ago' : `${minutes} mins ago`;
  }
  if (hours < 24) {
    return hours === 1 ? '1 hour ago' : `${hours} hours ago`;
  }
  if (days < 7) {
    return days === 1 ? '1 day ago' : `${days} days ago`;
  }

  // Format as date for older timestamps
  return new Date(ts).toLocaleDateString('en-US', {
    month: 'short',
    day: 'numeric',
    year: days > 365 ? 'numeric' : undefined,
  });
}

// Format timestamp to full date/time
export function formatDateTime(timestamp: number): string {
  const ts = timestamp * 1000;
  return new Date(ts).toLocaleString('en-US', {
    month: 'short',
    day: 'numeric',
    year: 'numeric',
    hour: 'numeric',
    minute: '2-digit',
    second: '2-digit',
    hour12: true,
  });
}

// Format token amount with decimals
export function formatTokenAmount(
  amount: string | number,
  decimals: number = 9,
  maxDecimals: number = 4
): string {
  const value = typeof amount === 'string' ? BigInt(amount) : BigInt(amount);
  const divisor = BigInt(10 ** decimals);
  const wholePart = value / divisor;
  const fractionalPart = value % divisor;

  if (fractionalPart === BigInt(0)) {
    return formatNumber(Number(wholePart));
  }

  const fractionalStr = fractionalPart.toString().padStart(decimals, '0');
  const trimmedFractional = fractionalStr.slice(0, maxDecimals).replace(/0+$/, '');

  if (trimmedFractional === '') {
    return formatNumber(Number(wholePart));
  }

  return `${formatNumber(Number(wholePart))}.${trimmedFractional}`;
}

// Format lamports to ATLAS/SHRUG display
export function formatLamports(lamports: string | number): string {
  return formatTokenAmount(lamports, 9, 4);
}

// Shorten hash/address for display
export function shortenHash(hash: string, startChars: number = 4, endChars: number = 4): string {
  if (hash.length <= startChars + endChars + 3) {
    return hash;
  }
  return `${hash.slice(0, startChars)}...${hash.slice(-endChars)}`;
}

// Format number with commas
export function formatNumber(n: number | bigint): string {
  return n.toLocaleString('en-US');
}

// Format large numbers with K/M/B suffixes
export function formatCompactNumber(n: number): string {
  if (n < 1000) return n.toString();
  if (n < 1_000_000) return `${(n / 1000).toFixed(1)}K`;
  if (n < 1_000_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
  return `${(n / 1_000_000_000).toFixed(1)}B`;
}

// Format percentage
export function formatPercentage(value: number, decimals: number = 2): string {
  return `${value.toFixed(decimals)}%`;
}

// Format TPS
export function formatTps(tps: number): string {
  return tps.toFixed(0);
}

// Get transaction type display name
export function getTransactionTypeLabel(type: string): string {
  const labels: Record<string, string> = {
    transfer: 'Transfer',
    stake: 'Stake',
    vote: 'Vote',
    program: 'Program',
    system: 'System',
    unknown: 'Unknown',
  };
  return labels[type] || type;
}

// Get transaction status color class
export function getStatusColorClass(status: string): string {
  switch (status) {
    case 'success':
      return 'text-green-500';
    case 'failed':
      return 'text-red-500';
    case 'pending':
      return 'text-yellow-500';
    default:
      return 'text-gray-500';
  }
}

// Get validator status color class
export function getValidatorStatusColorClass(status: string): string {
  switch (status) {
    case 'active':
      return 'text-green-500';
    case 'delinquent':
      return 'text-yellow-500';
    case 'inactive':
      return 'text-gray-500';
    default:
      return 'text-gray-500';
  }
}

// Validate if string is a valid Solana address (base58, 32-44 chars)
export function isValidAddress(address: string): boolean {
  return /^[1-9A-HJ-NP-Za-km-z]{32,44}$/.test(address);
}

// Validate if string is a valid transaction signature
export function isValidSignature(signature: string): boolean {
  return /^[1-9A-HJ-NP-Za-km-z]{86,88}$/.test(signature);
}

// Validate if string is a valid block hash
export function isValidBlockHash(hash: string): boolean {
  return /^[1-9A-HJ-NP-Za-km-z]{43,44}$/.test(hash);
}

// Validate if string is a valid slot number
export function isValidSlot(slot: string): boolean {
  return /^\d+$/.test(slot) && parseInt(slot, 10) >= 0;
}

// Determine search query type
export function getSearchQueryType(query: string): 'slot' | 'signature' | 'address' | 'unknown' {
  const trimmed = query.trim();

  if (isValidSlot(trimmed)) {
    return 'slot';
  }
  if (isValidSignature(trimmed)) {
    return 'signature';
  }
  if (isValidAddress(trimmed)) {
    return 'address';
  }

  return 'unknown';
}

// Copy text to clipboard
export async function copyToClipboard(text: string): Promise<boolean> {
  try {
    await navigator.clipboard.writeText(text);
    return true;
  } catch {
    return false;
  }
}

// Class name utility (simple version of clsx)
export function cn(...classes: (string | undefined | null | false)[]): string {
  return classes.filter(Boolean).join(' ');
}
