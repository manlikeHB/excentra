export const toPathSymbol = (s: string) => s.replace('/', '-')    // BTC/USDT → BTC-USDT
export const toDisplaySymbol = (s: string) => s.replace('-', '/')  // BTC-USDT → BTC/USDT

export function formatDecimal(value: string | number, decimals: number): string {
  const n = typeof value === 'string' ? parseFloat(value) : value
  if (isNaN(n)) return '0'
  return n.toLocaleString('en-US', {
    minimumFractionDigits: decimals,
    maximumFractionDigits: decimals,
  })
}

export function formatPrice(value: string | number, asset: string): string {
  const decimals: Record<string, number> = {
    USDT: 2,
    BTC: 6,
    ETH: 4,
    SOL: 4,
  }
  return formatDecimal(value, decimals[asset] ?? 4)
}

export function formatCompact(value: string | number): string {
  const n = typeof value === 'string' ? parseFloat(value) : value
  if (isNaN(n)) return '0'
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(2)}M`
  if (n >= 1_000) return `${(n / 1_000).toFixed(2)}K`
  return n.toFixed(2)
}
