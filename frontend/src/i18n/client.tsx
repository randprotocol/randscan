'use client';
// The page's language for client components: the dictionary, a translator, locale-aware
// formatters and a Link that keeps the language prefix.
import Link from 'next/link';
import {
  createContext,
  useCallback,
  useContext,
  useMemo,
  type ComponentProps,
  type ReactNode,
} from 'react';
import type { Messages } from './messages';
import { localeInfo, localizePath } from './locales';
import { fill } from './rich';
import { makeFormat, type Format } from './format';
import { Rich } from './RichText';
import type { TokenInfo } from '@/types';
import { formatTokenAmount } from '@/lib/utils';

interface Ctx {
  locale: string;
  tag: string;
  dir: 'ltr' | 'rtl';
  messages: Messages;
}

const I18nContext = createContext<Ctx | null>(null);

export function I18nProvider({
  locale,
  messages,
  children,
}: {
  locale: string;
  messages: Messages;
  children: ReactNode;
}) {
  const value = useMemo(() => {
    const info = localeInfo(locale);
    return { locale: info.code, tag: info.tag, dir: info.dir, messages };
  }, [locale, messages]);
  return <I18nContext.Provider value={value}>{children}</I18nContext.Provider>;
}

function useCtx(): Ctx {
  const c = useContext(I18nContext);
  if (!c) throw new Error('i18n: used outside I18nProvider');
  return c;
}

export function useLocale() {
  const { locale, tag, dir } = useCtx();
  return { code: locale, tag, dir };
}

const lookup = (m: unknown, key: string): unknown =>
  key
    .split('.')
    .reduce<unknown>(
      (o, k) => (o && typeof o === 'object' ? (o as Record<string, unknown>)[k] : undefined),
      m,
    );

export function useT() {
  const { messages, locale } = useCtx();
  const t = useCallback(
    (key: string, vars?: Record<string, string | number>) => {
      const v = lookup(messages, key);
      if (typeof v !== 'string') {
        if (process.env.NODE_ENV !== 'production') console.warn(`i18n: missing key ${key}`);
        return key;
      }
      return vars ? fill(v, vars) : v;
    },
    [messages],
  );
  const get = useCallback(<T,>(key: string): T => lookup(messages, key) as T, [messages]);
  const rich = useCallback(
    (key: string, vars?: Record<string, string | number>): ReactNode => (
      <Rich text={t(key, vars)} locale={locale} />
    ),
    [t, locale],
  );
  return { t, get, rich, locale };
}

export function useLocalizePath() {
  const { locale } = useCtx();
  return useCallback((path: string) => localizePath(locale, path), [locale]);
}

/** next/link with the page's language prefix applied to a root-relative href. */
export function L({ href, ...rest }: ComponentProps<typeof Link>) {
  const l = useLocalizePath();
  return <Link href={typeof href === 'string' ? l(href) : href} {...rest} />;
}

export type Fmt = Format & {
  tokenAmount(
    units: string | number | bigint | null | undefined,
    index: number | null | undefined,
    tokens: TokenInfo[] | null | undefined,
  ): string;
};

/** Formatters bound to the page's locale. */
export function useFmt(): Fmt {
  const { tag, messages } = useCtx();
  return useMemo(() => {
    const f = makeFormat(tag, messages.fmt);
    return { ...f, tokenAmount: (u, i, tokens) => formatTokenAmount(u, i, tokens, f) };
  }, [tag, messages]);
}
