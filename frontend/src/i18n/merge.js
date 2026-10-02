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
    return /** @type {T} */ (out);
  }
  if (typeof over !== typeof base) return base;
  if (typeof over === 'string' && over === '') return base;
  return /** @type {T} */ (over);
}
