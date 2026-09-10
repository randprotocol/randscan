import type {
  BlockSummary,
  ClientMessage,
  NetworkStats,
  ServerMessage,
  TransactionSummary,
  WebSocketChannel,
} from '@/types';

type BlockHandler = (block: BlockSummary) => void;
type TransactionHandler = (tx: TransactionSummary) => void;
type StatsHandler = (stats: NetworkStats) => void;
type StatusHandler = (connected: boolean) => void;

/**
 * Resolve the WebSocket endpoint. `NEXT_PUBLIC_WS_URL` wins when set; otherwise
 * the URL is derived from the page origin (`ws://host/ws` / `wss://host/ws`).
 */
export function resolveWebSocketUrl(): string | null {
  const configured = process.env.NEXT_PUBLIC_WS_URL;
  if (configured && configured.trim() !== '') {
    const base = configured.trim().replace(/\/+$/, '');
    return base.endsWith('/ws') ? base : `${base}/ws`;
  }

  if (typeof window === 'undefined') return null;

  const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
  return `${protocol}//${window.location.host}/ws`;
}

const CHANNELS: WebSocketChannel[] = ['blocks', 'transactions', 'stats'];

const INITIAL_RECONNECT_DELAY_MS = 1000;
const MAX_RECONNECT_DELAY_MS = 30_000;
const PING_INTERVAL_MS = 30_000;

/**
 * Single shared connection to `/ws`. Subscribers register per channel; the
 * client subscribes on the wire when the first listener for a channel appears
 * and unsubscribes when the last one goes away.
 */
class RandScanWebSocket {
  private ws: WebSocket | null = null;
  private connected = false;
  private closedByUser = false;
  private reconnectAttempts = 0;
  private reconnectTimer: ReturnType<typeof setTimeout> | null = null;
  private pingTimer: ReturnType<typeof setInterval> | null = null;

  private blockHandlers = new Set<BlockHandler>();
  private transactionHandlers = new Set<TransactionHandler>();
  private statsHandlers = new Set<StatsHandler>();
  private statusHandlers = new Set<StatusHandler>();

  get isConnected(): boolean {
    return this.connected;
  }

  // -- connection ----------------------------------------------------------

  connect(): void {
    if (typeof window === 'undefined') return;
    if (this.ws && (this.ws.readyState === WebSocket.OPEN || this.ws.readyState === WebSocket.CONNECTING)) {
      return;
    }

    const url = resolveWebSocketUrl();
    if (!url) return;

    this.closedByUser = false;

    let socket: WebSocket;
    try {
      socket = new WebSocket(url);
    } catch {
      this.scheduleReconnect();
      return;
    }

    this.ws = socket;

    socket.onopen = () => {
      this.connected = true;
      this.reconnectAttempts = 0;
      this.emitStatus(true);
      for (const channel of CHANNELS) {
        if (this.hasListeners(channel)) this.send({ type: 'subscribe', channel });
      }
      this.startPing();
    };

    socket.onmessage = (event: MessageEvent) => {
      this.handleMessage(event.data);
    };

    socket.onerror = () => {
      // `onclose` always follows; reconnect is handled there.
    };

    socket.onclose = () => {
      this.connected = false;
      this.stopPing();
      this.emitStatus(false);
      if (this.ws === socket) this.ws = null;
      if (!this.closedByUser) this.scheduleReconnect();
    };
  }

  disconnect(): void {
    this.closedByUser = true;
    this.clearReconnect();
    this.stopPing();
    if (this.ws) {
      this.ws.onopen = null;
      this.ws.onmessage = null;
      this.ws.onerror = null;
      this.ws.onclose = null;
      try {
        this.ws.close();
      } catch {
        // Already closing.
      }
      this.ws = null;
    }
    this.connected = false;
    this.emitStatus(false);
  }

  private scheduleReconnect(): void {
    if (this.closedByUser || this.reconnectTimer) return;

    const delay = Math.min(
      INITIAL_RECONNECT_DELAY_MS * 2 ** this.reconnectAttempts,
      MAX_RECONNECT_DELAY_MS
    );
    this.reconnectAttempts += 1;

    this.reconnectTimer = setTimeout(() => {
      this.reconnectTimer = null;
      this.connect();
    }, delay);
  }

  private clearReconnect(): void {
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }
    this.reconnectAttempts = 0;
  }

  private startPing(): void {
    this.stopPing();
    this.pingTimer = setInterval(() => {
      this.send({ type: 'ping' });
    }, PING_INTERVAL_MS);
  }

  private stopPing(): void {
    if (this.pingTimer) {
      clearInterval(this.pingTimer);
      this.pingTimer = null;
    }
  }

  private send(message: ClientMessage): void {
    if (this.ws?.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(message));
    }
  }

  // -- messages ------------------------------------------------------------

  private handleMessage(raw: unknown): void {
    if (typeof raw !== 'string') return;

    let message: ServerMessage;
    try {
      message = JSON.parse(raw) as ServerMessage;
    } catch {
      return;
    }

    switch (message.type) {
      case 'new_block':
        if (message.block) {
          this.blockHandlers.forEach((handler) => handler(message.block));
        }
        break;
      case 'new_transaction':
        if (message.transaction) {
          this.transactionHandlers.forEach((handler) => handler(message.transaction));
        }
        break;
      case 'stats_update':
        if (message.stats) {
          this.statsHandlers.forEach((handler) => handler(message.stats));
        }
        break;
      case 'subscribed':
      case 'unsubscribed':
      case 'pong':
        break;
      case 'error':
        if (process.env.NODE_ENV !== 'production') {
          console.warn('[ws] server error:', message.message ?? message.error);
        }
        break;
      default:
        break;
    }
  }

  // -- subscriptions -------------------------------------------------------

  private hasListeners(channel: WebSocketChannel): boolean {
    switch (channel) {
      case 'blocks':
        return this.blockHandlers.size > 0;
      case 'transactions':
        return this.transactionHandlers.size > 0;
      case 'stats':
        return this.statsHandlers.size > 0;
      default:
        return false;
    }
  }

  private register<T>(
    channel: WebSocketChannel,
    set: Set<T>,
    handler: T
  ): () => void {
    const wasEmpty = set.size === 0;
    set.add(handler);

    this.connect();
    if (wasEmpty) this.send({ type: 'subscribe', channel });

    return () => {
      set.delete(handler);
      if (set.size === 0) this.send({ type: 'unsubscribe', channel });
    };
  }

  onBlock(handler: BlockHandler): () => void {
    return this.register('blocks', this.blockHandlers, handler);
  }

  onTransaction(handler: TransactionHandler): () => void {
    return this.register('transactions', this.transactionHandlers, handler);
  }

  onStats(handler: StatsHandler): () => void {
    return this.register('stats', this.statsHandlers, handler);
  }

  onStatusChange(handler: StatusHandler): () => void {
    this.statusHandlers.add(handler);
    return () => {
      this.statusHandlers.delete(handler);
    };
  }

  private emitStatus(connected: boolean): void {
    this.statusHandlers.forEach((handler) => handler(connected));
  }
}

let client: RandScanWebSocket | null = null;

export function getWebSocketClient(): RandScanWebSocket {
  if (!client) client = new RandScanWebSocket();
  return client;
}

export { RandScanWebSocket };
