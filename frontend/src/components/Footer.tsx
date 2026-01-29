import Link from 'next/link';

const footerLinks = {
  explorer: [
    { href: '/blocks', label: 'Blocks' },
    { href: '/transactions', label: 'Transactions' },
    { href: '/validators', label: 'Validators' },
    { href: '/tokens', label: 'Tokens' },
  ],
  resources: [
    { href: 'https://docs.randprotocol.com', label: 'Documentation', external: true },
    { href: 'https://github.com/randprotocol', label: 'GitHub', external: true },
    { href: 'https://discord.gg/randprotocol', label: 'Discord', external: true },
  ],
};

export function Footer() {
  return (
    <footer className="border-t border-dark-700 bg-dark-900">
      <div className="mx-auto max-w-7xl px-4 py-12 sm:px-6 lg:px-8">
        <div className="grid grid-cols-1 gap-8 md:grid-cols-4">
          {/* Brand */}
          <div className="col-span-1 md:col-span-2">
            <Link href="/" className="flex items-center gap-2">
              <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-primary-600">
                <span className="text-lg font-bold text-white">R</span>
              </div>
              <span className="text-xl font-bold text-white">RandScan</span>
            </Link>
            <p className="mt-4 max-w-md text-sm text-dark-400">
              RandScan is a blockchain explorer for the Rand Protocol network.
              Explore blocks, transactions, validators, and tokens with real-time updates.
            </p>
          </div>

          {/* Explorer Links */}
          <div>
            <h3 className="text-sm font-semibold text-white">Explorer</h3>
            <ul className="mt-4 space-y-3">
              {footerLinks.explorer.map((link) => (
                <li key={link.href}>
                  <Link
                    href={link.href}
                    className="text-sm text-dark-400 hover:text-primary-400"
                  >
                    {link.label}
                  </Link>
                </li>
              ))}
            </ul>
          </div>

          {/* Resources */}
          <div>
            <h3 className="text-sm font-semibold text-white">Resources</h3>
            <ul className="mt-4 space-y-3">
              {footerLinks.resources.map((link) => (
                <li key={link.href}>
                  <a
                    href={link.href}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="inline-flex items-center gap-1 text-sm text-dark-400 hover:text-primary-400"
                  >
                    {link.label}
                    <svg
                      className="h-3 w-3"
                      fill="none"
                      viewBox="0 0 24 24"
                      strokeWidth="2"
                      stroke="currentColor"
                    >
                      <path
                        strokeLinecap="round"
                        strokeLinejoin="round"
                        d="M4.5 19.5l15-15m0 0H8.25m11.25 0v11.25"
                      />
                    </svg>
                  </a>
                </li>
              ))}
            </ul>
          </div>
        </div>

        {/* Bottom */}
        <div className="mt-12 border-t border-dark-700 pt-8">
          <p className="text-center text-sm text-dark-500">
            &copy; {new Date().getFullYear()} Rand Protocol. All rights reserved.
          </p>
        </div>
      </div>
    </footer>
  );
}
