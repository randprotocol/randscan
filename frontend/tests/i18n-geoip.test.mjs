import { test } from 'node:test';
import assert from 'node:assert/strict';
import { existsSync } from 'node:fs';
import { openCountryDb } from '../src/i18n/geoip.js';

const path = new URL('../geoip/dbip-country-lite.mmdb', import.meta.url).pathname;

test('a missing file opens as null without throwing', () => {
  assert.equal(openCountryDb('/nonexistent.mmdb'), null);
  assert.equal(openCountryDb(''), null);
  assert.equal(openCountryDb(undefined), null);
});

test('known addresses resolve; garbage does not', { skip: !existsSync(path) }, () => {
  const db = openCountryDb(path);
  assert.ok(db);
  assert.equal(db.country('8.8.8.8'), 'US');
  assert.equal(db.country('not an ip'), null);
  assert.equal(db.country(''), null);
  assert.equal(db.country('127.0.0.1'), null);
});
