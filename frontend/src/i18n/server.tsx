// The page's language for server components (privacy, terms and the sign-in pages): the same
// t / tp / rich / l a client component gets from useT(), from the [lang] route parameter.
import type { ReactNode } from 'react';
import { getMessages, pick, pickAny } from './messages';
import { isLocale, localeInfo, localizePath } from './locales';
import { plural } from './rich';
import { Rich } from './RichText';
import { makeFormat } from './format';

export async function serverT(params: Promise<{ lang: string }>) {
  const { lang: raw } = await params;
  const lang = isLocale(raw) ? raw : 'en';
  const m = getMessages(lang);
  const { tag, dir } = localeInfo(lang);
  const t = (key: string, vars?: Record<string, string | number>) => pick(m, key, vars);
  return {
    lang,
    tag,
    dir,
    t,
    get: <T,>(key: string) => pickAny<T>(m, key),
    tp: (key: string, n: number, vars?: Record<string, string | number>) =>
      plural(tag, pickAny<Record<string, string>>(m, key) ?? {}, n, {
        n: new Intl.NumberFormat(`${tag}-u-nu-latn`).format(n),
        ...vars,
      }),
    rich: (key: string, vars?: Record<string, string | number>): ReactNode => (
      <Rich text={t(key, vars)} locale={lang} />
    ),
    /** A root-relative path in this page's language. */
    l: (path: string) => localizePath(lang, path),
    fmt: makeFormat(tag, m.fmt),
  };
}
