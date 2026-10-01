# randscan.org in sixteen languages

2026-10-02. English stays the default; fifteen more languages are added, and a visitor is sent to
the language of their country on their first arrival at `/`. Every page of the explorer is
translated. Approved by the user in chat on 2026-10-02 after a sectioned design.

## What the user asked for

"Translate randscan.org into Russian, Mandarin, Korean, Cantonese, Indonesian, Malaysian,
Japanese, Arabic, Farsi, Spanish, Portuguese, German, French, Italian, Polish." English is the
default. Detect the visitor's country from their IP: Indonesia gets Indonesian, the United
States gets English. The same instruction was given for randprotocol.org, which another session
is building; the two sites share their codes, their country table and their cookie so a visitor
sees both in one language.

## What the explorer is

A Next.js 15.5 app-router site (`frontend/`): 31 pages under `src/app`, 20 components, about
10,000 lines of TypeScript. Nearly every page is a client component that fetches from the Rust
API with SWR; the terms, privacy and sign-in pages are server components. Prose is concentrated
in `terms`, `privacy`, `bridge`, `transactions/[hash]` and `provers`; the rest is labels,
headings, table columns, empty states and form text. The API's error messages are English and
stay so. The site runs as a standalone Node app on node E behind Caddy; Cloudflare holds the DNS
only (grey cloud), so no Cloudflare country header exists at the origin.

## Decisions

| Question | Decision | Why |
|---|---|---|
| URL scheme | English at `/…`; every other language at `/{code}/…` with the same path | Nothing already published moves; one prefix per language is what `hreflang` expects |
| Codes | `ru zh zh-hk ko id ms ja ar fa es pt de fr it pl`; tags `ru zh-Hans zh-Hant-HK ko id ms ja ar fa es pt de fr it pl` | The same as randprotocol.org. Mandarin is simplified Chinese; Cantonese is traditional Chinese as written in Hong Kong |
| Where the strings live | `frontend/src/i18n/messages/{code}.json`, one file per language, grouped by page; English is the source and the fallback | A missing key renders English instead of breaking the build; a language can be checked on its own |
| Detection | Next middleware reads the client IP Caddy forwards and looks its country up in an offline database held in memory on the box | The privacy page promises visitors are never geolocated by a third party; an in-process lookup that stores nothing and sends nothing keeps that in substance, and the page is amended to say so |
| Database | DB-IP Country Lite (`dbip-country-lite-YYYY-MM.mmdb`, CC BY 4.0, about 4 MB), fetched by the deploy script to `/etc/randscan/geoip/` and refreshed when older than 35 days | Free, no key, no account; the attribution goes on the privacy page |
| When it runs | Only on a request for exactly `/` with no valid `lang` cookie | A deep link stays what it says; a crawler that lands on `/` from the United States sees English |
| Memory | A `lang` cookie, one year, path `/`, set by the switcher; the redirect does not set it | A country is not a choice; a visitor who picks English stays in English on every later visit to `/` |
| Explicit `/en/…` | 308 to the unprefixed path | One URL per English page |
| RTL | Arabic and Farsi get `dir="rtl"` on `<html>`; hashes, addresses, amounts and code are `dir="ltr"` | Flex layouts mirror; data that reads left to right stays so |
| Fonts | None added; per-script system fallbacks behind Inter | Self-hosting six scripts is a later decision |
| Numbers and dates | `Intl` with the page's locale, replacing the hardcoded `en-US` | Separators and month names are part of the language |
| Legal pages | Translated like any page, with a one-line note that the English text governs | A reader can read the terms; the English text binds |
| `hreflang` | A `Link` header on every HTML response listing all sixteen alternates and `x-default`, added by the middleware | Path-aware without a per-page `generateMetadata` on client pages |

## Country table

| Code | Countries (ISO 3166-1 alpha-2) |
|---|---|
| id | ID |
| ms | MY BN |
| ru | RU BY KZ KG |
| zh | CN |
| zh-hk | HK MO TW |
| ko | KR |
| ja | JP |
| ar | SA AE EG QA KW BH OM JO IQ LB MA DZ TN LY SD YE SY PS MR |
| fa | IR AF |
| es | ES MX AR CO CL PE VE EC GT CU BO DO HN PY SV NI CR PA UY PR |
| pt | BR PT AO MZ |
| de | DE AT CH LI |
| fr | FR MC |
| it | IT SM |
| pl | PL |
| en | everything else, including US, GB, SG, CA, AU, IN |

Identical to randprotocol.org's table (its `docs/superpowers/specs/2026-10-02-i18n-design.md`).
Taiwan goes to the traditional-script page. Any change to this table is made on both sites.

## Routing

Every page moves from `src/app/X/page.tsx` to `src/app/[lang]/X/page.tsx`; `src/app/[lang]/layout.tsx`
becomes the root layout (there is no `src/app/layout.tsx` any more) and sets `<html lang dir>`
from the segment. `generateStaticParams` lists the sixteen codes. The metadata files
(`icon.png`, `apple-icon.png`, `opengraph-image.png`) stay at `src/app/` and keep their URLs.

`src/middleware.ts` (Node runtime), in this order, for requests that are not `/api`, `/ws`,
`/_next`, `/viewing` or a file with an extension:

1. `/en` or `/en/…` → 308 to the unprefixed path.
2. `/{code}/…` for a known code → pass through.
3. Exactly `/` with no valid `lang` cookie → country of the client IP (first address of
   `X-Forwarded-For`, which Caddy sets) → the table → if it names a code other than `en`,
   302 to `/{code}/`. No database, or an address it does not hold, means English.
4. Anything else → rewrite (not redirect) to `/en{path}` so the `[lang]` tree serves it.

On every HTML response the middleware adds a `Link` header with sixteen `rel="alternate"
hreflang` entries and `x-default`, all for the request's unprefixed path.

The `/tx/:hash` redirects in `next.config.js` run before the middleware and are unchanged; a
wallet's "open in randscan" link lands on the English transaction page.

## Strings

`src/i18n/locales.js` (plain JavaScript so `node --test` can load it): the locale table
(code, tag, native name, direction), the country table, `localeFromPath`, `stripLocale`,
`localizePath`, `localeForCountry`. `src/i18n/index.ts`: `getMessages(code)` (English deep-merged
under the language), the `I18nProvider` client context, `useLocale()`, `useT()` (`t(key, vars)`
with `{name}` placeholders), `useFmt()` (number, amount, date and relative-time formatters bound
to the locale), and `<L href>` — `next/link` with the locale prefix applied to root-relative
hrefs. Server components call `getMessages` directly.

Keys are grouped by page or component (`header.nav.blocks`, `blocks.title`,
`tx.detail.anchor`…). A table column, a badge, an empty state and a form label are keys like
any other. Strings built from fragments are rewritten as one key with placeholders. The API's
error strings are shown as received.

## Chrome

The header's right group gains one icon button (a globe), opening a menu of the sixteen native
names; the same list sits at the foot of the mobile drawer. Choosing a language sets the cookie
and goes to the same path in that language. Header links, the search box, the footer and the
account links are localized.

## Privacy page

A new paragraph under the IP-address section: on a first visit to the home page the country of
the address is looked up in a copy of DB-IP's country database held on our own server, to pick
a language; the result is not stored, nothing is sent to anyone, and a language chosen from
the menu is kept in a cookie named `lang`. The sentence "Visitors' addresses are never
geolocated" becomes "never sent to a geolocation service". The DB-IP attribution ("IP
Geolocation by DB-IP", CC BY 4.0, linked) is added. The cookie is listed with the others.

## Translation

`frontend/scripts/i18n/translate.mjs` flattens `messages/en.json` to `path → string`, drops the
keys that are never translated (`href`, `url`, `code`, `symbol`, identifiers), sends chunks of
about 1,500 words to `claude -p` with a glossary, checks that every key came back with its
`{placeholders}` and tags intact, and writes the language's file. It skips keys already present
and unchanged in the output, so a re-run after an English edit translates only what changed.
Glossary kept in English: Rand, RAND, zUSD, RPL, Rand Wallet, RandScan, randscan, STARK, FRI,
zkVM, RISC-V, HotStuff, ML-KEM, USDT, USDC, Solana, Tron, Ethereum, Binance Smart Chain,
testnet, mainnet-beta, API, WebSocket, JSON-RPC, and every identifier, field name and RPC method.
Registers: zh simplified Mandarin; zh-hk traditional Chinese, Hong Kong; pt Brazilian-leaning
neutral; ms Malaysian (not Indonesian) wording.

## Verification

- `frontend/tests/i18n.test.mjs` (`node --test`): the path helpers, the country table (ID → id,
  US → en, unknown → en, TW → zh-hk), key parity and placeholder parity of every language
  against English.
- `frontend/scripts/i18n/untranslated.mjs`: fails when a JSX text literal of two or more
  letters remains in `src/app` or `src/components`, outside an allowlist (brand names, units).
- `npm run build` (standalone) and `npm run lint`.
- Locally: the dev server against the live API, each language's home page and one data page
  opened; Arabic and Farsi checked for mirrored layout and left-to-right hashes.
- Live after deploy: `/` from here (English or the local country), `/id/` with the cookie
  forced, `/en/blocks` → 308, the `Link` header on `/ru/blocks`, the middleware's one-line log
  that the database loaded.

## Deploy

`deploy/vps-setup.sh` fetches the database into `/etc/randscan/geoip/` when missing or older
than 35 days (`https://download.db-ip.com/free/dbip-country-lite-YYYY-MM.mmdb.gz`, falling back
to the previous month), and `deploy/randscan-frontend.service` passes `GEOIP_DB` to the app.
Then the usual: commit, push, `deploy/push-to-vps.sh 188.166.235.187`, verify live.

## Out of scope, noted

Translated API error messages; self-hosted CJK and Arabic fonts; per-language link-preview
images; locale-aware `/tx/<hash>` redirects; a sitemap.
