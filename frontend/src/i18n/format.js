// Every number, amount, size, duration and date the explorer prints, for one locale. Plain
// JavaScript so `node --test` checks it. English output is exactly what the explorer printed before
// localization; other locales get their own grouping and separators through Intl, always in Latin
// digits so an amount reads the same way as the hash beside it. The few words that Intl cannot
// supply ("units of asset #3", "chain 9") come from the dictionary's `fmt` block.

const TOKEN_SYMBOL = 'RAND';
const TOKEN_DECIMALS = 9;
const BRIDGED_DECIMALS = 8;
const DASH = '—';

/** The English words; a dictionary's `fmt` block has the same keys. */
export const EN_WORDS = {
  units: 'units',
  assetUnits: '{units} units of asset #{index}',
  bridged: '{tokens} tokens ({units} units)',
  chain: 'chain {id}',
  unknownLocation: 'Unknown location',
};

const fill = (s, vars) => s.replace(/\{(\w+)\}/g, (m, k) => (k in vars ? String(vars[k]) : m));

const toBigInt = (value) => {
  try {
    if (typeof value === 'bigint') return value;
    if (typeof value === 'number') return BigInt(Math.trunc(value));
    const trimmed = String(value).trim();
    return trimmed === '' ? null : BigInt(trimmed);
  } catch {
    return null;
  }
};

/**
 * @param {string} tag BCP 47 tag of the page ("en", "zh-Hans", "ar")
 * @param {typeof EN_WORDS} [words]
 */
export function makeFormat(tag, words = EN_WORDS) {
  const w = { ...EN_WORDS, ...words };
  const isEn = tag === 'en' || tag.startsWith('en-');
  const loc = `${tag}-u-nu-latn`;
  const nf = (opts) => new Intl.NumberFormat(loc, opts);
  const plain = nf();
  const decimalSep = plain.formatToParts(1.5).find((p) => p.type === 'decimal')?.value ?? '.';
  const unit = (u, display = 'narrow', digits) =>
    nf({
      style: 'unit',
      unit: u,
      unitDisplay: display,
      ...(digits === undefined ? {} : { minimumFractionDigits: digits, maximumFractionDigits: digits }),
    });
  const rtf = isEn ? null : new Intl.RelativeTimeFormat(loc, { numeric: 'auto' });

  const number = (n) => {
    if (n === null || n === undefined) return DASH;
    if (typeof n === 'number' && !Number.isFinite(n)) return DASH;
    return typeof n === 'bigint' ? n.toLocaleString(loc) : plain.format(n);
  };

  const units = (value, decimals = TOKEN_DECIMALS) => {
    if (value === null || value === undefined) return '0';
    const v = toBigInt(value);
    if (v === null) return String(value);
    const negative = v < 0n;
    const abs = negative ? -v : v;
    const divisor = 10n ** BigInt(decimals);
    const whole = (abs / divisor).toLocaleString(loc);
    const fraction = abs % divisor;
    let out = whole;
    if (fraction > 0n) {
      const frac = fraction.toString().padStart(decimals, '0').replace(/0+$/, '');
      if (frac) out = `${whole}${decimalSep}${frac}`;
    }
    return negative ? `-${out}` : out;
  };

  const amount = (value, decimals = TOKEN_DECIMALS, symbol = TOKEN_SYMBOL) =>
    `${units(value, decimals)} ${symbol}`;

  const dateOpts = { month: 'short', day: 'numeric', year: 'numeric' };

  return {
    tag,
    number,
    units,
    amount,
    stake: (value, suffix = '') => {
      const base = amount(value ?? 0);
      return suffix ? `${base} ${suffix}` : base;
    },
    compact: (n) => {
      if (!Number.isFinite(n)) return DASH;
      if (Math.abs(n) < 1000) return plain.format(n);
      return nf({ notation: 'compact', minimumFractionDigits: 1, maximumFractionDigits: 1 }).format(n);
    },
    percentage: (value, decimals = 2) =>
      Number.isFinite(value)
        ? nf({ style: 'percent', minimumFractionDigits: decimals, maximumFractionDigits: decimals }).format(value / 100)
        : DASH,
    bytes: (bytes) => {
      if (bytes === null || bytes === undefined || !Number.isFinite(bytes)) return DASH;
      const fixed = (n) => nf({ minimumFractionDigits: 2, maximumFractionDigits: 2 }).format(n);
      if (bytes < 1024) return `${number(bytes)} B`;
      if (bytes < 1024 * 1024) return `${fixed(bytes / 1024)} KB`;
      return `${fixed(bytes / (1024 * 1024))} MB`;
    },
    binaryBytes: (bytes) => {
      if (bytes === null || bytes === undefined || !Number.isFinite(bytes)) return DASH;
      const trim = (n) => (Number.isInteger(n) ? number(n) : nf({ minimumFractionDigits: 2, maximumFractionDigits: 2 }).format(n));
      if (bytes >= 1024 * 1024) return `${trim(bytes / (1024 * 1024))} MiB`;
      if (bytes >= 1024) return `${trim(bytes / 1024)} KiB`;
      return `${number(bytes)} B`;
    },
    duration: (ms) => {
      if (!Number.isFinite(ms) || ms <= 0) return DASH;
      if (ms < 1000) return unit('millisecond').format(Math.round(ms));
      if (ms < 60_000) return unit('second', 'narrow', 2).format(ms / 1000);
      const m = Math.floor(ms / 60_000);
      const s = Math.round((ms % 60_000) / 1000);
      return `${unit('minute').format(m)} ${unit('second').format(s)}`;
    },
    connected: (seconds) => {
      if (seconds === null || seconds === undefined || !Number.isFinite(seconds) || seconds < 0) return DASH;
      const total = Math.floor(seconds);
      if (total < 60) return unit('second').format(total);
      const d = Math.floor(total / 86_400);
      const h = Math.floor((total % 86_400) / 3600);
      const m = Math.floor((total % 3600) / 60);
      const s = total % 60;
      if (d > 0) return `${unit('day').format(d)} ${unit('hour').format(h)}`;
      if (h > 0) return `${unit('hour').format(h)} ${unit('minute').format(m)}`;
      return `${unit('minute').format(m)} ${unit('second').format(s)}`;
    },
    mintWindow: (secs) => {
      if (secs >= 3600 && secs % 3600 === 0) return unit('hour', 'long').format(secs / 3600);
      if (secs >= 60 && secs % 60 === 0) return unit('minute', 'long').format(secs / 60);
      return unit('second', 'long').format(secs);
    },
    /** "12s ago" style; `now` is injectable for tests. */
    ago: (ms, now = Date.now()) => {
      if (!Number.isFinite(ms)) return DASH;
      const diff = now - ms;
      const sec = Math.floor(Math.abs(diff) / 1000);
      const min = Math.floor(sec / 60);
      const hr = Math.floor(min / 60);
      const day = Math.floor(hr / 24);
      if (day >= 7) return new Date(ms).toLocaleDateString(loc, dateOpts);
      if (isEn) {
        if (diff < 0) return 'just now';
        if (sec < 60) return sec <= 1 ? 'just now' : `${sec}s ago`;
        if (min < 60) return min === 1 ? '1 min ago' : `${min} mins ago`;
        if (hr < 24) return hr === 1 ? '1 hour ago' : `${hr} hours ago`;
        return day === 1 ? '1 day ago' : `${day} days ago`;
      }
      if (diff < 0 || sec <= 1) return rtf.format(0, 'second');
      if (sec < 60) return rtf.format(-sec, 'second');
      if (min < 60) return rtf.format(-min, 'minute');
      if (hr < 24) return rtf.format(-hr, 'hour');
      return rtf.format(-day, 'day');
    },
    /** `timeZone` is for tests; pages use the viewer's own. */
    dateTime: (ms, timeZone) =>
      Number.isFinite(ms)
        ? new Date(ms).toLocaleString(loc, {
            ...dateOpts,
            hour: 'numeric',
            minute: '2-digit',
            second: '2-digit',
            ...(isEn ? { hour12: true } : {}),
            ...(timeZone ? { timeZone } : {}),
          })
        : DASH,
    bridgeUnits: (u, symbol) =>
      u === null || u === undefined ? DASH : `${units(u, BRIDGED_DECIMALS)} ${symbol ?? w.units}`,
    bridged: (u) =>
      u === null || u === undefined
        ? DASH
        : fill(w.bridged, { tokens: units(u, BRIDGED_DECIMALS), units: units(u, 0) }),
    bridgeChain: (id, name) => {
      if (id === null || id === undefined) return DASH;
      return name ? `${name} (${id})` : fill(w.chain, { id });
    },
    assetUnits: (u, index) => fill(w.assetUnits, { units: units(u, 0), index }),
    location: (geo) => {
      if (!geo) return w.unknownLocation;
      const parts = [geo.city, geo.region, geo.country].filter(
        (p) => typeof p === 'string' && p.trim() !== '',
      );
      const deduped = parts.filter((p, i) => parts.indexOf(p) === i);
      return deduped.length ? deduped.join(', ') : w.unknownLocation;
    },
  };
}
