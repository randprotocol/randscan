import type { Config } from 'tailwindcss';

/**
 * Every colour resolves to a CSS variable defined in globals.css, so the light
 * and dark themes swap by flipping `data-theme` on <html> with no `dark:`
 * variants in the markup.
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
        'accent-3': 'var(--color-accent-3)',
        'on-accent': 'var(--color-on-accent)',
        live: 'var(--color-live)',
      },
      borderRadius: {
        none: '0',
        DEFAULT: '3px',
        sm: '3px',
        md: '3px',
        lg: '3px',
        xl: '3px',
        '2xl': '3px',
        '3xl': '3px',
        full: '9999px',
      },
      fontFamily: {
        sans: ['var(--font-sans)', 'ui-sans-serif', 'system-ui', 'sans-serif'],
        serif: ['var(--font-serif)', 'Georgia', 'ui-serif', 'serif'],
        mono: ['var(--font-mono)', 'ui-monospace', 'SFMono-Regular', 'monospace'],
      },
      fontSize: {
        '2xs': ['0.6875rem', { lineHeight: '1rem' }],
      },
    },
  },
  plugins: [],
};

export default config;
