// The dictionary of a locale on the server, English deep-merged underneath so a key a
// translation lacks renders in English instead of breaking the page.
import en from './messages/en.json';
import ru from './messages/ru.json';
import zh from './messages/zh.json';
import zhHk from './messages/zh-hk.json';
import ko from './messages/ko.json';
import id from './messages/id.json';
import ms from './messages/ms.json';
import ja from './messages/ja.json';
import ar from './messages/ar.json';
import fa from './messages/fa.json';
import es from './messages/es.json';
import pt from './messages/pt.json';
import de from './messages/de.json';
import fr from './messages/fr.json';
import it from './messages/it.json';
import pl from './messages/pl.json';
import hi from './messages/hi.json';
import ur from './messages/ur.json';
import ps from './messages/ps.json';
import ta from './messages/ta.json';
import { merge } from './merge';
import { DEFAULT_LOCALE } from './locales';

export type Messages = typeof en;

const FILES: Record<string, unknown> = {
  ru, zh, 'zh-hk': zhHk, ko, id, ms, ja, ar, fa, es, pt, de, fr, it, pl, hi, ur, ps, ta,
};

const cache = new Map<string, Messages>();

export function getMessages(code: string): Messages {
  if (code === DEFAULT_LOCALE || !(code in FILES)) return en;
  let hit = cache.get(code);
  if (!hit) {
    hit = merge(en, FILES[code]);
    cache.set(code, hit);
  }
  return hit;
}

const lookup = (m: unknown, key: string): unknown =>
  key
    .split('.')
    .reduce<unknown>(
      (o, k) => (o && typeof o === 'object' ? (o as Record<string, unknown>)[k] : undefined),
      m,
    );

/** A string from a dictionary by dotted key; the key itself when missing (visible, never a crash). */
export function pick(m: Messages, key: string, vars?: Record<string, string | number>): string {
  const v = lookup(m, key);
  if (typeof v !== 'string') return key;
  return vars ? v.replace(/\{(\w+)\}/g, (s, k) => (k in vars ? String(vars[k]) : s)) : v;
}

/** Any value (array, object) from a dictionary by dotted key. */
export function pickAny<T>(m: Messages, key: string): T {
  return lookup(m, key) as T;
}
