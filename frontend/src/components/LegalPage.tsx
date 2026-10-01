import Link from 'next/link';
import { PageHeader } from './States';

/** A section of a legal document: a heading and its paragraphs or lists. */
export interface LegalSection {
  id: string;
  title: string;
  body: React.ReactNode;
}

interface LegalPageProps {
  title: string;
  /** The date the text took effect, already formatted ("1 October 2026"). */
  effective: string;
  /** One or two sentences under the title saying what the document covers. */
  intro: React.ReactNode;
  sections: LegalSection[];
  /** The other legal document, linked at the foot of this one. */
  related: { href: string; label: string };
}

/**
 * The layout shared by /privacy and /terms: the page header, a contents list, numbered sections
 * with anchors, and a pointer to the companion document. Plain text, in the body face, at a
 * reading width.
 */
export function LegalPage({ title, effective, intro, sections, related }: LegalPageProps) {
  return (
    <div className="mx-auto max-w-3xl">
      <PageHeader title={title} label="Legal" subtitle={<>Effective {effective}</>} />

      <div className="legal">
        <p className="text-base text-text">{intro}</p>

        <nav aria-label="Contents" className="card-padded mt-8">
          <h2 className="chip">Contents</h2>
          <ol className="mt-3 columns-1 gap-x-8 space-y-1.5 text-sm sm:columns-2">
            {sections.map((section, index) => (
              <li key={section.id}>
                <a href={`#${section.id}`} className="link">
                  <span className="mr-2 font-mono text-xs text-mute">{index + 1}</span>
                  {section.title}
                </a>
              </li>
            ))}
          </ol>
        </nav>

        {sections.map((section, index) => (
          <section key={section.id} id={section.id} className="mt-10 scroll-mt-6">
            <h2 className="text-xl font-semibold tracking-tight text-strong">
              <span className="mr-3 font-mono text-sm text-mute">{index + 1}</span>
              {section.title}
            </h2>
            <div className="mt-3 space-y-3 text-[0.9375rem] leading-relaxed text-soft">
              {section.body}
            </div>
          </section>
        ))}

        <p className="mt-12 border-t border-border-soft pt-6 text-sm text-mute">
          See also the{' '}
          <Link href={related.href} className="link">
            {related.label}
          </Link>
          .
        </p>
      </div>
    </div>
  );
}
