const PLURAL_FORMS = ['zero', 'one', 'two', 'few', 'many', 'other'];

/** An object whose keys are all CLDR plural categories, with `other` among them. */
export const isPlural = (o) =>
  o !== null &&
  typeof o === 'object' &&
  !Array.isArray(o) &&
  'other' in o &&
  Object.keys(o).every((k) => PLURAL_FORMS.includes(k));

/**
 * Deep merge: the translation over English. Only keys English has survive; a wrong type, an
 * empty string or a missing value falls back to English; arrays merge element-wise.
 * @template T @param {T} base @param {unknown} over @returns {T}
 */
export function merge(base, over) {
  if (over === undefined || over === null) return base;
  if (Array.isArray(base)) {
    if (!Array.isArray(over)) return base;
    return /** @type {T} */ (base.map((item, i) => (i < over.length ? merge(item, over[i]) : item)));
  }
  if (base !== null && typeof base === 'object') {
    if (typeof over !== 'object' || Array.isArray(over)) return base;
    /** @type {Record<string, unknown>} */
    const out = {};
    for (const k of Object.keys(base)) out[k] = merge(base[k], over[k]);
    // A plural message ({ one, other } in English) keeps the forms its language has and English
    // lacks: zero, two, few, many.
    if (isPlural(base)) {
      for (const k of PLURAL_FORMS) {
        if (!(k in out) && typeof over[k] === 'string' && over[k] !== '') out[k] = over[k];
      }
    }
    return /** @type {T} */ (out);
  }
  if (typeof over !== typeof base) return base;
  if (typeof over === 'string' && over === '') return base;
  return /** @type {T} */ (over);
}
