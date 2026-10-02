export interface FormatWords {
  units: string;
  assetUnits: string;
  bridged: string;
  chain: string;
  unknownLocation: string;
}
export const EN_WORDS: FormatWords;
type Num = string | number | bigint | null | undefined;
type Geo = { city: string | null; region: string | null; country: string | null } | null | undefined;
export interface Format {
  tag: string;
  number(n: number | bigint | null | undefined): string;
  units(value: Num, decimals?: number): string;
  amount(value: Num, decimals?: number, symbol?: string): string;
  stake(value: Num, suffix?: string): string;
  compact(n: number): string;
  percentage(value: number, decimals?: number): string;
  bytes(bytes: number | null | undefined): string;
  binaryBytes(bytes: number | null | undefined): string;
  duration(ms: number): string;
  connected(seconds: number | null | undefined): string;
  mintWindow(secs: number): string;
  ago(ms: number, now?: number): string;
  dateTime(ms: number, timeZone?: string): string;
  bridgeUnits(units: string | null | undefined, symbol?: string | null): string;
  bridged(units: string | null | undefined): string;
  bridgeChain(id: number | null | undefined, name: string | null | undefined): string;
  assetUnits(units: Num, index: number): string;
  location(geo: Geo): string;
}
export function makeFormat(tag: string, words?: Partial<FormatWords>): Format;
