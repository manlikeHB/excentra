export interface UserResponse {
  id: string
  username: string | null
  email: string
  role: 'user' | 'admin'
  is_suspended: boolean
  created_at: string
  updated_at: string
}

export interface BalanceResponse {
  asset: string
  available: string
  held: string
  updated_at: string
}

export interface TickerResponse {
  symbol: string
  last_price: string
  high_24h: string
  low_24h: string
  volume_24h: string
  price_change_pct: string
}

export interface UserTradeResponse {
  id: string;
  symbol: string;
  side: "buy" | "sell";
  price: string;
  quantity: string;
  total: string;
  created_at: string;
}

export interface PriceLevel {
  price: string;
  quantity: string;
}

export interface OrderBookResponse {
  symbol: string;
  bids: PriceLevel[];
  asks: PriceLevel[];
}

export interface OrderBookUpdateData {
  snapshot: {
    bids: PriceLevel[];
    asks: PriceLevel[];
  };
}

export interface PairResponse {
  id: string
  symbol: string
  base_asset: string
  quote_asset: string
  is_active: boolean
}

export interface OrderResponse {
  id: string
  symbol: string
  side: 'buy' | 'sell'
  order_type: 'limit' | 'market'
  price: string | null
  quantity: string
  remaining_quantity: string
  status: 'open' | 'partially_filled' | 'filled' | 'cancelled'
  created_at: string
  updated_at: string
}

export interface TradeResponse {
  id: string
  symbol: string
  side: 'buy' | 'sell'
  price: string
  quantity: string
  created_at: string
}

export interface PaginatedResponse<T> {
  data: T[]
  page: number
  limit: number
  total: number
}

export interface AdminStats {
  total_users: number
  active_ws_connections: number
  orders_processed: number
  uptime_seconds: number
  pair_volumes: { symbol: string; volume_24h: string }[]
}

export interface ErrorResponse {
  error: string
}

export interface AssetResponse {
  symbol: string
  name: string
}

// WebSocket message types
export interface WsAuthMessage {
  action: 'auth'
  token: string
}

export interface WsSubscribeMessage {
  action: 'subscribe' | 'unsubscribe'
  channel: string
}

export type WsInbound = WsAuthMessage | WsSubscribeMessage

export interface WsSubscribedEvent {
  type: 'subscribed'
  channel: string
}

export interface WsAuthenticatedEvent {
  type: 'authenticated'
}

export interface WsErrorEvent {
  type: 'error'
  message: string
}

export interface TradeEventData {
  symbol: string;
  price: string;
  side: "buy" | "sell"
  quantity: string;
  created_at: string;
}

export interface TickerUpdateData {
  symbol: string
  last_price: string
  high_24h: string
  low_24h: string
  volume_24h: string
  price_change_pct: string
}

export interface OrderStatusUpdateData {
  user_id: string
  order_id: string
  status: string
  quantity: string
  remaining_quantity: string
}

export type WsEventData =
  | { OrderBookUpdate: OrderBookUpdateData }
  | { TradeEvent: TradeEventData }
  | { TickerUpdate: TickerUpdateData }
  | { OrderStatusUpdate: OrderStatusUpdateData }

export interface WsEventMessage {
  type: 'event'
  data: WsEventData
}

export type WsOutbound = WsSubscribedEvent | WsAuthenticatedEvent | WsErrorEvent | WsEventMessage

// Candle type for chart
export interface Candle {
  time: number
  open: number
  high: number
  low: number
  close: number
}
