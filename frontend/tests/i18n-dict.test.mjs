import { test } from 'node:test';
import assert from 'node:assert/strict';
import { nOptional, formMarks } from '../scripts/i18n/dict.mjs';

test('{n} may be left out only in a plural form that covers a single number', () => {
  assert.equal(nOptional('ar', 'zero'), true);   // only 0
  assert.equal(nOptional('ar', 'one'), true);    // only 1
  assert.equal(nOptional('ar', 'two'), true);    // only 2
  assert.equal(nOptional('ar', 'few'), false);
  assert.equal(nOptional('ru', 'one'), false);   // 1, 21, 31…
  assert.equal(nOptional('fr', 'one'), false);   // 0 and 1
  assert.equal(nOptional('en', 'one'), true);
  assert.equal(nOptional('en', 'other'), false);
});

test('formMarks drops {n} where it is optional, keeps every other placeholder', () => {
  assert.equal(formMarks('ar', 'zero', 'لا برامج'), formMarks('ar', 'zero', '{n} programs').replace('{n}', '').trim());
  assert.equal(formMarks('ar', 'one', 'برنامج واحد في {where}'), '{where}');
  assert.equal(formMarks('ru', 'one', '{n} блок'), '{n}');
});
