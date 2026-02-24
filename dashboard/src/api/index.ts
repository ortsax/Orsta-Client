// ---------------------------------------------------------------------------
// REST API client
//
// All requests target the Rust/Axum backend. The base URL is read from the
// Vite environment variable VITE_API_BASE_URL (default: http://localhost:3000).
// ---------------------------------------------------------------------------

import type { BillingSummary, Instance, NewInstance } from '../types';

const BASE_URL = import.meta.env.VITE_API_BASE_URL ?? 'http://localhost:3000';

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch(`${BASE_URL}${path}`, {
    headers: { 'Content-Type': 'application/json', ...init?.headers },
    ...init,
  });
  if (!res.ok) {
    const text = await res.text();
    throw new Error(`API error ${res.status}: ${text}`);
  }
  return res.json() as Promise<T>;
}

// ---------------------------------------------------------------------------
// Instances
// ---------------------------------------------------------------------------

export function fetchInstances(userId: number): Promise<Instance[]> {
  return request<Instance[]>(`/instances?user_id=${userId}`);
}

export function createInstance(data: NewInstance): Promise<Instance> {
  return request<Instance>('/instances', {
    method: 'POST',
    body: JSON.stringify(data),
  });
}

export function activateInstance(id: number): Promise<void> {
  return request<void>(`/instances/${id}/activate`, { method: 'PATCH' });
}

export function deactivateInstance(id: number): Promise<void> {
  return request<void>(`/instances/${id}/deactivate`, { method: 'PATCH' });
}

// ---------------------------------------------------------------------------
// Billing
// ---------------------------------------------------------------------------

export function fetchBilling(userId: number): Promise<BillingSummary> {
  return request<BillingSummary>(`/billing?user_id=${userId}`);
}
