import { test } from 'node:test';
import assert from 'node:assert/strict';
import { tokenize, fill } from '../src/i18n/rich.js';

test('fill replaces placeholders and leaves unknown ones', () => {
  assert.equal(fill('{n} blocks on chain {chain}', { n: '5', chain: 20 }), '5 blocks on chain 20');
  assert.equal(fill('{x}', {}), '{x}');
});

test('tokenize splits text, links, code and emphasis', () => {
  assert.deepEqual(tokenize('See <a href="/terms">the terms</a> and <code>rand_getHead</code>.'), [
    { type: 'text', text: 'See ' },
    { type: 'a', href: '/terms', children: [{ type: 'text', text: 'the terms' }] },
    { type: 'text', text: ' and ' },
    { type: 'code', children: [{ type: 'text', text: 'rand_getHead' }] },
    { type: 'text', text: '.' },
  ]);
  assert.deepEqual(tokenize('<strong>bold</strong><em>it</em><br/>x'), [
    { type: 'strong', children: [{ type: 'text', text: 'bold' }] },
    { type: 'em', children: [{ type: 'text', text: 'it' }] },
    { type: 'br' },
    { type: 'text', text: 'x' },
  ]);
  assert.deepEqual(tokenize('plain'), [{ type: 'text', text: 'plain' }]);
  assert.deepEqual(tokenize('a <b> b'), [{ type: 'text', text: 'a <b> b' }]);
  assert.deepEqual(tokenize('x</a>y'), [{ type: 'text', text: 'x' }, { type: 'text', text: 'y' }]);
});
