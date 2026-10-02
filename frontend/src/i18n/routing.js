// What the middleware does with a request, as a pure function the tests can call.
import {
  DEFAULT_LOCALE,
  LOCALES,
  UNPREFIXED,
  isLocale,
  localeForCountry,
  localizePath,
  stripLocale,
} from './locales.js';

const HAS_EXTENSION = /\.[a-z0-9]{2,5}$/i;

/**
 * @param {{ pathname: string, search: string, cookie: string | null | undefined, country: string | null | undefined }} req
 * @returns {{ kind: 'next' } | { kind: 'redirect', to: string, status: 302 | 308 } | { kind: 'rewrite', to: string }}
 */
export function decide({ pathname, search, cookie, country }) {
  const q = search || '';
  if (UNPREFIXED.test(pathname) || HAS_EXTENSION.test(pathname)) return { kind: 'next' };

  const first = pathname.split('/')[1] ?? '';
  if (first === DEFAULT_LOCALE) {
    return { kind: 'redirect', to: stripLocale(pathname) + q, status: 308 };
  }
  if (isLocale(first)) return { kind: 'next' };

  if (pathname === '/') {
    const chosen = isLocale(cookie) ? cookie : localeForCountry(country);
    if (chosen !== DEFAULT_LOCALE) {
      return { kind: 'redirect', to: localizePath(chosen, '/') + q, status: 302 };
    }
  }
  return { kind: 'rewrite', to: `/${DEFAULT_LOCALE}${pathname === '/' ? '' : pathname}${q}` };
}

/** The `Link` header: every language's URL for one unprefixed path, then x-default. */
export function alternatesHeader(origin, barePath) {
  const entries = LOCALES.map(
    (l) => `<${origin}${localizePath(l.code, barePath)}>; rel="alternate"; hreflang="${l.tag}"`,
  );
  entries.push(
    `<${origin}${localizePath(DEFAULT_LOCALE, barePath)}>; rel="alternate"; hreflang="x-default"`,
  );
  return entries.join(', ');
}

/** The first address of X-Forwarded-For, or null. @param {string | null | undefined} xff */
export function clientIp(xff) {
  if (!xff) return null;
  const first = xff.split(',')[0].trim();
  return first === '' ? null : first;
}
