"use client";

import Link from "next/link";
import { Column, DataTable } from "@/components/DataTable";
import { CopyButton, Hash } from "@/components/Hash";
import { DetailSkeleton } from "@/components/Loading";
import { DetailRow, ErrorState, PageHeader, Panel } from "@/components/States";
import { StatsCard, StatsRow } from "@/components/StatsCard";
import {
  useBridge,
  useBridgeAssets,
  useBridgeTokens,
  useStats,
} from "@/hooks/useApi";
import {
  BRIDGE_SOURCE_CHAINS,
  bridgeChainName,
  cn,
  formatBridgeChain,
  formatBridgeUnits,
  formatNumber,
} from "@/lib/utils";
import type { ApprovedToken, BridgeAsset, BridgeAssetActivity } from "@/types";

// ---------------------------------------------------------------------------
// Bridged tokens, one table per source chain
// ---------------------------------------------------------------------------

function TokenCell({ asset }: { asset: BridgeAssetActivity }) {
  if (asset.symbol) {
    return (
      <span className="flex flex-col">
        <span className="font-medium text-strong">{asset.symbol}</span>
        <span className="text-xs text-mute">{asset.name}</span>
      </span>
    );
  }
  return (
    <span className="flex flex-col">
      <Hash value={asset.token} start={10} end={8} />
      <span className="text-xs text-mute">not on the approved list</span>
    </span>
  );
}

function FlowCell({
  count,
  units,
  symbol,
}: {
  count: number;
  units: string;
  symbol: string | null;
}) {
  if (count === 0) return <span className="text-mute">—</span>;
  return (
    <span className="flex flex-col">
      <span className="font-mono">{formatBridgeUnits(units, symbol)}</span>
      <span className="text-xs text-mute">
        {formatNumber(count)} {count === 1 ? "transaction" : "transactions"}
      </span>
    </span>
  );
}

const activityColumns: Column<BridgeAssetActivity>[] = [
  { key: "token", header: "Token", render: (a) => <TokenCell asset={a} /> },
  {
    key: "index",
    header: "Asset",
    render: (a) => <span className="font-mono">#{formatNumber(a.index)}</span>,
  },
  {
    key: "in",
    header: "Bridged in",
    render: (a) => (
      <FlowCell count={a.deposits} units={a.deposited} symbol={a.symbol} />
    ),
  },
  {
    key: "out",
    header: "Bridged out",
    render: (a) => (
      <FlowCell count={a.burns} units={a.burned} symbol={a.symbol} />
    ),
  },
  {
    key: "outstanding",
    header: "On Rand now",
    render: (a) => (
      <span className="font-mono text-strong">
        {formatBridgeUnits(a.outstanding, a.symbol)}
      </span>
    ),
  },
  {
    key: "last",
    header: "Last activity",
    render: (a) =>
      a.last_height === null ? (
        <span className="text-mute">—</span>
      ) : (
        <Link
          href={`/blocks/${a.last_height}`}
          className="font-mono text-soft hover:text-strong"
        >
          #{formatNumber(a.last_height)}
        </Link>
      ),
  },
];

function SourceChainPanel({
  chain,
  name,
  emitter,
  assets,
}: {
  chain: number;
  name: string;
  emitter: string | undefined;
  assets: BridgeAssetActivity[];
}) {
  const deposits = assets.reduce((n, a) => n + a.deposits, 0);
  const burns = assets.reduce((n, a) => n + a.burns, 0);
  return (
    <section className="space-y-3">
      <div className="flex flex-wrap items-baseline justify-between gap-x-6 gap-y-1">
        <h3 className="font-serif text-lg text-strong">{name}</h3>
        <p className="text-xs text-mute">
          {emitter ? (
            <>
              bridge contract{" "}
              <Hash value={emitter} start={8} end={6} className="text-xs" />
            </>
          ) : (
            "no bridge contract registered for this chain"
          )}
          {assets.length > 0 && (
            <>
              {" · "}
              {formatNumber(deposits)} in, {formatNumber(burns)} out
            </>
          )}
        </p>
      </div>
      <DataTable
        columns={activityColumns}
        data={assets}
        keyExtractor={(a) => String(a.index)}
        emptyMessage={`Nothing has been bridged from ${name} yet · chain id ${chain}`}
      />
    </section>
  );
}

// ---------------------------------------------------------------------------
// Approved tokens
// ---------------------------------------------------------------------------

function StatusBadge({ status }: { status: ApprovedToken["status"] }) {
  return (
    <span
      className={cn(
        "badge",
        status === "allowed" ? "badge-bridge" : "badge-neutral line-through",
      )}
    >
      {status === "allowed" ? "allowed" : "discontinued"}
    </span>
  );
}

const tokenColumns: Column<ApprovedToken>[] = [
  {
    key: "token",
    header: "Token",
    render: (t) => (
      <span className="flex flex-col">
        <span className="font-medium text-strong">{t.symbol}</span>
        <span className="text-xs text-mute">{t.name}</span>
      </span>
    ),
  },
  {
    key: "chain",
    header: "Chain",
    render: (t) => (
      <span className="flex flex-col">
        <span>{t.chain_name}</span>
        <span className="text-xs text-mute">{t.standard}</span>
      </span>
    ),
  },
  {
    key: "address",
    header: "Contract address",
    className: "min-w-[22rem]",
    render: (t) => (
      <span className="inline-flex items-center gap-1.5">
        <a
          href={t.explorer_url}
          target="_blank"
          rel="noopener noreferrer"
          className="break-all font-mono text-sm text-text hover:text-strong hover:underline"
          title={`Open on the ${t.chain_name} explorer`}
        >
          {t.address}
        </a>
        <CopyButton value={t.address} />
      </span>
    ),
  },
  {
    key: "decimals",
    header: "Decimals",
    render: (t) => (
      <span className="flex flex-col font-mono">
        <span>{t.decimals}</span>
        <span className="text-xs text-mute">8 on Rand</span>
      </span>
    ),
  },
  {
    key: "status",
    header: "Status",
    render: (t) => <StatusBadge status={t.status} />,
  },
];

function ApprovedTokens({ tokens }: { tokens: ApprovedToken[] }) {
  const notes = tokens.filter((t) => t.note);
  return (
    <section className="space-y-4">
      <h2 className="chip">Approved tokens</h2>
      <p className="text-sm text-soft">
        The bridge accepts these tokens and nothing else. Send only to the
        bridge contract on the token&apos;s own chain; a deposit of any other
        token is refused by the contract, and a deposit of an approved token on
        the wrong chain is a different asset on Rand.
      </p>
      <DataTable
        columns={tokenColumns}
        data={tokens}
        keyExtractor={(t) => `${t.chain}-${t.symbol}`}
        emptyMessage="No approved tokens"
      />
      {notes.length > 0 && (
        <ul className="space-y-1 text-xs text-mute">
          {notes.map((t) => (
            <li key={`${t.chain}-${t.symbol}`}>
              <span className="text-soft">
                {t.symbol} on {t.chain_name}:
              </span>{" "}
              {t.note}
            </li>
          ))}
        </ul>
      )}
      <p className="text-xs text-mute">
        Amounts on Rand always carry 8 decimals, whatever the token has at home;
        the bridge contract converts on the way in and out. The registry stores
        each address as a 32-byte word: an Ethereum, BSC or Tron address
        left-padded with zeros, a Solana mint as is.
      </p>
    </section>
  );
}

// ---------------------------------------------------------------------------
// Registry (the node's raw view)
// ---------------------------------------------------------------------------

const registryColumns: Column<BridgeAssetActivity | BridgeAsset>[] = [
  {
    key: "index",
    header: "Index",
    render: (a) => <span className="font-mono">#{formatNumber(a.index)}</span>,
  },
  {
    key: "chain",
    header: "Source chain",
    render: (a) => <span>{formatBridgeChain(a.chain)}</span>,
  },
  {
    key: "symbol",
    header: "Token",
    render: (a) =>
      "symbol" in a && a.symbol ? (
        <span>{a.symbol}</span>
      ) : (
        <span className="text-mute">unknown</span>
      ),
  },
  {
    key: "token",
    header: "Token address (32 bytes)",
    render: (a) => <Hash value={a.token} start={10} end={8} />,
  },
  {
    key: "asset_id",
    header: "Asset id",
    render: (a) => <Hash value={a.asset_id} start={10} end={8} />,
  },
];

// ---------------------------------------------------------------------------
// Page
// ---------------------------------------------------------------------------

export default function BridgePage() {
  const { data: bridge, error, isLoading, mutate } = useBridge();
  const { data: activity } = useBridgeAssets();
  const { data: tokens } = useBridgeTokens();
  const { data: stats } = useStats();

  if (isLoading && !bridge) {
    return <DetailSkeleton />;
  }

  if (error || !bridge) {
    return (
      <>
        <PageHeader title="Bridge" />
        <ErrorState
          message="Could not load the bridge state."
          onRetry={() => void mutate()}
        />
      </>
    );
  }

  const emitters = Object.entries(bridge.emitters);
  const assets = activity ?? [];
  const byChain = new Map<number, BridgeAssetActivity[]>();
  for (const a of assets) {
    byChain.set(a.chain, [...(byChain.get(a.chain) ?? []), a]);
  }
  // Any chain the registry names that is not one of the four we list (should not happen).
  const otherChains = [...byChain.keys()].filter(
    (id) => !BRIDGE_SOURCE_CHAINS.some((c) => c.id === id),
  );
  const deposits = assets.reduce((n, a) => n + a.deposits, 0);
  const burns = assets.reduce((n, a) => n + a.burns, 0);
  const activeChains = BRIDGE_SOURCE_CHAINS.filter(
    (c) => (byChain.get(c.id) ?? []).length > 0,
  );
  const allowed = tokens?.filter((t) => t.status === "allowed").length;

  return (
    <div className="space-y-10">
      <PageHeader
        title="Bridge"
        subtitle="Tokens locked on Ethereum, BSC, Solana and Tron and minted here as shielded notes. Bridged value is notes, so there are no balances: only what came in and what went out."
      />

      {!bridge.enabled && (
        <Panel>
          <p className="py-4 text-sm text-mute">
            Chain {stats?.chain_id ?? ""} has no bridge section in its genesis:
            no guardians, no asset registry and no bridged notes yet. The source
            chains and the approved tokens below are what the bridge will accept
            once a chain with a bridge section is running.
          </p>
        </Panel>
      )}

      <StatsRow columns={4}>
        <StatsCard
          title="Bridged in"
          value={formatNumber(deposits)}
          subtitle="deposits attested"
        />
        <StatsCard
          title="Bridged out"
          value={formatNumber(burns)}
          subtitle="burns released"
        />
        <StatsCard
          title="Assets seen"
          value={formatNumber(assets.length)}
          subtitle={
            activeChains.length === 0
              ? "no source chain active yet"
              : `from ${activeChains.map((c) => c.name).join(", ")}`
          }
        />
        <StatsCard
          title="Approved tokens"
          value={allowed === undefined ? "—" : formatNumber(allowed)}
          subtitle="USDT and USDC on four chains"
        />
      </StatsRow>

      <section className="space-y-6">
        <h2 className="chip">Bridged tokens by source chain</h2>
        {BRIDGE_SOURCE_CHAINS.map((c) => (
          <SourceChainPanel
            key={c.id}
            chain={c.id}
            name={c.name}
            emitter={bridge.emitters[String(c.id)]}
            assets={byChain.get(c.id) ?? []}
          />
        ))}
        {otherChains.map((id) => (
          <SourceChainPanel
            key={id}
            chain={id}
            name={bridgeChainName(id)}
            emitter={bridge.emitters[String(id)]}
            assets={byChain.get(id) ?? []}
          />
        ))}
      </section>

      {tokens && <ApprovedTokens tokens={tokens} />}

      {bridge.enabled && (
        <>
          <Panel title="Bridge state">
            <DetailRow label="Outbound emitter">
              <Hash value={bridge.emitter} full />
            </DetailRow>
            <DetailRow label="Guardian set">
              <span className="font-mono">
                {bridge.guardian_set_index === null
                  ? "—"
                  : `#${formatNumber(bridge.guardian_set_index)}`}
              </span>
            </DetailRow>
            <DetailRow label={`Guardians (${bridge.guardians.length})`}>
              {bridge.guardians.length === 0 ? (
                <span className="text-mute">—</span>
              ) : (
                <ul className="space-y-1.5">
                  {bridge.guardians.map((g, i) => (
                    <li key={g} className="flex items-center gap-2">
                      <span className="w-6 flex-shrink-0 text-right font-mono text-xs text-mute">
                        {i}
                      </span>
                      <Hash value={g} full />
                    </li>
                  ))}
                </ul>
              )}
            </DetailRow>
            <DetailRow label="Messages emitted">
              <span className="font-mono">
                {bridge.burn_sequence === null
                  ? "—"
                  : formatNumber(bridge.burn_sequence)}
              </span>
            </DetailRow>
            <DetailRow label="Next asset index">
              <span className="font-mono">
                {bridge.next_index === null
                  ? "—"
                  : formatNumber(bridge.next_index)}
              </span>
            </DetailRow>
            <DetailRow label={`Trusted emitters (${emitters.length})`}>
              {emitters.length === 0 ? (
                <span className="text-mute">—</span>
              ) : (
                <ul className="space-y-1.5">
                  {emitters.map(([chain, address]) => (
                    <li
                      key={chain}
                      className="flex flex-wrap items-center gap-2"
                    >
                      <span className="text-soft">
                        {formatBridgeChain(Number(chain))}
                      </span>
                      <Hash value={address} full />
                    </li>
                  ))}
                </ul>
              )}
            </DetailRow>
          </Panel>

          <section className="space-y-4">
            <h2 className="chip">Asset registry</h2>
            <DataTable
              columns={registryColumns}
              data={activity ?? bridge.assets}
              keyExtractor={(a) => String(a.index)}
              emptyMessage="No bridged asset registered yet"
            />
            <p className="text-xs text-mute">
              The node&apos;s own registry, filled on the first deposit of each
              asset. The index is the asset word a bridged note carries; index 0
              is SHRUGG and is never in the registry.
            </p>
          </section>
        </>
      )}
    </div>
  );
}
