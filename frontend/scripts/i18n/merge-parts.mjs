// Fold the extraction fragments (src/i18n/messages/_parts/*.json) into en.json. A namespace may
// come from one fragment only; a clash is an error, never a silent overwrite.
import { readFileSync, writeFileSync, readdirSync, existsSync } from 'node:fs';
const DIR = new URL('../../src/i18n/messages/', import.meta.url).pathname;
const en = JSON.parse(readFileSync(`${DIR}en.json`, 'utf8'));
const owner = {};
if (!existsSync(`${DIR}_parts`)) process.exit(0);
for (const f of readdirSync(`${DIR}_parts`).filter((f) => f.endsWith('.json')).sort()) {
  const part = JSON.parse(readFileSync(`${DIR}_parts/${f}`, 'utf8'));
  for (const [ns, v] of Object.entries(part)) {
    if (owner[ns]) throw new Error(`namespace ${ns} in both ${owner[ns]} and ${f}`);
    if (ns in en && !['common', 'fmt', 'header', 'footer'].includes(ns) && !owner[ns]) {
      // re-running after a previous merge: the fragment is the source of truth for its namespace
    }
    if (['common', 'fmt', 'header', 'footer'].includes(ns)) throw new Error(`${f} must not define ${ns}`);
    owner[ns] = f;
    en[ns] = v;
  }
}
writeFileSync(`${DIR}en.json`, JSON.stringify(en, null, 2) + '\n');
console.log(`en.json: ${Object.keys(en).length} namespaces (${Object.entries(owner).map(([n, f]) => `${n}<${f}`).join(', ')})`);
