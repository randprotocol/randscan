import { cn } from '@/lib/utils';

/**
 * The Rand mark: one 4×4 Bayer tile of the wallet's entropy field at a single threshold, with one
 * cell burning in the signal colour. The same cells as `markSvg()` in clients/ui/lib/entropy.js
 * and the app icons, so the explorer and the wallet carry one logo.
 */
const ON = [0, 2, 5, 7, 8, 10, 13, 15, 1, 11];
const HOT = 11;

export function Mark({ size = 22, className }: { size?: number; className?: string }) {
  return (
    <svg
      className={cn('mark', className)}
      width={size}
      height={size}
      viewBox="0 0 23 23"
      aria-hidden="true"
      focusable="false"
    >
      {ON.map((i) => (
        <rect
          key={i}
          className={i === HOT ? 'hot' : undefined}
          x={(i % 4) * 6}
          y={Math.floor(i / 4) * 6}
          width={5}
          height={5}
        />
      ))}
    </svg>
  );
}

/** Mark plus wordmark. The visible word is lowercase in the pixel face; the accessible name is the product's. */
export function Brand({ className }: { className?: string }) {
  return (
    <span className={cn('brand', className)}>
      <Mark />
      <span className="name" aria-hidden="true">
        randscan
      </span>
      <span className="sr-only">RandScan</span>
    </span>
  );
}
