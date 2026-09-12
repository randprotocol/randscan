'use client';

import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { useState } from 'react';
import { SearchBar } from './SearchBar';
import { ThemeToggle } from './ThemeToggle';
import { cn } from '@/lib/utils';
import { useMe } from '@/hooks/useApi';

const navLinks = [
  { href: '/', label: 'Dashboard' },
  { href: '/blocks', label: 'Blocks' },
  { href: '/transactions', label: 'Transactions' },
  { href: '/notes', label: 'Notes' },
  { href: '/viewing', label: 'History' },
  { href: '/validators', label: 'Validators' },
  { href: '/programs', label: 'Programs' },
  { href: '/bridge', label: 'Bridge' },
  { href: '/nodes', label: 'Nodes' },
];

function AccountLink({
  signedIn,
  className,
  onClick,
}: {
  signedIn: boolean;
  className?: string;
  onClick?: () => void;
}) {
  return (
    <Link href={signedIn ? '/dashboard' : '/login'} className={className} onClick={onClick}>
      {signedIn ? 'API keys' : 'Sign in'}
    </Link>
  );
}

export function Header() {
  const pathname = usePathname();
  const [mobileOpen, setMobileOpen] = useState(false);
  const { data: me } = useMe();

  const isActive = (href: string) =>
    href === '/' ? pathname === '/' : pathname.startsWith(href);

  return (
    <header className="border-b border-border bg-bg">
      <div className="mx-auto max-w-6xl px-6 lg:px-8">
        <div className="flex h-16 items-center justify-between gap-6">
          <Link href="/" className="flex flex-shrink-0 items-center gap-2.5">
            <span
              className="flex h-7 w-7 items-center justify-center rounded font-serif text-sm leading-none text-bg"
              style={{ background: 'var(--color-text-strong)' }}
              aria-hidden="true"
            >
              R
            </span>
            <span className="font-serif text-lg text-strong">RandScan</span>
          </Link>

          <nav className="hidden items-center gap-6 lg:flex">
            {navLinks.map((link) => {
              const active = isActive(link.href);
              return (
                <Link
                  key={link.href}
                  href={link.href}
                  className={cn(
                    'relative py-1 text-sm transition-colors hover:text-strong',
                    active ? 'text-strong' : 'text-soft'
                  )}
                >
                  {link.label}
                  {active && (
                    <span
                      className="absolute -bottom-px left-0 h-0.5 w-full"
                      style={{ background: 'var(--color-text-strong)' }}
                    />
                  )}
                </Link>
              );
            })}
          </nav>

          <div className="ml-auto hidden items-center gap-3 lg:flex">
            <div className="w-56">
              <SearchBar />
            </div>
            <AccountLink signedIn={!!me} className="text-sm text-soft transition-colors hover:text-strong" />
            <a
              href="https://github.com/randprotocol"
              target="_blank"
              rel="noopener noreferrer"
              className="inline-flex items-center gap-1.5 text-sm text-soft transition-colors hover:text-strong"
            >
              <svg className="h-4 w-4" viewBox="0 0 16 16" fill="currentColor" aria-hidden="true">
                <path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27s1.36.09 2 .27c1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.01 8.01 0 0 0 16 8c0-4.42-3.58-8-8-8z" />
              </svg>
              GitHub
            </a>
            <ThemeToggle />
          </div>

          <div className="ml-auto flex items-center gap-2 lg:hidden">
            <ThemeToggle />
            <button
              type="button"
              onClick={() => setMobileOpen((open) => !open)}
              aria-expanded={mobileOpen}
              className="btn-icon"
            >
              <span className="sr-only">Toggle navigation</span>
              <svg className="h-4 w-4" fill="none" viewBox="0 0 24 24" strokeWidth={1.6} stroke="currentColor">
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  d={mobileOpen ? 'M6 18L18 6M6 6l12 12' : 'M3.75 6.75h16.5M3.75 12h16.5m-16.5 5.25h16.5'}
                />
              </svg>
            </button>
          </div>
        </div>

        {mobileOpen && (
          <nav className="flex flex-col border-t border-border-soft py-2 lg:hidden">
            {navLinks.map((link) => (
              <Link
                key={link.href}
                href={link.href}
                onClick={() => setMobileOpen(false)}
                className={cn(
                  'px-1 py-2 text-sm transition-colors hover:text-strong',
                  isActive(link.href) ? 'text-strong' : 'text-soft'
                )}
              >
                {link.label}
              </Link>
            ))}
            <AccountLink
              signedIn={!!me}
              className="px-1 py-2 text-sm text-soft transition-colors hover:text-strong"
              onClick={() => setMobileOpen(false)}
            />
          </nav>
        )}

        <div className="pb-4 lg:hidden">
          <SearchBar />
        </div>
      </div>
    </header>
  );
}
