import type { Metadata } from 'next';
import localFont from 'next/font/local';
import { Header } from '@/components/Header';
import { Footer } from '@/components/Footer';
import './globals.css';

// The wallet's three faces, self-hosted (src/fonts/SOURCES.txt): Inter for text, JetBrains Mono
// for hashes and keys, Departure Mono for the wordmark and the big figures.
const sans = localFont({
  src: '../fonts/Inter-Variable.woff2',
  weight: '100 900',
  variable: '--font-sans',
  display: 'swap',
});

const mono = localFont({
  src: '../fonts/JetBrainsMono-Variable.woff2',
  weight: '100 800',
  variable: '--font-mono',
  display: 'swap',
});

const display = localFont({
  src: '../fonts/DepartureMono-Regular.woff2',
  weight: '400',
  variable: '--font-display',
  display: 'swap',
});

// metadataBase makes the link-preview card (app/opengraph-image.png, drawn by randprotocol.org's
// design/og/make.mjs) an absolute URL, which scrapers require.
export const metadata: Metadata = {
  metadataBase: new URL('https://randscan.org'),
  title: 'RandScan — Rand Protocol Explorer',
  description:
    'Explore blocks, transactions, accounts, validators and confidential programs on the Rand Protocol network.',
  keywords: ['Rand Protocol', 'RAND', 'blockchain', 'explorer', 'blocks', 'transactions'],
  openGraph: {
    type: 'website',
    url: 'https://randscan.org/',
    siteName: 'RandScan',
    title: 'RandScan — Rand Protocol Explorer',
    description:
      'Explore blocks, transactions, accounts, validators and confidential programs on the Rand Protocol network.',
  },
  twitter: { card: 'summary_large_image' },
};

/**
 * Applies the stored theme before first paint, or follows the operating system when nothing is
 * stored (the wallet does the same), so the wrong theme never flashes.
 */
const themeScript = `(function(){var d=document.documentElement;try{var t=localStorage.getItem('randscan-theme');if(t!=='dark'&&t!=='light'){t=window.matchMedia('(prefers-color-scheme: light)').matches?'light':'dark';}d.setAttribute('data-theme',t);}catch(e){d.setAttribute('data-theme','dark');}})();`;

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en" suppressHydrationWarning>
      <head>
        <meta name="color-scheme" content="dark light" />
        <script dangerouslySetInnerHTML={{ __html: themeScript }} />
      </head>
      <body
        className={`${sans.variable} ${mono.variable} ${display.variable} min-h-screen bg-bg font-sans text-text`}
      >
        <div className="flex min-h-screen flex-col">
          <Header />
          <main className="flex-1">
            <div className="mx-auto max-w-6xl px-6 py-10 lg:px-8">{children}</div>
          </main>
          <Footer />
        </div>
      </body>
    </html>
  );
}
