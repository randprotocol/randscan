"use client";

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
import { BRIDGE_SOURCE_CHAINS, cn, bridgeTokenTotals, knownBridgeChainName } from "@/lib/utils";
import { L, useFmt, useT, type Fmt } from "@/i18n/client";
import type {
  ApprovedToken,
  BridgeAsset,
  BridgeAssetActivity,
  BridgeEndpoint,
} from "@/types";

// ---------------------------------------------------------------------------
// A source chain's bridge contract
// ---------------------------------------------------------------------------

type T = ReturnType<typeof useT>;

/** A bridge chain's own name when it is one we know, so fmt.bridgeChain can say "chain 9" otherwise. */
/**
 * The contract (or program) a source chain's locks must come from, as that chain's own explorer
 * prints it. Falls back to the node's 32-byte word when the API did not derive an address.
 */
function EndpointAddress({
  endpoint,
  emitter,
  className,
}: {
  endpoint: BridgeEndpoint | undefined;
  emitter: string;
  className?: string;
}) {
  const { t } = useT();
  if (!endpoint?.address) return <Hash value={emitter} full className={className} />;
  return (
    <span className={cn("inline-flex min-w-0 items-center gap-1.5", className)}>
      {endpoint.explorer_url ? (
        <a
          href={endpoint.explorer_url}
          target="_blank"
          rel="noopener noreferrer"
          className="break-all font-mono text-text hover:text-strong hover:underline"
          title={
            endpoint.chain_name
              ? t("bridge.openOn", { chain: endpoint.chain_name })
              : t("bridge.openOnChain")
          }
        >
          {endpoint.address}
        </a>
      ) : (
        <span className="break-all font-mono text-soft">{endpoint.address}</span>
      )}
      <CopyButton value={endpoint.address} />
    </span>
  );
}

// ---------------------------------------------------------------------------
// Bridged tokens, one table per source chain
// ---------------------------------------------------------------------------

function TokenCell({ asset }: { asset: BridgeAssetActivity }) {
  const { t } = useT();
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
      <span className="text-xs text-mute">{t("bridge.activity.notApproved")}</span>
    </span>
  );
}

function FlowCell({
  count,
  units,
  symbol,
}: {
  count: number | null;
  units: string | null;
  symbol: string | null;
}) {
  const { t, tp } = useT();
  const fmt = useFmt();
  // A deposit names the token it minted, not the source coin locked for it, so a token with
  // several backings has no per-coin deposit figure; "On Rand now" is the per-coin truth.
  if (count === null || units === null)
    return (
      <span
        className="text-mute"
        title={t("bridge.activity.naTitle")}
      >
        {t("bridge.activity.na")}
      </span>
    );
  if (count === 0) return <span className="text-mute">—</span>;
  return (
    <span className="flex flex-col">
      <span className="font-mono">{fmt.bridgeUnits(units, symbol)}</span>
      <span className="text-xs text-mute">{tp("bridge.activity.transactions", count)}</span>
    </span>
  );
}

/** A registry index is a token; its backings share it, so a row's key is the backing. */
const backingKey = (a: BridgeAsset) => `${a.index}-${a.chain}-${a.token}`;

const activityColumns = ({ t }: T, fmt: Fmt): Column<BridgeAssetActivity>[] => [
  { key: "token", header: t("bridge.activity.token"), render: (a) => <TokenCell asset={a} /> },
  {
    key: "index",
    header: t("bridge.activity.asset"),
    render: (a) => <span className="font-mono">#{fmt.number(a.index)}</span>,
  },
  {
    key: "in",
    header: t("bridge.activity.in"),
    render: (a) => (
      <FlowCell count={a.deposits} units={a.deposited} symbol={a.symbol} />
    ),
  },
  {
    key: "out",
    header: t("bridge.activity.out"),
    render: (a) => (
      <FlowCell count={a.burns} units={a.burned} symbol={a.symbol} />
    ),
  },
  {
    key: "fees",
    header: t("bridge.activity.fees"),
    render: (a) =>
      a.burn_fees == null || a.burn_fees === "0" ? (
        <span className="text-mute">—</span>
      ) : (
        <span
          className="font-mono"
          title={t("bridge.activity.feesTitle")}
        >
          {fmt.bridgeUnits(a.burn_fees, a.symbol)}
        </span>
      ),
  },
  {
    key: "outstanding",
    header: t("bridge.activity.outstanding"),
    render: (a) => (
      <span className="font-mono text-strong">
        {fmt.bridgeUnits(a.outstanding, a.symbol)}
      </span>
    ),
  },
  {
    key: "locked",
    header: t("bridge.activity.locked"),
    render: (a) =>
      a.locked === null ? (
        <span className="text-mute">—</span>
      ) : (
        <span className="font-mono">{fmt.bridgeUnits(a.locked, a.symbol)}</span>
      ),
  },
  {
    key: "minted_today",
    header: t("bridge.activity.minted"),
    render: (a) =>
      a.minted_today === null && a.mint_cap_per_day === null ? (
        <span className="text-mute">—</span>
      ) : (
        <span className="flex flex-col">
          <span className="font-mono">
            {fmt.bridgeUnits(a.minted_today, a.symbol)}
            {a.mint_cap_per_day !== null && (
              <span className="text-mute"> / {fmt.bridgeUnits(a.mint_cap_per_day, a.symbol)}</span>
            )}
          </span>
          <span className="text-xs text-mute">
            {typeof a.mint_window_secs === "number"
              ? t("bridge.activity.inLast", { window: fmt.mintWindow(a.mint_window_secs) })
              : t("bridge.activity.today")}
          </span>
        </span>
      ),
  },
  {
    key: "headroom",
    header: t("bridge.activity.headroom"),
    render: (a) =>
      a.mint_headroom == null ? (
        <span className="text-mute" title={t("bridge.activity.headroomMissing")}>—</span>
      ) : (
        <span
          className="font-mono"
          title={t("bridge.activity.headroomTitle")}
        >
          {fmt.bridgeUnits(a.mint_headroom, a.symbol)}
        </span>
      ),
  },
  {
    key: "last",
    header: t("bridge.activity.last"),
    render: (a) =>
      a.last_height === null ? (
        <span className="text-mute">—</span>
      ) : (
        <L
          href={`/blocks/${a.last_height}`}
          className="font-mono text-soft hover:text-strong"
        >
          #{fmt.number(a.last_height)}
        </L>
      ),
  },
];

function SourceChainPanel({
  chain,
  name,
  emitter,
  endpoint,
  assets,
}: {
  chain: number;
  name: string;
  emitter: string | undefined;
  endpoint: BridgeEndpoint | undefined;
  assets: BridgeAssetActivity[];
}) {
  const tr = useT();
  const { t } = tr;
  const fmt = useFmt();
  // Burns name their coin, so they add up per source chain. Deposits do only when every row here
  // is its token's one backing; otherwise this chain's share of them is not public.
  const deposits = assets.every((a) => a.deposits !== null)
    ? assets.reduce((n, a) => n + (a.deposits ?? 0), 0)
    : null;
  const burns = assets.reduce((n, a) => n + a.burns, 0);
  return (
    <section className="space-y-3">
      <div className="flex flex-wrap items-baseline justify-between gap-x-6 gap-y-1">
        <h3 className="text-lg text-strong">{name}</h3>
        <p className="text-xs text-mute">
          {emitter ? (
            <>
              {t("bridge.source.contract")}{" "}
              <EndpointAddress endpoint={endpoint} emitter={emitter} className="text-xs" />
            </>
          ) : (
            t("bridge.source.noContract")
          )}
          {typeof endpoint?.min_inbound_sequence === "number" && (
            <span title={t("bridge.floorTitle")}>
              {" · "}
              {t("bridge.source.floor", { sequence: fmt.number(endpoint.min_inbound_sequence) })}
            </span>
          )}
          {assets.length > 0 && (
            <>
              {" · "}
              {deposits !== null
                ? t("bridge.source.inOut", { in: fmt.number(deposits), out: fmt.number(burns) })
                : t("bridge.source.out", { out: fmt.number(burns) })}
            </>
          )}
        </p>
      </div>
      <DataTable
        columns={activityColumns(tr, fmt)}
        data={assets}
        keyExtractor={backingKey}
        emptyMessage={t("bridge.source.empty", { name, chain })}
      />
    </section>
  );
}

// ---------------------------------------------------------------------------
// Approved tokens
// ---------------------------------------------------------------------------

function StatusBadge({ status }: { status: ApprovedToken["status"] }) {
  const { t } = useT();
  return (
    <span
      className={cn(
        "badge",
        status === "allowed" ? "badge-bridge" : "badge-neutral line-through",
      )}
    >
      {status === "allowed" ? t("bridge.tokens.allowed") : t("bridge.tokens.discontinued")}
    </span>
  );
}

const tokenColumns = ({ t }: T): Column<ApprovedToken>[] => [
  {
    key: "token",
    header: t("bridge.tokens.token"),
    render: (tk) => (
      <span className="flex flex-col">
        <span className="font-medium text-strong">{tk.symbol}</span>
        <span className="text-xs text-mute">{tk.name}</span>
      </span>
    ),
  },
  {
    key: "chain",
    header: t("bridge.tokens.chain"),
    render: (tk) => (
      <span className="flex flex-col">
        <span>{tk.chain_name}</span>
        <span className="text-xs text-mute">{tk.standard}</span>
      </span>
    ),
  },
  {
    key: "address",
    header: t("bridge.tokens.address"),
    className: "min-w-[22rem]",
    render: (tk) => (
      <span className="inline-flex items-center gap-1.5">
        <a
          href={tk.explorer_url}
          target="_blank"
          rel="noopener noreferrer"
          className="break-all font-mono text-sm text-text hover:text-strong hover:underline"
          title={t("bridge.openOn", { chain: tk.chain_name })}
        >
          {tk.address}
        </a>
        <CopyButton value={tk.address} />
      </span>
    ),
  },
  {
    key: "decimals",
    header: t("bridge.tokens.decimals"),
    render: (tk) => (
      <span className="flex flex-col font-mono">
        <span>{tk.decimals}</span>
        <span className="text-xs text-mute">{t("bridge.tokens.onRand")}</span>
      </span>
    ),
  },
  {
    key: "status",
    header: t("bridge.tokens.status"),
    render: (tk) => <StatusBadge status={tk.status} />,
  },
];

function ApprovedTokens({ tokens }: { tokens: ApprovedToken[] }) {
  const tr = useT();
  const { t } = tr;
  const notes = tokens.filter((tk) => tk.note);
  return (
    <section className="space-y-4">
      <h2 className="chip">{t("bridge.tokens.title")}</h2>
      <p className="text-sm text-soft">{t("bridge.tokens.intro")}</p>
      <DataTable
        columns={tokenColumns(tr)}
        data={tokens}
        keyExtractor={(tk) => `${tk.chain}-${tk.symbol}`}
        emptyMessage={t("bridge.tokens.empty")}
      />
      {notes.length > 0 && (
        <ul className="space-y-1 text-xs text-mute">
          {notes.map((tk) => (
            <li key={`${tk.chain}-${tk.symbol}`}>
              <span className="text-soft">
                {t("bridge.tokens.noteOn", { symbol: tk.symbol, chain: tk.chain_name })}
              </span>{" "}
              {tk.note}
            </li>
          ))}
        </ul>
      )}
      <p className="text-xs text-mute">{t("bridge.tokens.decimalsNote")}</p>
    </section>
  );
}

// ---------------------------------------------------------------------------
// Registry (the node's raw view)
// ---------------------------------------------------------------------------

const registryColumns = (
  { t }: T,
  fmt: Fmt,
): Column<BridgeAssetActivity | BridgeAsset>[] => [
  {
    key: "index",
    header: t("bridge.registry.index"),
    render: (a) => <span className="font-mono">#{fmt.number(a.index)}</span>,
  },
  {
    key: "chain",
    header: t("bridge.registry.sourceChain"),
    render: (a) => <span>{fmt.bridgeChain(a.chain, knownBridgeChainName(a.chain))}</span>,
  },
  {
    key: "symbol",
    header: t("bridge.registry.token"),
    render: (a) =>
      "symbol" in a && a.symbol ? (
        <span>{a.symbol}</span>
      ) : (
        <span className="text-mute">{t("bridge.registry.unknown")}</span>
      ),
  },
  {
    key: "token",
    header: t("bridge.registry.tokenAddress"),
    render: (a) => <Hash value={a.token} start={10} end={8} />,
  },
  {
    key: "asset_id",
    header: t("bridge.registry.assetId"),
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
  const tr = useT();
  const { t, rich } = tr;
  const fmt = useFmt();

  if (isLoading && !bridge) {
    return <DetailSkeleton />;
  }

  if (error || !bridge) {
    return (
      <>
        <PageHeader title={t("bridge.title")} />
        <ErrorState
          message={t("bridge.loadError")}
          onRetry={() => void mutate()}
        />
      </>
    );
  }

  const emitters = Object.entries(bridge.emitters).sort(
    ([a], [b]) => Number(a) - Number(b),
  );
  const endpointOf = (chain: number | string) =>
    bridge.endpoints?.find((e) => e.chain === Number(chain));
  const assets = activity ?? [];
  const byChain = new Map<number, BridgeAssetActivity[]>();
  for (const a of assets) {
    byChain.set(a.chain, [...(byChain.get(a.chain) ?? []), a]);
  }
  // Any chain the registry names that is not one of the four we list (should not happen).
  const otherChains = [...byChain.keys()].filter(
    (id) => !BRIDGE_SOURCE_CHAINS.some((c) => c.id === id),
  );
  const { deposits, burns } = bridgeTokenTotals(assets);
  const activeChains = BRIDGE_SOURCE_CHAINS.filter(
    (c) => (byChain.get(c.id) ?? []).length > 0,
  );
  const allowed = tokens?.filter((t) => t.status === "allowed").length;

  return (
    <div className="space-y-10">
      <PageHeader
        title={t("bridge.title")}
        subtitle={t("bridge.subtitle")}
      />

      {!bridge.enabled && (
        <Panel>
          <p className="py-4 text-sm text-mute">
            {t("bridge.noSection", { chain: stats?.chain_id ?? "" })}
          </p>
        </Panel>
      )}

      <StatsRow columns={4}>
        <StatsCard
          title={t("bridge.stats.in")}
          value={fmt.number(deposits)}
          subtitle={t("bridge.stats.inSub")}
        />
        <StatsCard
          title={t("bridge.stats.out")}
          value={fmt.number(burns)}
          subtitle={t("bridge.stats.outSub")}
        />
        <StatsCard
          title={t("bridge.stats.assets")}
          value={fmt.number(assets.length)}
          subtitle={
            activeChains.length === 0
              ? t("bridge.stats.noActive")
              : t("bridge.stats.from", { chains: activeChains.map((c) => c.name).join(", ") })
          }
        />
        <StatsCard
          title={t("bridge.stats.approved")}
          value={allowed === undefined ? "—" : fmt.number(allowed)}
          subtitle={t("bridge.stats.approvedSub")}
        />
      </StatsRow>

      <section className="space-y-6">
        <h2 className="chip">{t("bridge.byChain")}</h2>
        {BRIDGE_SOURCE_CHAINS.map((c) => (
          <SourceChainPanel
            key={c.id}
            chain={c.id}
            name={c.name}
            emitter={bridge.emitters[String(c.id)]}
            endpoint={endpointOf(c.id)}
            assets={byChain.get(c.id) ?? []}
          />
        ))}
        {otherChains.map((id) => (
          <SourceChainPanel
            key={id}
            chain={id}
            name={knownBridgeChainName(id) ?? fmt.bridgeChain(id, null)}
            emitter={bridge.emitters[String(id)]}
            endpoint={endpointOf(id)}
            assets={byChain.get(id) ?? []}
          />
        ))}
      </section>

      {tokens && <ApprovedTokens tokens={tokens} />}

      {bridge.enabled && (
        <>
          <Panel title={t("bridge.state.title")}>
            <DetailRow label={t("bridge.state.mintPause")}>
              {bridge.mint_paused ? (
                <span className="badge badge-neutral">
                  {t("bridge.state.paused")}
                </span>
              ) : (
                <span className="badge badge-bridge">{t("bridge.state.notPaused")}</span>
              )}
            </DetailRow>
            <DetailRow label={t("bridge.state.pauseNonce")}>
              <span className="font-mono">
                {bridge.pause_nonce === null ? "—" : fmt.number(bridge.pause_nonce)}
              </span>
            </DetailRow>
            <DetailRow label={t("bridge.state.listNonce")}>
              <span className="font-mono">
                {bridge.list_nonce === null ? "—" : fmt.number(bridge.list_nonce)}
              </span>
            </DetailRow>
            <DetailRow label={t("bridge.state.registrationFee")}>
              <span className="font-mono">
                {bridge.registration_fee === null ? "—" : fmt.amount(bridge.registration_fee)}
              </span>
            </DetailRow>
            <DetailRow label={t("bridge.state.pqGuardians", { count: fmt.number(bridge.pq_guardians.length) })}>
              {bridge.pq_guardians.length === 0 ? (
                <span className="text-mute">—</span>
              ) : (
                <span className="text-soft">
                  {tr.tp("bridge.state.pqGuardiansText", bridge.pq_guardians.length)}
                </span>
              )}
            </DetailRow>
            <DetailRow label={t("bridge.state.pauseKey")}>
              {bridge.pause_key ? (
                <Hash value={bridge.pause_key} start={10} end={8} />
              ) : (
                <span className="text-mute">—</span>
              )}
            </DetailRow>
            <DetailRow label={t("bridge.state.outboundEmitter")}>
              <Hash value={bridge.emitter} full />
            </DetailRow>
            <DetailRow label={t("bridge.state.guardianSet")}>
              <span className="font-mono">
                {bridge.guardian_set_index === null
                  ? "—"
                  : `#${fmt.number(bridge.guardian_set_index)}`}
              </span>
            </DetailRow>
            <DetailRow label={t("bridge.state.guardians", { count: fmt.number(bridge.guardians.length) })}>
              {bridge.guardians.length === 0 ? (
                <span className="text-mute">—</span>
              ) : (
                <ul className="space-y-1.5">
                  {bridge.guardians.map((g, i) => (
                    <li key={g} className="flex items-center gap-2">
                      <span className="w-6 flex-shrink-0 text-end font-mono text-xs text-mute">
                        {i}
                      </span>
                      <Hash value={g} full />
                    </li>
                  ))}
                </ul>
              )}
            </DetailRow>
            <DetailRow label={t("bridge.state.messagesEmitted")}>
              <span className="font-mono">
                {bridge.burn_sequence === null
                  ? "—"
                  : fmt.number(bridge.burn_sequence)}
              </span>
            </DetailRow>
            <DetailRow label={t("bridge.state.nextIndex")}>
              <span className="font-mono">
                {bridge.next_index === null
                  ? "—"
                  : fmt.number(bridge.next_index)}
              </span>
            </DetailRow>
            {bridge.fees && (
              <DetailRow label={t("bridge.state.fees")}>
                <span className="font-mono">
                  {t("bridge.state.feeRates", {
                    in: fmt.percentage(bridge.fees.mint_bps / 100, 2),
                    out: fmt.percentage(bridge.fees.burn_bps / 100, 2),
                  })}
                </span>{" "}
                <span className="text-soft">
                  {t("bridge.state.feeBps", {
                    in: fmt.number(bridge.fees.mint_bps),
                    out: fmt.number(bridge.fees.burn_bps),
                  })}
                </span>{" "}
                <Hash value={bridge.fees.recipient} start={12} end={8} />
                {(() => {
                  const seen = new Set<number>();
                  let dep = 0n;
                  let out = 0n;
                  for (const a of assets) {
                    if (seen.has(a.index)) continue;
                    seen.add(a.index);
                    dep += BigInt(a.token_deposit_fees ?? "0");
                    out += BigInt(a.token_burn_fees ?? "0");
                  }
                  return (
                    <div className="text-xs text-mute">
                      {rich("bridge.state.feesCollected", {
                        deposits: fmt.bridgeUnits(dep.toString()),
                        burns: fmt.bridgeUnits(out.toString()),
                      })}
                    </div>
                  );
                })()}
              </DetailRow>
            )}
            {bridge.rotation_rules && (
              <DetailRow label={t("bridge.state.rotationRules")}>
                <span className="text-soft">
                  {t(
                    bridge.rotation_rules.needs_possession
                      ? "bridge.state.rotationDelayPossession"
                      : "bridge.state.rotationDelay",
                    { delay: fmt.mintWindow(bridge.rotation_rules.delay_secs) },
                  )}
                </span>
                {bridge.pending_rotations && bridge.pending_rotations.length > 0 ? (
                  <ul className="mt-1 space-y-0.5 text-xs">
                    {bridge.pending_rotations.map((r, i) => (
                      <li key={i}>
                        {rich(
                          r.kind === "pause_key"
                            ? "bridge.state.pendingPauseKey"
                            : "bridge.state.pendingPq",
                          { at: new Date(r.effective_at_secs * 1000).toISOString() },
                        )}
                      </li>
                    ))}
                  </ul>
                ) : (
                  <div className="text-xs text-mute">{t("bridge.state.noPending")}</div>
                )}
              </DetailRow>
            )}
            {bridge.rotation_nonce != null && (
              <DetailRow label={t("bridge.state.rotationNonce")}>
                <span className="font-mono">{fmt.number(bridge.rotation_nonce)}</span>
              </DetailRow>
            )}
            {bridge.rules_v2 && (
              <DetailRow label={t("bridge.state.mintCapAll")}>
                <span className="font-mono">
                  {fmt.bridgeUnits(bridge.rules_v2.global_mint_cap_per_window)}
                </span>{" "}
                <span className="text-soft">
                  {t("bridge.state.mintCapPer", {
                    window: fmt.mintWindow(bridge.rules_v2.cap_window_secs),
                  })}
                </span>
                {bridge.rules_v2.global_minted_in_window != null &&
                  bridge.rules_v2.global_mint_headroom != null && (
                    <div className="text-xs text-mute">
                      {rich("bridge.state.mintedInWindow", {
                        minted: fmt.bridgeUnits(bridge.rules_v2.global_minted_in_window),
                        room: fmt.bridgeUnits(bridge.rules_v2.global_mint_headroom),
                      })}
                    </div>
                  )}
              </DetailRow>
            )}
            <DetailRow label={t("bridge.state.trustedEmitters", { count: fmt.number(emitters.length) })}>
              {emitters.length === 0 ? (
                <span className="text-mute">—</span>
              ) : (
                <ul className="space-y-3">
                  {emitters.map(([chain, word]) => {
                    const endpoint = endpointOf(chain);
                    const floor = endpoint?.min_inbound_sequence;
                    return (
                      <li key={chain} className="space-y-0.5">
                        <div className="flex flex-wrap items-center gap-x-2 gap-y-0.5">
                          <span className="text-soft">
                            {fmt.bridgeChain(Number(chain), knownBridgeChainName(Number(chain)))}
                          </span>
                          <EndpointAddress endpoint={endpoint} emitter={word} />
                        </div>
                        {(endpoint?.address || typeof floor === "number") && (
                          <div className="flex flex-wrap items-center gap-x-4 text-xs text-mute">
                            {endpoint?.address && (
                              <span className="inline-flex items-center gap-x-2">
                                {t("bridge.state.asStored")}
                                <Hash value={word} start={26} end={8} className="text-xs" />
                              </span>
                            )}
                            {typeof floor === "number" && (
                              <span title={t("bridge.floorTitle")}>
                                {t("bridge.state.floorMinted", { sequence: fmt.number(floor) })}
                              </span>
                            )}
                          </div>
                        )}
                      </li>
                    );
                  })}
                </ul>
              )}
            </DetailRow>
          </Panel>

          <section className="space-y-4">
            <h2 className="chip">{t("bridge.registry.title")}</h2>
            <DataTable
              columns={registryColumns(tr, fmt)}
              data={activity ?? bridge.assets}
              keyExtractor={backingKey}
              emptyMessage={t("bridge.registry.empty")}
            />
            <p className="text-xs text-mute">{t("bridge.registry.note")}</p>
          </section>
        </>
      )}
    </div>
  );
}
