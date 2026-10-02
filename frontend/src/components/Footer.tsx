'use client';

import { Mark } from './Brand';
import { L, useT } from '@/i18n/client';

export function Footer() {
  const { t } = useT();
  return (
    <footer className="border-t border-border-soft">
      <div className="mx-auto flex max-w-6xl flex-col items-start justify-between gap-3 px-6 py-8 text-[0.8125rem] text-mute sm:flex-row sm:items-center lg:px-8">
        <p className="brand gap-2">
          <Mark size={14} />
          <span>{t('footer.blurb')}</span>
        </p>
        <div className="flex items-center gap-5">
          <a
            href="https://randprotocol.org"
            target="_blank"
            rel="noopener noreferrer"
            className="transition-colors hover:text-strong"
          >
            randprotocol.org
          </a>
          <a
            href="https://github.com/randprotocol"
            target="_blank"
            rel="noopener noreferrer"
            className="transition-colors hover:text-strong"
          >
            GitHub
          </a>
          <L href="/privacy" className="transition-colors hover:text-strong">
            {t('footer.privacy')}
          </L>
          <L href="/terms" className="transition-colors hover:text-strong">
            {t('footer.terms')}
          </L>
        </div>
      </div>
    </footer>
  );
}
