'use client';

import { useParams } from 'next/navigation';
import Link from 'next/link';
import { Hash } from '@/components/Hash';
import { DetailSkeleton } from '@/components/Loading';
import { DetailRow, ErrorState, NotFoundState, PageHeader, Panel } from '@/components/States';
import { isNotFound, useNote } from '@/hooks/useApi';
import { formatNumber } from '@/lib/utils';

export default function NoteDetailPage() {
  const params = useParams<{ id: string }>();
  const id = decodeURIComponent(params.id);
  const { data: note, error, isLoading, mutate } = useNote(id);

  if (isLoading && !note) {
    return <DetailSkeleton />;
  }

  if (error && isNotFound(error)) {
    return (
      <NotFoundState
        title="Note not found"
        message={`No tree leaf matches "${id}". Notes can be looked up by commitment (64 hex) or by leaf index; a commitment from a very recent block may not be paged in yet.`}
        backHref="/notes"
        backLabel="Back to notes"
      />
    );
  }

  if (error || !note) {
    return <ErrorState message="Could not load this note." onRetry={() => void mutate()} />;
  }

  return (
    <div className="space-y-6">
      <PageHeader
        title={`Note #${formatNumber(note.leaf_index)}`}
        subtitle={<Hash value={note.cm} full copyable />}
      />

      <Panel title="Leaf">
        <DetailRow label="Leaf index">
          <span className="font-mono">{formatNumber(note.leaf_index)}</span>
        </DetailRow>
        <DetailRow label="Commitment">
          <Hash value={note.cm} full />
        </DetailRow>
        <DetailRow label="Created at height">
          <Link href={`/blocks/${note.height}`} className="link font-mono">
            #{formatNumber(note.height)}
          </Link>
        </DetailRow>
        <DetailRow label="Created by">
          {note.tx_hash ? (
            <Hash value={note.tx_hash} href={`/transactions/${note.tx_hash}`} full />
          ) : (
            <span className="text-mute">
              {note.height === 0
                ? 'A genesis deposit note'
                : 'A deposit whose commitment is not on the wire (withdraw or bridge attestation)'}
            </span>
          )}
        </DetailRow>
        <p className="px-4 py-3 text-xs text-mute">
          {note.tx_hash && (
            <>
              Hold the receiver&apos;s or the sender&apos;s viewing key, or this transaction&apos;s key?{' '}
              <Link href={`/transactions/${note.tx_hash}`} className="link">
                Open it with a key on the transaction page
              </Link>
              . <br />
            </>
          )}
          The owner and the amount are inside the sealed envelope and open only for the
          receiver&apos;s and the sender&apos;s viewing keys. Whether this note has been spent is
          not public either: a nullifier cannot be matched to its leaf.
        </p>
      </Panel>
    </div>
  );
}
