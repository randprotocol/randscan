// English unprefixed, fifteen languages prefixed, the root by cookie then country, and an
// hreflang Link header on every page. The rules are src/i18n/routing.js (tested); this file only
// talks to Next. Node runtime so the country database can be read from disk.
import { NextResponse, type NextRequest } from 'next/server';
import { decide, alternatesHeader, clientIp } from '@/i18n/routing';
import { countryDb } from '@/i18n/geoip';
import { stripLocale } from '@/i18n/locales';

export const config = {
  runtime: 'nodejs',
  matcher: ['/((?!_next/|api/|ws$|viewing/|rpc$).*)'],
};

/** The canonical origin, for the hreflang alternates. */
const ORIGIN = 'https://randscan.org';
const IS_FILE = /\.[a-z0-9]{2,5}$/i;

/** Where the request really came in: Caddy keeps Host and sets X-Forwarded-Proto. */
const requestOrigin = (req: NextRequest) => {
  const proto = req.headers.get('x-forwarded-proto') ?? 'http';
  const host = req.headers.get('x-forwarded-host') ?? req.headers.get('host') ?? 'localhost';
  return `${proto}://${host}`;
};

export function middleware(req: NextRequest) {
  const { pathname, search } = req.nextUrl;
  const cookie = req.cookies.get('lang')?.value ?? null;
  const country =
    pathname === '/' && !cookie
      ? (countryDb()?.country(clientIp(req.headers.get('x-forwarded-for')) ?? '') ?? null)
      : null;

  const d = decide({ pathname, search, cookie, country });
  let res: NextResponse;
  if (d.kind === 'redirect') {
    res = NextResponse.redirect(new URL(d.to, requestOrigin(req)), d.status);
  } else if (d.kind === 'rewrite') {
    res = NextResponse.rewrite(new URL(d.to, req.url));
  } else {
    res = NextResponse.next();
  }
  if (d.kind !== 'redirect' && !IS_FILE.test(pathname)) {
    res.headers.set('Link', alternatesHeader(ORIGIN, stripLocale(pathname)));
  }
  return res;
}
