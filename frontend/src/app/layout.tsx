import type { Metadata } from 'next';
import { Inter, JetBrains_Mono, Source_Serif_4 } from 'next/font/google';
import { Header } from '@/components/Header';
import { Footer } from '@/components/Footer';
import './globals.css';

const sans = Inter({
  subsets: ['latin'],
  variable: '--font-sans',
  display: 'swap',
});

const serif = Source_Serif_4({
  subsets: ['latin'],
  weight: ['400', '500', '600'],
  variable: '--font-serif',
  display: 'swap',
});

const mono = JetBrains_Mono({
  subsets: ['latin'],
  variable: '--font-mono',
  display: 'swap',
});

export const metadata: Metadata = {
  title: 'RandScan — Rand Protocol Explorer',
  description:
    'Explore blocks, transactions, accounts, validators and confidential programs on the Rand Protocol network.',
  keywords: ['Rand Protocol', 'SHRUGG', 'blockchain', 'explorer', 'blocks', 'transactions'],
};

/**
 * Applies the stored (or system) theme before first paint so the light default
 * never flashes for someone who chose dark.
 */
const themeScript = `(function(){try{var t=localStorage.getItem('randscan-theme');if(t!=='dark'&&t!=='light'){t='light';}document.documentElement.setAttribute('data-theme',t);}catch(e){document.documentElement.setAttribute('data-theme','light');}})();`;

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en" suppressHydrationWarning>
      <head>
        <script dangerouslySetInnerHTML={{ __html: themeScript }} />
      </head>
      <body
        className={`${sans.variable} ${serif.variable} ${mono.variable} min-h-screen bg-bg font-sans text-text`}
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
