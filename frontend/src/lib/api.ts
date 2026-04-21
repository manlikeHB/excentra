import { toast } from 'sonner'

const BASE_URL = process.env.NEXT_PUBLIC_API_URL ?? 'http://localhost:5098/api/v1'

let accessToken: string | null = null
let refreshPromise: Promise<string | null> | null = null
let onRefreshFail: (() => void) | null = null

export function setAccessToken(token: string | null) {
  accessToken = token
}

export function getAccessToken(): string | null {
  return accessToken
}

export function setOnRefreshFail(cb: () => void) {
  onRefreshFail = cb
}

async function doRefresh(): Promise<string | null> {
  try {
    const res = await fetch(`${BASE_URL}/auth/refresh`, {
      method: 'POST',
      credentials: 'include',
    })
    if (!res.ok) {
      return null
    }
    const data = await res.json()
    accessToken = data.access_token
    return accessToken
  } catch {
    return null
  }
}

async function refreshToken(): Promise<string | null> {
  if (refreshPromise) return refreshPromise
  refreshPromise = doRefresh().finally(() => {
    refreshPromise = null
  })
  return refreshPromise
}

async function request<T>(
  path: string,
  options: RequestInit = {},
  retry = true,
): Promise<T> {
  const headers: Record<string, string> = {
    "Content-Type": "application/json",
    ...(options.headers as Record<string, string>),
  };

  if (accessToken) {
    headers["Authorization"] = `Bearer ${accessToken}`;
  }

  let res: Response;
  try {
    res = await fetch(`${BASE_URL}${path}`, {
      ...options,
      headers,
      credentials: "include",
    });
  } catch (err) {
    const msg =
      err instanceof TypeError
        ? `Cannot reach backend at ${BASE_URL}. Check that it is running and CORS is configured for ${window.location.origin}.`
        : "Network error";
    throw new Error(msg);
  }

  if (res.status === 401 && retry) {
    const newToken = await refreshToken();
    if (newToken) {
      return request<T>(path, options, false);
    } else {
      let message = "Unauthorized";
      try {
        const errData = await res.json();
        message = errData.error ?? message;
      } catch {
        // body not parseable, keep default
      }
      onRefreshFail?.();
      throw new Error(message);
    }
  }

  if (!res.ok) {
    let errorMessage = `Request failed: ${res.status}`;
    try {
      const errData = await res.json();
      errorMessage = errData.error || errorMessage;
    } catch {
      // ignore parse errors
    }
    throw new Error(errorMessage);
  }

  if (res.status === 204) {
    return undefined as unknown as T;
  }

  return res.json();
}

// Auth
export const authApi = {
  login: (email: string, password: string) =>
    request<{ access_token: string }>('/auth/login', {
      method: 'POST',
      body: JSON.stringify({ email, password }),
    }),

  register: (email: string, password: string) =>
    request<{ access_token: string }>('/auth/register', {
      method: 'POST',
      body: JSON.stringify({ email, password }),
    }),

  refresh: () =>
    request<{ access_token: string }>('/auth/refresh', { method: 'POST' }),

  logout: () =>
    request<void>('/auth/logout', { method: 'POST' }),

  forgotPassword: (email: string) =>
    request<void>('/auth/forgot-password', {
      method: 'POST',
      body: JSON.stringify({ email }),
    }),

  resetPassword: (token: string, new_password: string) =>
    request<void>('/auth/reset-password', {
      method: 'POST',
      body: JSON.stringify({ token, new_password }),
    }),
}

// Market data (public)
export const marketApi = {
  getActivePairs: () => request<import('./types').PairResponse[]>('/pairs/active'),

  getPair: (symbol: string) =>
    request<import('./types').PairResponse>(`/pairs/${symbol}`),

  getOrderBook: (symbol: string) =>
    request<import('./types').OrderBookResponse>(`/orderbook/${symbol}`),

  getTrades: (symbol: string, limit = 50) =>
    request<import('./types').TradeResponse[]>(`/trades/${symbol}?limit=${limit}`),

  getTicker: (symbol?: string) =>
    symbol
      ? request<import('./types').TickerResponse>(`/ticker/${symbol}`)
      : request<import('./types').TickerResponse[]>('/ticker'),
}

// Orders (auth required)
export const ordersApi = {
  place: (order: {
    symbol: string;
    side: "buy" | "sell";
    order_type: "limit" | "market";
    price?: string;
    quantity: string;
  }) =>
    request<import("./types").OrderResponse>("/orders", {
      method: "POST",
      body: JSON.stringify(order),
    }),

  list: (
    params: {
      status?: string;
      pair?: string;
      page?: number;
      limit?: number;
      order?: string;
    } = {},
  ) => {
    const qs = new URLSearchParams();
    if (params.status) qs.set("status", params.status);
    if (params.pair) qs.set("pair", params.pair);
    if (params.page) qs.set("page", String(params.page));
    if (params.limit) qs.set("limit", String(params.limit));
    if (params.order) qs.set("order", params.order);
    return request<
      import("./types").PaginatedOrderResponse
    >(`/orders?${qs}`);
  },

  get: (id: string) => request<import("./types").OrderResponse>(`/orders/${id}`),

  cancel: (id: string) => request<void>(`/orders/${id}`, { method: "DELETE" }),
};

// Balances (auth required)
export const balancesApi = {
  list: () =>
    request<import('./types').BalanceResponse[]>('/balances'),

  get: (asset: string) =>
    request<import('./types').BalanceResponse>(`/balances/${asset}`),

  deposit: (asset: string, amount: string) =>
    request<import('./types').BalanceResponse>('/balances/deposit', {
      method: 'POST',
      body: JSON.stringify({ asset, amount }),
    }),

  withdraw: (asset: string, amount: string) =>
    request<import('./types').BalanceResponse>('/balances/withdraw', {
      method: 'POST',
      body: JSON.stringify({ asset, amount }),
    }),
}

// Trades (auth required)
export const tradesApi = {
  mine: (params: { pair?: string; page?: number; limit?: number; order?: string } = {}) => {
    const qs = new URLSearchParams()
    if (params.pair) qs.set('pair', params.pair)
    if (params.page) qs.set('page', String(params.page))
    if (params.limit) qs.set('limit', String(params.limit))
    if (params.order) qs.set('order', params.order)
    return request<
      import("./types").PaginatedUserTradeResponse
    >(`/trades/me?${qs}`);
  },
}

// Users (auth required)
export const usersApi = {
  me: () => request<import('./types').UserResponse>('/users/me'),

  update: (data: { username?: string; current_password?: string; new_password?: string }) =>
    request<import('./types').UserResponse>('/users/me', {
      method: 'PATCH',
      body: JSON.stringify(data),
    }),
}

// Admin (admin only)
export const adminApi = {
  getUsers: () => request<import("./types").PaginatedUserSummary>("/admin/users"),

  updateRole: (id: string, role: "user" | "admin") =>
    request<import("./types").UserResponse>(`/admin/users/${id}/role`, {
      method: "PATCH",
      body: JSON.stringify({ role }),
    }),

  suspend: (id: string) =>
    request<import("./types").UserResponse>(`/admin/users/${id}/suspend`, {
      method: "PATCH",
    }),

  getStats: () => request<import("./types").AdminStats>("/admin/stats"),

  getAllPairs: () => request<import("./types").PairResponse[]>("/pairs"),

  createPair: (base_asset: string, quote_asset: string) =>
    request<import("./types").PairResponse>("/pairs", {
      method: "POST",
      body: JSON.stringify({ base_asset, quote_asset }),
    }),

  getAssets: () => request<import("./types").AssetResponse[]>("/assets"),

  createAsset: (symbol: string, name: string, decimals: number) =>
    request<import("./types").AssetResponse>("/assets", {
      method: "POST",
      body: JSON.stringify({ symbol, name, decimals }),
    }),
};
