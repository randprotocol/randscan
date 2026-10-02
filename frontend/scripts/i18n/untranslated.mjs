// Fail when user-visible English is still hard-coded in src/app or src/components: JSX text of
// two or more letters, and title= / placeholder= / aria-label= / alt= literals. Part of `npm run lint`.
import { readFileSync, readdirSync, statSync } from 'node:fs';
import { join, relative } from 'node:path';

const SRC = new URL('../../src/', import.meta.url).pathname;
// Words that are the same in every language: brands, protocol names, units, symbols.
const ALLOW = new Set(
  readFileSync(new URL('./glossary.txt', import.meta.url), 'utf8')
    .trim()
    .split('\n')
    .concat(['rand', 'randprotocol.org', 'B', 'KB', 'MB', 'GB', 'KiB', 'MiB', 'GiB', 'ms', 'px', 'OK', 'ID', 'IP', 'URL', 'Esri', 'Leaflet', 'OpenStreetMap']),
);
const files = [];
const walk = (d) => {
  for (const n of readdirSync(d)) {
    const p = join(d, n);
    if (statSync(p).isDirectory()) walk(p);
    else if (/\.tsx$/.test(p)) files.push(p);
  }
};
walk(join(SRC, 'app'));
walk(join(SRC, 'components'));

const ok = (text) => {
  const t = text.trim().replace(/&[a-z]+;|&#\d+;/g, ' ').trim();
  if (!/[A-Za-z]{2,}/.test(t)) return true;
  if (ALLOW.has(t)) return true;
  // every word an allowed token, a number or punctuation ("RAND", "zUSD / USDT")
  return t.split(/[\s/·,()–—:+×-]+/).filter(Boolean).every((w) => ALLOW.has(w) || !/[A-Za-z]{2,}/.test(w));
};

let bad = 0;
for (const f of files) {
  const src = readFileSync(f, 'utf8')
    .replace(/\/\*[\s\S]*?\*\//g, (m) => m.replace(/[^\n]/g, ' '))
    .replace(/(^|[^:"'`])\/\/.*$/gm, '$1');
  const lines = src.split('\n');
  lines.forEach((line, i) => {
    for (const m of line.matchAll(/>([^<>{}`]*)</g)) {
      if (!ok(m[1])) {
        console.log(`${relative(SRC, f)}:${i + 1}: text "${m[1].trim()}"`);
        bad++;
      }
    }
    for (const m of line.matchAll(/\b(title|placeholder|aria-label|alt|label)=["']([^"']*)["']/g)) {
      if (!ok(m[2])) {
        console.log(`${relative(SRC, f)}:${i + 1}: ${m[1]}="${m[2]}"`);
        bad++;
      }
    }
  });
}
if (bad) {
  console.error(`i18n: ${bad} hard-coded strings in src/app and src/components`);
  process.exit(1);
}
console.log('i18n: no hard-coded English in src/app and src/components');
