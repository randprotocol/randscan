// The visitor's country from an offline copy of DB-IP Country Lite, read once into memory.
// Nothing is stored and nothing leaves the process (privacy page, "IP address").
import { readFileSync } from 'node:fs';
import { Reader } from 'mmdb-lib';

/** @param {string | undefined} path @returns {{ country: (ip: string) => string | null } | null} */
export function openCountryDb(path) {
  if (!path) return null;
  let reader;
  try {
    reader = new Reader(readFileSync(path));
  } catch {
    return null;
  }
  return {
    country(ip) {
      if (!ip) return null;
      try {
        const r = reader.get(ip);
        const cc = r?.country?.iso_code;
        return typeof cc === 'string' && cc.length === 2 ? cc.toUpperCase() : null;
      } catch {
        return null;
      }
    },
  };
}

let loaded = false;
/** @type {ReturnType<typeof openCountryDb>} */
let db = null;

/** The process-wide database from GEOIP_DB, opened on first use; one log line either way. */
export function countryDb() {
  if (!loaded) {
    loaded = true;
    const path = process.env.GEOIP_DB;
    db = openCountryDb(path);
    console.log(
      db
        ? `i18n: country database loaded from ${path}`
        : 'i18n: no country database (GEOIP_DB unset or unreadable); every first visit is English',
    );
  }
  return db;
}
