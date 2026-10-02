export interface LocaleInfo {
  code: string;
  tag: string;
  name: string;
  dir: 'ltr' | 'rtl';
}
export const DEFAULT_LOCALE: 'en';
export const LOCALES: LocaleInfo[];
export const LOCALE_CODES: string[];
export const COUNTRY_TO_LOCALE: Record<string, string>;
export const UNPREFIXED: RegExp;
export function isLocale(code: unknown): code is string;
export function localeInfo(code: string): LocaleInfo;
export function localeForCountry(cc: string | null | undefined): string;
export function localeFromPath(pathname: string): string;
export function stripLocale(pathname: string): string;
export function localizePath(locale: string, path: string): string;
