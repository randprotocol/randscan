import { test } from 'node:test';
import assert from 'node:assert/strict';
import {
  DEFAULT_LOCALE, LOCALES, LOCALE_CODES, isLocale, localeInfo,
  localeFromPath, stripLocale, localizePath, localeForCountry,
} from '../src/i18n/locales.js';

test('sixteen locales in the agreed order', () => {
  assert.deepEqual(LOCALE_CODES, ['en','ru','zh','zh-hk','ko','id','ms','ja','ar','fa','es','pt','de','fr','it','pl']);
  assert.equal(DEFAULT_LOCALE, 'en');
  assert.equal(localeInfo('zh-hk').tag, 'zh-Hant-HK');
  assert.equal(localeInfo('ar').dir, 'rtl');
  assert.equal(localeInfo('fa').dir, 'rtl');
  assert.equal(localeInfo('nope').code, 'en');
  assert.equal(LOCALES.filter((l) => l.dir === 'rtl').length, 2);
});

test('localeFromPath reads only a whole first segment', () => {
  assert.equal(localeFromPath('/'), 'en');
  assert.equal(localeFromPath('/blocks'), 'en');
  assert.equal(localeFromPath('/id'), 'id');
  assert.equal(localeFromPath('/id/'), 'id');
  assert.equal(localeFromPath('/id/blocks/5'), 'id');
  assert.equal(localeFromPath('/idle'), 'en');
  assert.equal(localeFromPath('/zh-hk/tokens'), 'zh-hk');
  assert.equal(localeFromPath('/en/blocks'), 'en');
  assert.equal(localeFromPath('/ru?x=1'), 'ru');
});

test('stripLocale and localizePath round-trip', () => {
  assert.equal(stripLocale('/ru/blocks/5'), '/blocks/5');
  assert.equal(stripLocale('/ru'), '/');
  assert.equal(stripLocale('/ru/'), '/');
  assert.equal(stripLocale('/ru?x=1'), '/?x=1');
  assert.equal(stripLocale('/blocks'), '/blocks');
  assert.equal(stripLocale('/'), '/');
  assert.equal(localizePath('en', '/blocks'), '/blocks');
  assert.equal(localizePath('en', '/ru/blocks'), '/blocks');
  assert.equal(localizePath('ru', '/blocks'), '/ru/blocks');
  assert.equal(localizePath('ru', '/'), '/ru');
  assert.equal(localizePath('ru', '/zh/blocks'), '/ru/blocks');
  assert.equal(localizePath('ru', '/api/v1/stats'), '/api/v1/stats');
  assert.equal(localizePath('ru', '/viewing/randscan_viewing.js'), '/viewing/randscan_viewing.js');
  assert.equal(localizePath('ru', '/viewing'), '/ru/viewing');
  assert.equal(localizePath('ru', '/viewing?key=1'), '/ru/viewing?key=1');
  assert.equal(localizePath('ru', 'https://randprotocol.org'), 'https://randprotocol.org');
  assert.equal(localizePath('ru', '/search?q=5'), '/ru/search?q=5');
  assert.equal(localizePath('ru', '/?q=5'), '/ru?q=5');
  assert.equal(localizePath('ru', '#top'), '#top');
  assert.equal(localizePath('ru', '/icon.png'), '/icon.png');
});

test('country table', () => {
  assert.equal(localeForCountry('ID'), 'id');
  assert.equal(localeForCountry('US'), 'en');
  assert.equal(localeForCountry('TW'), 'zh-hk');
  assert.equal(localeForCountry('CN'), 'zh');
  assert.equal(localeForCountry('BR'), 'pt');
  assert.equal(localeForCountry('CH'), 'de');
  assert.equal(localeForCountry('SG'), 'en');
  assert.equal(localeForCountry('XX'), 'en');
  assert.equal(localeForCountry(null), 'en');
  assert.equal(localeForCountry('id'), 'id');
});

test('isLocale', () => {
  assert.equal(isLocale('ms'), true);
  assert.equal(isLocale('xx'), false);
  assert.equal(isLocale(undefined), false);
});
