// A dictionary string with <a href>, <code>, <strong>, <em> and <br/> rendered as React elements,
// never through innerHTML. Root-relative links keep the page's language.
import Link from 'next/link';
import { Fragment, type ReactNode } from 'react';
import { tokenize, type Token } from './rich';
import { localizePath } from './locales';

function render(tokens: Token[], locale: string): ReactNode[] {
  return tokens.map((tk, i) => {
    switch (tk.type) {
      case 'text':
        return <Fragment key={i}>{tk.text}</Fragment>;
      case 'br':
        return <br key={i} />;
      case 'code':
        return <code key={i}>{render(tk.children, locale)}</code>;
      case 'strong':
        return <strong key={i}>{render(tk.children, locale)}</strong>;
      case 'em':
        return <em key={i}>{render(tk.children, locale)}</em>;
      case 'a':
        return tk.href.startsWith('/') && !tk.href.startsWith('//') ? (
          <Link key={i} href={localizePath(locale, tk.href)} className="link">
            {render(tk.children, locale)}
          </Link>
        ) : tk.href.startsWith('#') ? (
          <a key={i} href={tk.href} className="link">
            {render(tk.children, locale)}
          </a>
        ) : (
          <a key={i} href={tk.href} className="link" target="_blank" rel="noopener noreferrer">
            {render(tk.children, locale)}
          </a>
        );
    }
  });
}

export function Rich({ text, locale }: { text: string; locale: string }) {
  return <>{render(tokenize(text), locale)}</>;
}
