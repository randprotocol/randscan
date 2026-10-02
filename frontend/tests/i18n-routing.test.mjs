import { test } from 'node:test';
import assert from 'node:assert/strict';
import { decide, alternatesHeader, clientIp } from '../src/i18n/routing.js';

const d = (pathname, extra = {}) => decide({ pathname, search: '', cookie: null, country: null, ...extra });

test('/en and /en/… go to the unprefixed path with 308', () => {
  assert.deepEqual(d('/en'), { kind: 'redirect', to: '/', status: 308 });
  assert.deepEqual(d('/en/'), { kind: 'redirect', to: '/', status: 308 });
  assert.deepEqual(d('/en/blocks', { search: '?page=2' }), { kind: 'redirect', to: '/blocks?page=2', status: 308 });
});

test('a known prefix passes through', () => {
  assert.deepEqual(d('/ru/blocks'), { kind: 'next' });
  assert.deepEqual(d('/zh-hk'), { kind: 'next' });
});

test('an unprefixed page is rewritten into the English tree', () => {
  assert.deepEqual(d('/blocks'), { kind: 'rewrite', to: '/en/blocks' });
  assert.deepEqual(d('/idle'), { kind: 'rewrite', to: '/en/idle' });
  assert.deepEqual(d('/search', { search: '?q=5' }), { kind: 'rewrite', to: '/en/search?q=5' });
});

test('the root: country decides only without a valid cookie', () => {
  assert.deepEqual(d('/', { country: 'ID' }), { kind: 'redirect', to: '/id/', status: 302 });
  assert.deepEqual(d('/', { country: 'US' }), { kind: 'rewrite', to: '/en' });
  assert.deepEqual(d('/', { country: null }), { kind: 'rewrite', to: '/en' });
  assert.deepEqual(d('/', { country: 'ID', cookie: 'en' }), { kind: 'rewrite', to: '/en' });
  assert.deepEqual(d('/', { country: 'US', cookie: 'ja' }), { kind: 'redirect', to: '/ja/', status: 302 });
  assert.deepEqual(d('/', { country: 'ID', cookie: 'xx' }), { kind: 'redirect', to: '/id/', status: 302 });
  assert.deepEqual(d('/', { country: 'ID', search: '?utm=1' }), { kind: 'redirect', to: '/id/?utm=1', status: 302 });
  assert.deepEqual(d('/blocks', { country: 'ID' }), { kind: 'rewrite', to: '/en/blocks' });
});

test('non-page paths are left alone', () => {
  assert.deepEqual(d('/api/v1/stats'), { kind: 'next' });
  assert.deepEqual(d('/viewing/randscan_viewing_bg.wasm'), { kind: 'next' });
  assert.deepEqual(d('/icon.png'), { kind: 'next' });
});

test('the alternates header lists sixteen languages and x-default', () => {
  const h = alternatesHeader('https://randscan.org', '/blocks');
  assert.match(h, /<https:\/\/randscan\.org\/blocks>; rel="alternate"; hreflang="en"/);
  assert.match(h, /<https:\/\/randscan\.org\/zh-hk\/blocks>; rel="alternate"; hreflang="zh-Hant-HK"/);
  assert.match(h, /<https:\/\/randscan\.org\/blocks>; rel="alternate"; hreflang="x-default"/);
  assert.equal(h.split(', ').length, 17);
  assert.match(alternatesHeader('https://randscan.org', '/'), /<https:\/\/randscan\.org\/ru\/>; rel="alternate"; hreflang="ru"/);
});

test('clientIp takes the first forwarded address', () => {
  assert.equal(clientIp('1.2.3.4, 10.0.0.1'), '1.2.3.4');
  assert.equal(clientIp(' 2001:db8::1 '), '2001:db8::1');
  assert.equal(clientIp(null), null);
  assert.equal(clientIp(''), null);
});
