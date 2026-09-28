import type { Config } from 'tailwindcss';

/**
 * Every colour resolves to a CSS variable defined in globals.css, so the dark
 * and light themes swap by flipping `data-theme` on <html> with no `dark:`
 * variants in the markup. The values are the Rand Wallet's design tokens.
 */
const config: Config = {
  content: [
    './src/pages/**/*.{js,ts,jsx,tsx,mdx}',
    './src/components/**/*.{js,ts,jsx,tsx,mdx}',
    './src/app/**/*.{js,ts,jsx,tsx,mdx}',
  ],
  darkMode: ['selector', '[data-theme="dark"]'],
  theme: {
    extend: {
      colors: {
        bg: 'var(--color-bg)',
        'bg-soft': 'var(--color-bg-soft)',
        surface: 'var(--color-surface)',
        'surface-2': 'var(--color-surface-2)',
        border: 'var(--color-border)',
        'border-soft': 'var(--color-border-soft)',
        text: 'var(--color-text)',
        soft: 'var(--color-text-soft)',
        mute: 'var(--color-text-mute)',
        strong: 'var(--color-text-strong)',
        accent: 'var(--color-accent)',
        'accent-2': 'var(--color-accent-2)',
        'on-accent': 'var(--color-on-accent)',
        positive: 'var(--color-positive)',
        negative: 'var(--color-negative)',
        warning: 'var(--color-warning)',
      },
      // The wallet's radius scale: 8 / 12 / 16 / 24, pills for chips and dots.
      borderRadius: {
        none: '0',
        DEFAULT: '8px',
        sm: '8px',
        md: '12px',
        lg: '16px',
        xl: '24px',
        '2xl': '24px',
        '3xl': '24px',
        full: '9999px',
      },
      fontFamily: {
        sans: ['var(--font-sans)', 'Inter', 'ui-sans-serif', 'system-ui', 'sans-serif'],
        mono: ['var(--font-mono)', 'ui-monospace', 'SFMono-Regular', 'monospace'],
        display: ['var(--font-display)', 'var(--font-mono)', 'ui-monospace', 'monospace'],
      },
      fontSize: {
        '2xs': ['0.6875rem', { lineHeight: '1rem' }],
      },
      minHeight: {
        10: '2.5rem',
      },
    },
  },
  plugins: [],
};

export default config;
