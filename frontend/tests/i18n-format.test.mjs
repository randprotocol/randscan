import { test } from 'node:test';
import assert from 'node:assert/strict';
import { makeFormat, EN_WORDS } from '../src/i18n/format.js';

const en = makeFormat('en');
const de = makeFormat('de', { ...EN_WORDS, units: 'Einheiten', chain: 'Chain {id}' });
const ar = makeFormat('ar');
const ru = makeFormat('ru');

test('English output is what the explorer printed before', () => {
  assert.equal(en.number(1234567), '1,234,567');
  assert.equal(en.number(null), '—');
  assert.equal(en.units('1500000000', 9), '1.5');
  assert.equal(en.units('1234567000000000', 9), '1,234,567');
  assert.equal(en.units('-2500000000', 9), '-2.5');
  assert.equal(en.amount('1000000000'), '1 RAND');
  assert.equal(en.stake('1000000000000', 'total stake'), '1,000 RAND total stake');
  assert.equal(en.compact(999), '999');
  assert.equal(en.compact(1234), '1.2K');
  assert.equal(en.compact(2500000), '2.5M');
  assert.equal(en.percentage(12.3456), '12.35%');
  assert.equal(en.bytes(512), '512 B');
  assert.equal(en.bytes(2048), '2.00 KB');
  assert.equal(en.duration(500), '500ms');
  assert.equal(en.duration(1200), '1.20s');
  assert.equal(en.duration(125000), '2m 5s');
  assert.equal(en.connected(30), '30s');
  assert.equal(en.connected(11520), '3h 12m');
  assert.equal(en.connected(187200), '2d 4h');
  assert.equal(en.mintWindow(86400), '24 hours');
  assert.equal(en.mintWindow(60), '1 minute');
  assert.equal(en.mintWindow(45), '45 seconds');
  assert.equal(en.bridgeChain(2, 'Ethereum'), 'Ethereum (2)');
  assert.equal(en.bridgeChain(9, null), 'chain 9');
  assert.equal(en.bridgeUnits('125050000000', 'USDT'), '1,250.5 USDT');
  assert.equal(en.bridgeUnits('100000000', null), '1 units');
  assert.equal(en.bridged('150000000'), '1.5 tokens (150,000,000 units)');
  assert.equal(en.assetUnits('42', 3), '42 units of asset #3');
  assert.equal(en.location(null), 'Unknown location');
  assert.equal(en.dateTime(1759300000000, 'UTC'), 'Oct 1, 2025, 6:26:40 AM');
});

test('relative time: English strings, Intl elsewhere', () => {
  const now = 1_000_000_000_000;
  assert.equal(en.ago(now - 12_000, now), '12s ago');
  assert.equal(en.ago(now - 500, now), 'just now');
  assert.equal(en.ago(now - 60_000, now), '1 min ago');
  assert.equal(en.ago(now - 3 * 3_600_000, now), '3 hours ago');
  assert.equal(ru.ago(now - 12_000, now), '12 секунд назад');
  assert.match(ru.ago(now - 8 * 86_400_000, now), /\d{4}/);
});

test('other locales group and separate their way, always in Latin digits', () => {
  assert.equal(de.number(1234567), '1.234.567');
  assert.equal(de.units('1234567500000000', 9), '1.234.567,5');
  assert.equal(de.bridgeUnits('100000000', null), '1 Einheiten');
  assert.equal(de.bridgeChain(9, null), 'Chain 9');
  assert.equal(ar.number(1234567), '1,234,567');
  assert.doesNotMatch(ar.units('1500000000', 9), /[٠-٩]/);
  assert.match(ru.number(1234567), /^1\s234\s567$/);
  assert.equal(de.mintWindow(86400), '24 Stunden');
});
