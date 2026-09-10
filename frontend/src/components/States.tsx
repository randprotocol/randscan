import Link from 'next/link';
import { cn } from '@/lib/utils';

/** Accent section label with the short leading rule, as on randprotocol.org. */
export function SectionLabel({ children, className }: { children: React.ReactNode; className?: string }) {
  return <span className={cn('chip', className)}>{children}</span>;
}

interface SectionHeadingProps {
  label: string;
  title?: string;
  actions?: React.ReactNode;
  className?: string;
}

export function SectionHeading({ label, title, actions, className }: SectionHeadingProps) {
  return (
    <div className={cn('flex flex-wrap items-end justify-between gap-3', className)}>
      <div>
        <SectionLabel>{label}</SectionLabel>
        {title && (
          <h2 className="mt-1.5 font-serif text-2xl font-medium tracking-tight text-strong">
            {title}
          </h2>
        )}
      </div>
      {actions}
    </div>
  );
}

interface PageHeaderProps {
  title: string;
  label?: string;
  subtitle?: React.ReactNode;
  actions?: React.ReactNode;
  className?: string;
}

export function PageHeader({ title, label, subtitle, actions, className }: PageHeaderProps) {
  return (
    <div className={cn('mb-8 flex flex-wrap items-start justify-between gap-4', className)}>
      <div className="min-w-0">
        {label && <SectionLabel className="mb-2">{label}</SectionLabel>}
        <h1 className="font-serif text-4xl font-medium tracking-tight text-strong">{title}</h1>
        {subtitle && <div className="mt-2 text-sm text-soft">{subtitle}</div>}
      </div>
      {actions && <div className="flex items-center gap-3">{actions}</div>}
    </div>
  );
}

interface NotFoundStateProps {
  title?: string;
  message: string;
  backHref?: string;
  backLabel?: string;
}

export function NotFoundState({
  title = 'Not found',
  message,
  backHref = '/',
  backLabel = 'Back to dashboard',
}: NotFoundStateProps) {
  return (
    <div className="card-padded py-16 text-center">
      <h2 className="font-serif text-2xl font-medium tracking-tight text-strong">{title}</h2>
      <p className="mx-auto mt-3 max-w-md break-all text-sm text-soft">{message}</p>
      <Link href={backHref} className="btn-secondary mt-7">
        {backLabel}
      </Link>
    </div>
  );
}

interface ErrorStateProps {
  title?: string;
  message?: string;
  onRetry?: () => void;
}

export function ErrorState({
  title = 'Something went wrong',
  message = 'The explorer API could not be reached. Please try again.',
  onRetry,
}: ErrorStateProps) {
  return (
    <div className="card-padded py-16 text-center">
      <h2 className="font-serif text-2xl font-medium tracking-tight text-accent-3">{title}</h2>
      <p className="mx-auto mt-3 max-w-md text-sm text-soft">{message}</p>
      {onRetry && (
        <button type="button" onClick={onRetry} className="btn-primary mt-7">
          Retry
        </button>
      )}
    </div>
  );
}

interface DetailRowProps {
  label: string;
  children: React.ReactNode;
  className?: string;
}

export function DetailRow({ label, children, className }: DetailRowProps) {
  return (
    <div className={cn('detail-row', className)}>
      <div className="detail-label">{label}</div>
      <div className="detail-value">{children}</div>
    </div>
  );
}

interface PanelProps {
  title?: string;
  children: React.ReactNode;
  actions?: React.ReactNode;
  className?: string;
}

export function Panel({ title, children, actions, className }: PanelProps) {
  return (
    <section className={cn('card overflow-hidden', className)}>
      {(title || actions) && (
        <header className="flex items-center justify-between border-b border-border px-6 py-3.5">
          {title && <SectionLabel>{title}</SectionLabel>}
          {actions}
        </header>
      )}
      <div className="px-6 py-2">{children}</div>
    </section>
  );
}

/** Muted green/grey dot reflecting the WebSocket connection. */
export function LiveIndicator({ isConnected }: { isConnected: boolean }) {
  return (
    <span className="inline-flex items-center gap-2 text-xs text-mute">
      <span className={cn('status-dot', isConnected ? 'status-dot-live' : 'status-dot-off')} />
      {isConnected ? 'Live' : 'Offline'}
    </span>
  );
}
