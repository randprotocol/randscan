import { test } from 'node:test';
import assert from 'node:assert/strict';
import { merge } from '../src/i18n/merge.js';

test('translation over English, gaps filled from English', () => {
  const en = { a: 'A', nest: { b: 'B', c: 'C' }, list: ['one', 'two'], n: 1 };
  assert.deepEqual(merge(en, undefined), en);
  assert.deepEqual(merge(en, { a: 'Я' }), { a: 'Я', nest: { b: 'B', c: 'C' }, list: ['one', 'two'], n: 1 });
  assert.deepEqual(merge(en, { nest: { c: 'Ц' } }).nest, { b: 'B', c: 'Ц' });
  assert.deepEqual(merge(en, { list: ['один'] }).list, ['один', 'two']);
  assert.equal(merge(en, { a: '' }).a, 'A');
  assert.equal(merge(en, { a: 5 }).a, 'A');
  assert.equal(merge(en, { nest: 'flat' }).nest.b, 'B');
  assert.equal(merge(en, { extra: 'x' }).extra, undefined);
});

test('a plural object keeps the translation\'s own forms (few, many) that English lacks', () => {
  const en = { n: { one: '{n} block', other: '{n} blocks' } };
  const ru = { n: { one: '{n} блок', few: '{n} блока', many: '{n} блоков', other: '{n} блока' } };
  assert.deepEqual(merge(en, ru).n, ru.n);
  assert.deepEqual(merge(en, { n: { few: 'x' } }).n, { one: '{n} block', other: '{n} blocks', few: 'x' });
  assert.deepEqual(merge(en, { n: { bogus: 'x' } }).n, en.n);
});
