// Backend-derived types are re-exported from ./generated/api-types,
// which is generated from the Rust OpenAPI spec.
// Regenerate via `npm run gen:api` from the frontend directory.
// Do not edit ./generated/api-types.ts by hand.

import type { components } from './generated/api-types'

type S = components['schemas']

// — REST response types —
export type UserResponse = S['UserResponse']
export type BalanceResponse = S['BalanceResponse']
export type TickerResponse = S['TickerResponse']
export type OrderResponse = S['OrderResponse']
export type TradeResponse = S['TradeResponse']
export type UserTradeResponse = S['UserTradeResponse']
export type OrderBookResponse = S['OrderBookResponse']
export type PairResponse = S['TradingPairsResponse']
export type AssetResponse = S['Asset']
export type AdminStats = S['AdminStats']
export type PriceLevel = S['PriceLevel']

// — Paginated types —
export type PaginatedOrderResponse = S['PaginatedResponse_OrderResponse']
export type PaginatedUserTradeResponse = S['PaginatedResponse_UserTradeResponse']
export type PaginatedUserSummary = S['PaginatedResponse_UserSummary']

// — WebSocket types —
export type WsEvent = S['WsEvent']
export type InboundMessage = S['InboundMessage']
export type OutboundMessage = S['OutboundMessage']
export type OrderBookSnapshot = S['OrderBookSnapshot']

// — Frontend-only types (no backend equivalent) —

export interface Candle {
  time: number
  open: number
  high: number
  low: number
  close: number
}

export interface ErrorResponse {
  error: string
}
