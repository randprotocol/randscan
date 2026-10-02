// Shared by the translate script and the parity test: flatten a dictionary to `path -> value`,
// treating a plural message ({ one, other, few… }) as one value, and build it back.
const PLURAL_FORMS = ['zero', 'one', 'two', 'few', 'many', 'other'];

export const isPlural = (o) =>
  o !== null &&
  typeof o === 'object' &&
  !Array.isArray(o) &&
  'other' in o &&
  Object.keys(o).every((k) => PLURAL_FORMS.includes(k));

export function flat(o, prefix = '', out = {}) {
  for (const [k, v] of Object.entries(o)) {
    const key = prefix ? `${prefix}.${k}` : k;
    if (v !== null && typeof v === 'object' && !isPlural(v)) flat(v, key, out);
    else out[key] = v;
  }
  return out;
}

export function unflat(f) {
  const o = {};
  for (const [k, v] of Object.entries(f)) {
    const parts = k.split('.');
    let c = o;
    for (let i = 0; i < parts.length - 1; i++) {
      const p = parts[i];
      if (c[p] === undefined) c[p] = /^\d+$/.test(parts[i + 1]) ? [] : {};
      c = c[p];
    }
    c[parts.at(-1)] = v;
  }
  return o;
}

/** The {placeholders} and markup tags of a string, sorted, for comparing a translation with its source. */
export const marks = (s) =>
  [...String(s).matchAll(/\{(\w+)\}|<(a|code|strong|em)\b|<br|^- /g)]
    .map((m) => m[0].replace(/\s.*$/, ''))
    .sort()
    .join(' ');

/** Keys whose values are identifiers, never translated. */
export const NEVER = /(^|\.)(id|href|url|slug|code|symbol|hash|method|example|cmd|curl)$/;

export const pluralCategories = (tag) => new Intl.PluralRules(tag).resolvedOptions().pluralCategories;

/**
 * Whether a plural form may leave out {n}: only when the form covers exactly one number (Arabic
 * "zero"/"one"/"two", English "one"), so the words alone say how many. Russian "one" also covers
 * 21, 31…, so its {n} must stay.
 */
export function nOptional(tag, category) {
  const rules = new Intl.PluralRules(tag);
  let hits = 0;
  for (let i = 0; i <= 1000 && hits < 2; i++) if (rules.select(i) === category) hits++;
  return hits === 1;
}

/** The marks of one plural form, without {n} where the form may leave it out. */
export function formMarks(tag, category, s) {
  const m = marks(s);
  return nOptional(tag, category) ? m.split(' ').filter((x) => x !== '{n}').join(' ') : m;
}
