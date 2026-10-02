export interface CountryDb {
  country(ip: string): string | null;
}
export function openCountryDb(path: string | undefined): CountryDb | null;
export function countryDb(): CountryDb | null;
