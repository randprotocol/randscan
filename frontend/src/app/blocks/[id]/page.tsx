'use client';

import Link from 'next/link';
import { Hash } from '@/components/Hash';
import { DetailSkeleton } from '@/components/Loading';
import { TransactionsTable } from '@/components/Tables';
import { DetailRow, ErrorState, NotFoundState, PageHeader, Panel } from '@/components/States';
import { isNotFound, useBlock } from '@/hooks/useApi';
import { formatDateTime, formatNumber, formatTimestamp } from '@/lib/utils';

export default function BlockDetailPage({ params }: { params: { id: string } }) {
  const id = decodeURIComponent(params.id);
  const { data: block, error, isLoading, mutate } = useBlock(id);

  if (isLoading && !block) {
    return <DetailSkeleton />;
  }

  if (error && isNotFound(error)) {
    return (
      <NotFoundState
        title="Block not found"
        message={`No block matches "${id}". Blocks can be looked up by height or by 64-character hash.`}
        backHref="/blocks"
        backLabel="Back to blocks"
      />
    );
  }

  if (error || !block) {
    return <ErrorState message="Could not load this block." onRetry={() => void mutate()} />;
  }

  return (
    <div className="space-y-6">
      <PageHeader
        title={`Block #${formatNumber(block.height)}`}
        subtitle={
          <span className="inline-flex items-center gap-2">
            <Hash value={block.hash} full copyable />
          </span>
        }
      />

      <Panel title="Overview">
        <DetailRow label="Height">
          <span className="font-mono">{formatNumber(block.height)}</span>
        </DetailRow>
        <DetailRow label="Hash">
          <Hash value={block.hash} full />
        </DetailRow>
        <DetailRow label="Parent">
          {block.parent && !/^0+$/.test(block.parent) ? (
            <Hash value={block.parent} href={`/blocks/${block.parent}`} full />
          ) : (
            <span className="text-mute">Genesis (no parent)</span>
          )}
        </DetailRow>
        <DetailRow label="Proposer">
          <Hash value={block.proposer} href={`/validators/${block.proposer}`} full />
        </DetailRow>
        <DetailRow label="Timestamp">
          <span>
            {formatDateTime(block.timestamp_ms)}{' '}
            <span className="text-mute">({formatTimestamp(block.timestamp_ms)})</span>
          </span>
        </DetailRow>
        <DetailRow label="View">
          <span className="font-mono">{formatNumber(block.view)}</span>
        </DetailRow>
        <DetailRow label="Justify view">
          <span className="font-mono">{formatNumber(block.justify_view)}</span>
        </DetailRow>
        <DetailRow label="Transactions">
          <span className="font-mono">{formatNumber(block.tx_count)}</span>
        </DetailRow>
        <DetailRow label="Transaction root">
          <Hash value={block.tx_root} full />
        </DetailRow>
        <DetailRow label="State root">
          <Hash value={block.state_root} full />
        </DetailRow>
      </Panel>

      <section className="space-y-4">
        <div className="flex items-center justify-between">
          <h2 className="chip">Transactions ({formatNumber(block.transactions.length)})</h2>
          {block.height > 0 && (
            <Link href={`/blocks/${block.height - 1}`} className="link text-sm">
              ← Previous block
            </Link>
          )}
        </div>
        <TransactionsTable
          transactions={block.transactions}
          hideColumns={['height']}
          emptyMessage="This block contains no transactions"
        />
      </section>
    </div>
  );
}
