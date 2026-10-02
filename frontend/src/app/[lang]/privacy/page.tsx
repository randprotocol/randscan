import type { Metadata } from 'next';
import { LegalPage, legalSections, type LegalSectionText } from '@/components/LegalPage';
import { serverT } from '@/i18n/server';

type P = { params: Promise<{ lang: string }> };

export async function generateMetadata({ params }: P): Promise<Metadata> {
  const { t } = await serverT(params);
  return {
    title: t('privacy.metaTitle'),
    description: t('privacy.metaDescription'),
  };
}

export default async function PrivacyPage({ params }: P) {
  const { t, get, lang } = await serverT(params);
  const effective = t('privacy.effective');
  return (
    <LegalPage
      title={t('privacy.title')}
      effective={effective}
      intro={t('privacy.intro')}
      sections={legalSections(get<LegalSectionText[]>('privacy.sections') ?? [], lang, { effective })}
      related={{ href: '/terms', label: t('terms.title') }}
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
