'use client';

import { useCallback, useEffect, useRef, useState } from 'react';
import L from 'leaflet';
import 'leaflet/dist/leaflet.css';
import {
  formatConnectedTime,
  formatEndpoint,
  formatLocation,
  getNodeRoleLabel,
  shortenHash,
} from '@/lib/utils';
import type { NodeInfo } from '@/types';

const TILE_DARK =
  'https://server.arcgisonline.com/ArcGIS/rest/services/Canvas/World_Dark_Gray_Base/MapServer/tile/{z}/{y}/{x}';
const TILE_LIGHT =
  'https://server.arcgisonline.com/ArcGIS/rest/services/Canvas/World_Light_Gray_Base/MapServer/tile/{z}/{y}/{x}';
const TILE_ATTRIBUTION =
  'Tiles &copy; <a href="https://www.esri.com/">Esri</a> &mdash; Esri, DeLorme, NAVTEQ';

function isDarkTheme(): boolean {
  if (typeof document === 'undefined') return false;
  return document.documentElement.getAttribute('data-theme') === 'dark';
}

/** Marker colours come from the same CSS variables as the rest of the UI. */
function themeColor(name: string, fallback: string): string {
  if (typeof window === 'undefined') return fallback;
  const value = getComputedStyle(document.documentElement).getPropertyValue(name).trim();
  return value || fallback;
}

interface GeolocatedNode {
  node: NodeInfo;
  lat: number;
  lon: number;
}

/**
 * Nodes in the same datacentre share coordinates exactly. Spread co-located
 * markers around a small circle so each stays clickable.
 */
function spreadOverlapping(nodes: GeolocatedNode[]): GeolocatedNode[] {
  const groups = new Map<string, GeolocatedNode[]>();

  for (const entry of nodes) {
    const key = `${entry.lat.toFixed(3)},${entry.lon.toFixed(3)}`;
    const group = groups.get(key);
    if (group) group.push(entry);
    else groups.set(key, [entry]);
  }

  const out: GeolocatedNode[] = [];
  for (const group of Array.from(groups.values())) {
    if (group.length === 1) {
      out.push(group[0]);
      continue;
    }

    const radius = 0.6 + group.length * 0.05;
    group.forEach((entry, index) => {
      const angle = (2 * Math.PI * index) / group.length;
      out.push({
        ...entry,
        lat: entry.lat + radius * Math.sin(angle),
        lon: entry.lon + radius * Math.cos(angle),
      });
    });
  }

  return out;
}

function escapeHtml(value: string): string {
  return value
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');
}

function popupHtml(node: NodeInfo): string {
  const rows: string[] = [];

  const label = node.is_self
    ? `${getNodeRoleLabel(node.role)} · this node`
    : getNodeRoleLabel(node.role);

  rows.push(
    `<div style="font-weight:500;color:var(--color-text-strong);margin-bottom:3px">${escapeHtml(label)}</div>`
  );
  rows.push(
    `<div style="font-family:var(--font-mono),ui-monospace,monospace;font-size:11px;color:var(--color-text-soft);word-break:break-all" title="${escapeHtml(
      node.peer_id
    )}">${escapeHtml(shortenHash(node.peer_id, 10, 8))}</div>`
  );

  const endpoint = formatEndpoint(node.ip, node.port);
  if (endpoint !== '—') {
    rows.push(
      `<div style="font-family:var(--font-mono),ui-monospace,monospace;font-size:11px;color:var(--color-text-mute)">${escapeHtml(endpoint)}</div>`
    );
  }

  rows.push(
    `<div style="margin-top:5px;color:var(--color-text)">${escapeHtml(formatLocation(node.geo))}</div>`
  );

  if (node.geo?.org) {
    rows.push(`<div style="color:var(--color-text-mute)">${escapeHtml(node.geo.org)}</div>`);
  }

  if (node.connected_secs !== null) {
    rows.push(
      `<div style="margin-top:5px;color:var(--color-text-mute)">Connected ${escapeHtml(
        formatConnectedTime(node.connected_secs)
      )}</div>`
    );
  }

  rows.push(
    `<button type="button" data-copy="${escapeHtml(node.peer_id)}" ` +
      'style="margin-top:9px;border:1px solid var(--color-border);background:var(--color-surface-2);' +
      'color:var(--color-accent);border-radius:3px;padding:3px 8px;font-size:11px;cursor:pointer">Copy peer id</button>'
  );

  return `<div style="min-width:190px;font-size:12px;line-height:1.55">${rows.join('')}</div>`;
}

interface NodeMapProps {
  nodes: NodeInfo[];
}

export default function NodeMap({ nodes }: NodeMapProps) {
  const containerRef = useRef<HTMLDivElement | null>(null);
  const mapRef = useRef<L.Map | null>(null);
  const layerRef = useRef<L.LayerGroup | null>(null);
  const tileRef = useRef<L.TileLayer | null>(null);
  const [dark, setDark] = useState(false);

  // Track the theme so tiles and marker colours follow the toggle.
  useEffect(() => {
    setDark(isDarkTheme());
    const observer = new MutationObserver(() => setDark(isDarkTheme()));
    observer.observe(document.documentElement, {
      attributes: true,
      attributeFilter: ['data-theme'],
    });
    return () => observer.disconnect();
  }, []);

  const colorFor = useCallback((node: NodeInfo) => {
    if (node.is_self) return themeColor('--color-accent-3', '#8c2f39');
    if (node.role === 'validator') return themeColor('--color-accent', '#1b4b8f');
    return themeColor('--color-accent-2', '#4a6fa5');
  }, []);

  // Create the map once.
  useEffect(() => {
    if (!containerRef.current || mapRef.current) return;

    const map = L.map(containerRef.current, {
      center: [20, 0],
      zoom: 2,
      minZoom: 1,
      worldCopyJump: true,
      scrollWheelZoom: false,
      attributionControl: true,
    });

    tileRef.current = L.tileLayer(isDarkTheme() ? TILE_DARK : TILE_LIGHT, {
      attribution: TILE_ATTRIBUTION,
      subdomains: 'abcd',
      maxZoom: 16,
    }).addTo(map);

    layerRef.current = L.layerGroup().addTo(map);
    mapRef.current = map;

    // Popups are raw HTML, so wire the copy button up by delegation.
    const onPopupClick = (event: Event) => {
      const target = (event.target as HTMLElement | null)?.closest('[data-copy]');
      if (!target) return;
      const value = target.getAttribute('data-copy');
      if (!value) return;
      void navigator.clipboard?.writeText(value).then(() => {
        target.textContent = 'Copied';
      });
    };
    map.getContainer().addEventListener('click', onPopupClick);

    return () => {
      map.getContainer().removeEventListener('click', onPopupClick);
      map.remove();
      mapRef.current = null;
      layerRef.current = null;
      tileRef.current = null;
    };
  }, []);

  // Swap the basemap when the theme changes.
  useEffect(() => {
    tileRef.current?.setUrl(dark ? TILE_DARK : TILE_LIGHT);
  }, [dark]);

  // Redraw markers whenever the node set (or theme) changes.
  useEffect(() => {
    const map = mapRef.current;
    const layer = layerRef.current;
    if (!map || !layer) return;

    layer.clearLayers();

    const geolocated: GeolocatedNode[] = nodes
      .filter((node) => node.geo !== null)
      .map((node) => ({
        node,
        lat: (node.geo as NonNullable<NodeInfo['geo']>).lat,
        lon: (node.geo as NonNullable<NodeInfo['geo']>).lon,
      }))
      .filter((entry) => Number.isFinite(entry.lat) && Number.isFinite(entry.lon));

    const positioned = spreadOverlapping(geolocated);

    for (const entry of positioned) {
      const { node } = entry;
      const color = colorFor(node);
      const radius = node.is_self ? 8 : node.role === 'validator' ? 7 : 5;

      L.circleMarker([entry.lat, entry.lon], {
        radius,
        color,
        weight: 1.5,
        fillColor: color,
        fillOpacity: 0.35,
      })
        .bindPopup(popupHtml(node))
        .addTo(layer);
    }

    if (positioned.length >= 2) {
      const bounds = L.latLngBounds(positioned.map((e) => [e.lat, e.lon] as [number, number]));
      map.fitBounds(bounds, { padding: [48, 48], maxZoom: 6 });
    } else {
      map.setView([20, 0], 2);
    }
  }, [nodes, dark, colorFor]);

  return (
    <div className="card overflow-hidden">
      <div ref={containerRef} className="h-[480px] w-full" />
    </div>
  );
}
