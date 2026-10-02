// Translate src/i18n/messages/en.json into every other language with `claude -p`: ~1,200 words a
// call, several calls in parallel, {placeholders}, tags, identifiers and the glossary kept intact,
// plural messages given exactly the forms the language uses. Re-runnable: a key whose English has
// not changed since it was translated (src/i18n/messages/.source/{code}.json) is kept. One file per
// language, so several languages can run at once.
//
//   node scripts/i18n/translate.mjs              # every language, changed keys only
//   node scripts/i18n/translate.mjs ru ja        # some languages
//   FORCE=1 node scripts/i18n/translate.mjs ar   # everything again for Arabic
//   MODEL=opus PARALLEL=4 node scripts/i18n/translate.mjs
import { readFileSync, writeFileSync, existsSync, mkdirSync } from 'node:fs';
import { spawn } from 'node:child_process';
import { createHash } from 'node:crypto';
import { LOCALES, localeInfo } from '../../src/i18n/locales.js';
import { flat, unflat, marks, formMarks, NEVER, isPlural, pluralCategories } from './dict.mjs';

const ROOT = new URL('../../', import.meta.url).pathname;
const MSG = `${ROOT}src/i18n/messages/`;
const STAMPS = `${MSG}.source/`;
const GLOSSARY = readFileSync(new URL('./glossary.txt', import.meta.url), 'utf8').trim().split('\n').join(', ');
const PARALLEL = Number(process.env.PARALLEL || 6);
const MODEL = process.env.MODEL || 'sonnet';
const WORDS = Number(process.env.WORDS || 1200);

const LANGUAGE = {
  ru: 'Russian',
  zh: 'Simplified Chinese (Mandarin, mainland conventions)',
  'zh-hk': 'Traditional Chinese as written in Hong Kong (standard written Chinese with Hong Kong vocabulary, not colloquial Cantonese)',
  ko: 'Korean',
  id: 'Indonesian',
  ms: 'Malay as used in Malaysia (Bahasa Melayu; Malaysian vocabulary, not Indonesian)',
  ja: 'Japanese',
  ar: 'Modern Standard Arabic',
  fa: 'Persian (Farsi, Iran)',
  es: 'Spanish (neutral international Spanish)',
  pt: 'Portuguese (neutral, Brazilian-leaning)',
  de: 'German',
  fr: 'French',
  it: 'Italian',
  pl: 'Polish',
};

const sha = (v) => createHash('sha1').update(JSON.stringify(v)).digest('hex');
const words = (v) => (typeof v === 'string' ? v : Object.values(v).join(' ')).split(/\s+/).length;

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

/** claude -p, retried with backoff (a busy or rate-limited CLI exits 1 with nothing on stderr). */
async function claudeRetry(prompt) {
  for (let i = 0; ; i++) {
    try {
      return await claude(prompt);
    } catch (e) {
      if (i >= 4) throw e;
      await sleep(15_000 * 2 ** i);
    }
  }
}

function claude(prompt) {
  return new Promise((resolve, reject) => {
    const p = spawn('claude', ['-p', '--model', MODEL, '--output-format', 'text'], { stdio: ['pipe', 'pipe', 'pipe'] });
    let out = '';
    let err = '';
    p.stdout.on('data', (d) => (out += d));
    p.stderr.on('data', (d) => (err += d));
    p.on('close', (code) => (code === 0 ? resolve(out) : reject(new Error(`claude exited ${code}: ${err.slice(0, 300)}`))));
    p.stdin.end(prompt);
  });
}

function chunks(entries) {
  const out = [];
  let cur = [];
  let n = 0;
  for (const e of entries) {
    const w = words(e[1]);
    if (n + w > WORDS && cur.length) {
      out.push(cur);
      cur = [];
      n = 0;
    }
    cur.push(e);
    n += w;
  }
  if (cur.length) out.push(cur);
  return out;
}

/** Why a translated value is not acceptable, or null. */
function problem(code, source, value) {
  if (isPlural(source)) {
    if (!isPlural(value)) return 'not a plural object';
    const need = pluralCategories(localeInfo(code).tag);
    const missing = need.filter((c) => typeof value[c] !== 'string' || value[c].trim() === '');
    if (missing.length) return `plural forms missing: ${missing.join(',')}`;
    const tag = localeInfo(code).tag;
    for (const c of need)
      if (formMarks(tag, c, value[c]) !== formMarks(tag, c, source.other)) return `plural form ${c} changed placeholders`;
    return null;
  }
  if (typeof value !== 'string' || value.trim() === '') return 'empty';
  if (marks(value) !== marks(source)) return `placeholders/tags differ: ${marks(source)} vs ${marks(value)}`;
  return null;
}

async function translateChunk(code, entries, attempt = 1) {
  const tag = localeInfo(code).tag;
  const cats = pluralCategories(tag);
  const prompt = `You are translating the user interface of RandScan, the block explorer of the Rand Protocol (a privacy-preserving, post-quantum blockchain with shielded notes, zero-knowledge proofs and a bridge for USD stablecoins), into ${LANGUAGE[code]}.

Return ONLY a JSON object with exactly the same keys as the input, each mapped to its translation. No commentary, no code fences.

Rules:
- Keep every {placeholder} exactly as written (same name, same braces), and keep every <a href="...">…</a>, <code>…</code>, <strong>…</strong>, <em>…</em> and <br/> tag; translate the visible words inside <a>, <strong> and <em>, but never the href and never what is inside <code>.
- A value starting with "- " is a bullet item: keep the leading "- ".
- Keep these terms in English exactly as written: ${GLOSSARY}. Also keep untranslated every identifier, field name, RPC method (rand_*), CLI command or flag, file name, hex string, number, unit (KB, MB, MiB) and URL.
- A value that is an object of plural forms (keys among zero/one/two/few/many/other) must come back as an object with exactly these keys for ${LANGUAGE[code]}: ${cats.join(', ')}. Each form is the full phrase for that count, using {n} where the English uses it.
- The output must be valid JSON: escape any double quote inside a value as \\" (or use the language's own quotation marks).
- Write as a careful native technical writer for a blockchain explorer would: concise, plain, consistent terminology across keys (the same English term always gets the same translation). UI labels stay short. Do not add explanations or change meaning. Use the language's own punctuation and quotation marks.
- Legal text (keys under privacy.* and terms.*) must be translated faithfully and completely, sentence by sentence, without summarising.

Input JSON:
${JSON.stringify(Object.fromEntries(entries), null, 1)}`;
  let out = null;
  try {
    const text = await claudeRetry(prompt);
    const m = text.match(/\{[\s\S]*\}/);
    out = JSON.parse(m ? m[0] : text);
  } catch (e) {
    console.warn(`  ${code}: ${entries.length} keys: ${String(e.message).slice(0, 120)}`);
  }
  // Unparseable output from a long chunk: halve it rather than resend the same thing.
  if (!out && entries.length > 4 && attempt < 4) {
    const mid = Math.ceil(entries.length / 2);
    const [a, b] = await Promise.all([
      translateChunk(code, entries.slice(0, mid), attempt + 1),
      translateChunk(code, entries.slice(mid), attempt + 1),
    ]);
    return { ...a, ...b };
  }
  const bad = entries.filter(([k, v]) => !out || problem(code, v, out[k]) !== null);
  if (bad.length && attempt < 4) {
    const reasons = bad.slice(0, 3).map(([k, v]) => `${k}: ${out ? problem(code, v, out[k]) : 'no JSON'}`).join('; ');
    console.warn(`  ${code}: ${bad.length}/${entries.length} keys failed checks (${reasons}); retry ${attempt + 1}`);
    const again = await translateChunk(code, bad, attempt + 1);
    return { ...(out ?? {}), ...again };
  }
  if (bad.length) {
    console.error(`  ${code}: gave up on ${bad.map((e) => e[0]).join(', ')}`);
    for (const [k] of bad) if (out) delete out[k];
  }
  return out ?? {};
}

async function pool(items, n, fn) {
  const queue = items.map((it, i) => [it, i]);
  const res = new Array(items.length);
  await Promise.all(
    Array.from({ length: Math.min(n, items.length) }, async () => {
      while (queue.length) {
        const [it, i] = queue.shift();
        res[i] = await fn(it);
      }
    }),
  );
  return res;
}

const en = flat(JSON.parse(readFileSync(`${MSG}en.json`, 'utf8')));
const args = process.argv.slice(2);
const wanted = args.length ? args : LOCALES.map((l) => l.code).filter((c) => c !== 'en');

for (const code of wanted) {
  if (!LANGUAGE[code]) throw new Error(`unknown language ${code}`);
  const started = Date.now();
  const file = `${MSG}${code}.json`;
  const existing = existsSync(file) ? flat(JSON.parse(readFileSync(file, 'utf8'))) : {};
  const stampFile = `${STAMPS}${code}.json`;
  const st = existsSync(stampFile) ? JSON.parse(readFileSync(stampFile, 'utf8')) : {};
  const translatable = ([k, v]) =>
    !NEVER.test(k) && (isPlural(v) || (typeof v === 'string' && /\p{L}/u.test(v)));
  const todo = Object.entries(en).filter(
    (e) => translatable(e) && (process.env.FORCE || existing[e[0]] === undefined || st[e[0]] !== sha(e[1]) || problem(code, e[1], existing[e[0]]) !== null),
  );
  console.log(`${code}: ${todo.length} of ${Object.keys(en).length} keys to translate`);
  const results = await pool(chunks(todo), PARALLEL, (c) => translateChunk(code, c));
  const merged = {};
  for (const [k, v] of Object.entries(en)) {
    if (!translatable([k, v])) merged[k] = v;
    else if (existing[k] !== undefined && problem(code, v, existing[k]) === null) merged[k] = existing[k];
  }
  for (const r of results)
    for (const [k, v] of Object.entries(r)) {
      if (!(k in en)) continue;
      merged[k] = v;
      st[k] = sha(en[k]);
    }
  for (const k of Object.keys(st)) if (!(k in en)) delete st[k];
  const ordered = Object.fromEntries(Object.keys(en).filter((k) => k in merged).map((k) => [k, merged[k]]));
  writeFileSync(file, JSON.stringify(unflat(ordered), null, 2) + '\n');
  mkdirSync(STAMPS, { recursive: true });
  writeFileSync(stampFile, JSON.stringify(st, null, 1) + '\n');
  const missing = Object.keys(en).filter((k) => !(k in merged)).length;
  console.log(`${code}: written in ${Math.round((Date.now() - started) / 1000)} s${missing ? `, ${missing} keys still English` : ''}`);
}
