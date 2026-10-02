// The locale table and the path helpers, in plain JavaScript so `node --test` loads them and the
// middleware, the layout and the client share one copy. The design:
// docs/superpowers/specs/2026-10-02-explorer-i18n-design.md.

export const DEFAULT_LOCALE = 'en';

/** @typedef {{ code: string, tag: string, name: string, dir: 'ltr' | 'rtl' }} LocaleInfo */

/** @type {LocaleInfo[]} */
export const LOCALES = [
  { code: 'en', tag: 'en', name: 'English', dir: 'ltr' },
  { code: 'ru', tag: 'ru', name: 'Русский', dir: 'ltr' },
  { code: 'zh', tag: 'zh-Hans', name: '中文（简体）', dir: 'ltr' },
  { code: 'zh-hk', tag: 'zh-Hant-HK', name: '中文（繁體）', dir: 'ltr' },
  { code: 'ko', tag: 'ko', name: '한국어', dir: 'ltr' },
  { code: 'id', tag: 'id', name: 'Bahasa Indonesia', dir: 'ltr' },
  { code: 'ms', tag: 'ms', name: 'Bahasa Melayu', dir: 'ltr' },
  { code: 'ja', tag: 'ja', name: '日本語', dir: 'ltr' },
  { code: 'ar', tag: 'ar', name: 'العربية', dir: 'rtl' },
  { code: 'fa', tag: 'fa', name: 'فارسی', dir: 'rtl' },
  { code: 'es', tag: 'es', name: 'Español', dir: 'ltr' },
  { code: 'pt', tag: 'pt', name: 'Português', dir: 'ltr' },
  { code: 'de', tag: 'de', name: 'Deutsch', dir: 'ltr' },
  { code: 'fr', tag: 'fr', name: 'Français', dir: 'ltr' },
  { code: 'it', tag: 'it', name: 'Italiano', dir: 'ltr' },
  { code: 'pl', tag: 'pl', name: 'Polski', dir: 'ltr' },
];

export const LOCALE_CODES = LOCALES.map((l) => l.code);
const BY_CODE = new Map(LOCALES.map((l) => [l.code, l]));

/** @param {unknown} code */
export const isLocale = (code) => typeof code === 'string' && BY_CODE.has(code);

/** @param {string} code @returns {LocaleInfo} */
export const localeInfo = (code) => BY_CODE.get(code) ?? LOCALES[0];

/** The same table as randprotocol.org's; change both together. */
const COUNTRIES = {
  id: 'ID',
  ms: 'MY BN',
  ru: 'RU BY KZ KG',
  zh: 'CN',
  'zh-hk': 'HK MO TW',
  ko: 'KR',
  ja: 'JP',
  ar: 'SA AE EG QA KW BH OM JO IQ LB MA DZ TN LY SD YE SY PS MR',
  fa: 'IR AF',
  es: 'ES MX AR CO CL PE VE EC GT CU BO DO HN PY SV NI CR PA UY PR',
  pt: 'BR PT AO MZ',
  de: 'DE AT CH LI',
  fr: 'FR MC',
  it: 'IT SM',
  pl: 'PL',
};

/** @type {Record<string, string>} ISO 3166-1 alpha-2 (upper case) -> locale code */
export const COUNTRY_TO_LOCALE = Object.fromEntries(
  Object.entries(COUNTRIES).flatMap(([code, list]) => list.split(' ').map((cc) => [cc, code])),
);

/** @param {string | null | undefined} cc */
export const localeForCountry = (cc) =>
  (typeof cc === 'string' && COUNTRY_TO_LOCALE[cc.toUpperCase()]) || DEFAULT_LOCALE;

/** @param {string} pathname */
const firstSegment = (pathname) => pathname.split('?')[0].split('#')[0].split('/')[1] ?? '';

/** The locale named by a path's whole first segment, else English. @param {string} pathname */
export const localeFromPath = (pathname) => {
  const first = firstSegment(pathname);
  return isLocale(first) ? first : DEFAULT_LOCALE;
};

/** The path without its locale prefix; always starts with a slash. @param {string} pathname */
export const stripLocale = (pathname) => {
  const first = firstSegment(pathname);
  if (!isLocale(first)) return pathname || '/';
  const rest = pathname.slice(1 + first.length);
  return rest === '' || rest.startsWith('?') || rest.startsWith('#') ? `/${rest}` : rest;
};

/** Paths the app does not serve as pages; never prefixed. */
export const UNPREFIXED = /^\/(api|ws|_next|viewing|rpc|cdn-cgi|\.well-known)(\/|$)/;

/**
 * A root-relative page path in a locale: "/blocks" -> "/ru/blocks". Absolute URLs, anchors,
 * the API, static files and anything with an extension come back unchanged.
 * @param {string} locale @param {string} path
 */
export const localizePath = (locale, path) => {
  if (!path.startsWith('/') || path.startsWith('//')) return path;
  if (UNPREFIXED.test(path)) return path;
  if (/\.[a-z0-9]{2,5}(\?|#|$)/i.test(path)) return path;
  const bare = stripLocale(path);
  if (locale === DEFAULT_LOCALE) return bare;
  return bare === '/' || bare.startsWith('/?') || bare.startsWith('/#')
    ? `/${locale}/${bare.slice(1)}`
    : `/${locale}${bare}`;
};
