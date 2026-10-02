// The privacy page must describe what the middleware does (src/i18n/routing.js needsCountry).
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { flat } from '../scripts/i18n/dict.mjs';

const en = flat(JSON.parse(readFileSync(new URL('../src/i18n/messages/en.json', import.meta.url), 'utf8')));
const text = Object.values(en).filter((v) => typeof v === 'string').join('\n');

test('the privacy text describes when the country is looked up', () => {
  const p = Object.values(en).find((v) => typeof v === 'string' && v.includes('DB-IP’s country database'));
  assert.ok(p, 'the country-lookup paragraph exists');
  assert.doesNotMatch(p, /only then/, 'the lookup is not limited to a first visit');
  assert.match(p, /from another site|typing the address/, 'says the lookup happens on arrival from outside');
  assert.match(p, /link within the site/, 'says a click within the site never switches language');
  assert.match(p, /not stored/);
  assert.match(text, /never sent to a geolocation service/);
});
