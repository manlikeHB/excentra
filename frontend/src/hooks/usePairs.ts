import { useQuery } from '@tanstack/react-query'
import { marketApi } from '@/lib/api'
import { PairResponse } from '@/lib/types'

export function usePairs() {
  return useQuery<PairResponse[]>({
    queryKey: ['pairs', 'active'],
    queryFn: () => marketApi.getActivePairs(),
    staleTime: 30_000,
  })
}

export function usePair(symbol: string | undefined) {
  const { data: pairs } = usePairs()
  if (!symbol || !pairs) return undefined
  return pairs.find((p) => p.symbol === symbol || p.symbol.replace('/', '-') === symbol)
}
