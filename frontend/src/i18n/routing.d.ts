export type Decision =
  | { kind: 'next' }
  | { kind: 'redirect'; to: string; status: 302 | 308 }
  | { kind: 'rewrite'; to: string };
export function decide(req: {
  pathname: string;
  search: string;
  cookie: string | null | undefined;
  country: string | null | undefined;
}): Decision;
export function alternatesHeader(origin: string, barePath: string): string;
export function clientIp(xff: string | null | undefined): string | null;
