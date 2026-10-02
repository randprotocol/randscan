import type { Metadata } from 'next';
import { LegalPage, legalSections, type LegalSectionText } from '@/components/LegalPage';
import { serverT } from '@/i18n/server';

type P = { params: Promise<{ lang: string }> };

export async function generateMetadata({ params }: P): Promise<Metadata> {
  const { t } = await serverT(params);
  return {
    title: t('terms.metaTitle'),
    description: t('terms.metaDescription'),
  };
}

export default async function TermsPage({ params }: P) {
  const { t, get, lang } = await serverT(params);
  const effective = t('terms.effective');
  return (
    <LegalPage
      title={t('terms.title')}
      effective={effective}
      intro={t('terms.intro')}
      sections={legalSections(get<LegalSectionText[]>('terms.sections') ?? [], lang, { effective })}
      related={{ href: '/privacy', label: t('privacy.title') }}
      lang={lang}
      words={{
        label: t('legal.label'),
        effective: t('legal.effective', { date: effective }),
        contents: t('legal.contents'),
        seeAlso: t('legal.seeAlso'),
        notice: lang !== 'en' ? t('common.englishGoverns') : undefined,
      }}
    />
  );
}
