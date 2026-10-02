// English unprefixed, fifteen languages prefixed, the root by cookie then country, and an
// hreflang Link header on every page. The rules are src/i18n/routing.js (tested); this file only
// talks to Next. Node runtime so the country database can be read from disk.
import { NextResponse, type NextRequest } from 'next/server';
import { decide, alternatesHeader, clientIp, internalReferer, needsCountry } from '@/i18n/routing';
import { countryDb } from '@/i18n/geoip';
import { stripLocale } from '@/i18n/locales';

export const config = {
  runtime: 'nodejs',
  matcher: ['/((?!_next/|api/|ws$|viewing/|rpc$).*)'],
};

/** The canonical origin, for the hreflang alternates. */
const ORIGIN = 'https://randscan.org';
const IS_FILE = /\.[a-z0-9]{2,5}$/i;
/** Set on our own rewrites into the English tree, so a second middleware pass leaves them be. */
const INTERNAL = 'x-randscan-i18n';

// Open the country database when the server starts, so its one log line is there after a
// restart rather than after the first visitor (deploy check).
countryDb();

// Responses to "/" differ by visitor (cookie, country, referer): no shared cache may store them.
// Caddy does not cache and Cloudflare is DNS-only today; keep it so, or key a cache on all three.

export function middleware(req: NextRequest) {
  const { pathname, search } = req.nextUrl;
  if (req.headers.get(INTERNAL)) return NextResponse.next();

  const cookie = req.cookies.get('lang')?.value ?? null;
  const internal = internalReferer(req.headers.get('referer'), req.headers.get('host') ?? '');
  const country =
    needsCountry(pathname, cookie, internal)
      ? (countryDb()?.country(clientIp(req.headers.get('x-forwarded-for')) ?? '') ?? null)
      : null;

  const d = decide({ pathname, search, cookie, country, internal });
  let res: NextResponse;
  if (d.kind === 'redirect') {
    // Built on https://<Host>: with experimental.trustHostHeader Next's own request URL has the
    // same origin, so it sends this as a relative Location. A URL built from req.url would carry
    // the listen address (127.0.0.1:3001) behind Caddy.
    const host = req.headers.get('host') ?? 'localhost';
    res = NextResponse.redirect(new URL(d.to, `https://${host}`), d.status);
  } else if (d.kind === 'rewrite') {
    const headers = new Headers(req.headers);
    headers.set(INTERNAL, '1');
    res = NextResponse.rewrite(new URL(d.to, req.url), { request: { headers } });
  } else {
    res = NextResponse.next();
  }
  if (d.kind !== 'redirect' && !IS_FILE.test(pathname)) {
    res.headers.set('Link', alternatesHeader(ORIGIN, stripLocale(pathname)));
  }
  return res;
}
