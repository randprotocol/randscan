'use client';

import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { SearchBar } from './SearchBar';
import { cn } from '@/lib/utils';

const navLinks = [
  { href: '/', label: 'Dashboard' },
  { href: '/blocks', label: 'Blocks' },
  { href: '/transactions', label: 'Transactions' },
  { href: '/validators', label: 'Validators' },
  { href: '/tokens', label: 'Tokens' },
];

export function Header() {
  const pathname = usePathname();

  return (
    <header className="sticky top-0 z-50 border-b border-dark-700 bg-dark-900/95 backdrop-blur supports-[backdrop-filter]:bg-dark-900/80">
      <div className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
        <div className="flex h-16 items-center justify-between">
          {/* Logo */}
          <Link href="/" className="flex items-center gap-2">
            <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-primary-600">
              <span className="text-lg font-bold text-white">R</span>
            </div>
            <span className="text-xl font-bold text-white">RandScan</span>
          </Link>

          {/* Navigation */}
          <nav className="hidden md:flex md:items-center md:gap-6">
            {navLinks.map((link) => {
              const isActive = pathname === link.href ||
                (link.href !== '/' && pathname.startsWith(link.href));

              return (
                <Link
                  key={link.href}
                  href={link.href}
                  className={cn(
                    'text-sm font-medium transition-colors hover:text-primary-400',
                    isActive ? 'text-primary-400' : 'text-dark-300'
                  )}
                >
                  {link.label}
                </Link>
              );
            })}
          </nav>

          {/* Search */}
          <div className="hidden w-80 lg:block">
            <SearchBar />
          </div>

          {/* Mobile menu button */}
          <button
            type="button"
            className="inline-flex items-center justify-center rounded-md p-2 text-dark-400 hover:bg-dark-800 hover:text-white md:hidden"
          >
            <span className="sr-only">Open main menu</span>
            <svg
              className="h-6 w-6"
              fill="none"
              viewBox="0 0 24 24"
              strokeWidth="1.5"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                d="M3.75 6.75h16.5M3.75 12h16.5m-16.5 5.25h16.5"
              />
            </svg>
          </button>
        </div>

        {/* Mobile Search */}
        <div className="pb-4 lg:hidden">
          <SearchBar />
        </div>
      </div>
    </header>
  );
}
