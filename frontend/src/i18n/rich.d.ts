export type Token =
  | { type: 'text'; text: string }
  | { type: 'br' }
  | { type: 'a'; href: string; children: Token[] }
  | { type: 'code' | 'strong' | 'em'; children: Token[] };
export function tokenize(s: string): Token[];
export function fill(s: string, vars?: Record<string, string | number>): string;
