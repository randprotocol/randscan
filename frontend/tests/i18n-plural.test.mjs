import { test } from 'node:test';
import assert from 'node:assert/strict';
import { plural } from '../src/i18n/rich.js';

test('plural picks the locale\'s form, falling back to other', () => {
  const en = { one: '{n} block', other: '{n} blocks' };
  const ru = { one: '{n} блок', few: '{n} блока', many: '{n} блоков', other: '{n} блока' };
  assert.equal(plural('en', en, 1), '1 block');
  assert.equal(plural('en', en, 5), '5 blocks');
  assert.equal(plural('ru', ru, 1), '1 блок');
  assert.equal(plural('ru', ru, 3), '3 блока');
  assert.equal(plural('ru', ru, 5), '5 блоков');
  assert.equal(plural('ja', { other: '{n} ブロック' }, 1), '1 ブロック');
  assert.equal(plural('ar', en, 0), '0 blocks');
  assert.equal(plural('en', en, 1234, { n: '1,234' }), '1,234 blocks');
  assert.equal(plural('en', en, 2, { extra: 'x' }), '2 blocks');
});
