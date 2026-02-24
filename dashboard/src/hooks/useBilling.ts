import { useCallback, useEffect, useState } from 'react';
import { fetchBilling } from '../api';
import type { BillingSummary } from '../types';

interface UseBillingResult {
  summary: BillingSummary | null;
  loading: boolean;
  error: string | null;
  refresh: () => void;
}

/**
 * Hook that fetches the pay-as-you-go billing summary for a user.
 * Re-fetches every `pollIntervalMs` milliseconds.
 */
export function useBilling(
  userId: number,
  pollIntervalMs = 15_000,
): UseBillingResult {
  const [summary, setSummary] = useState<BillingSummary | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const load = useCallback(async () => {
    setLoading(true);
    try {
      const data = await fetchBilling(userId);
      setSummary(data);
      setError(null);
    } catch (e) {
      setError((e as Error).message);
    } finally {
      setLoading(false);
    }
  }, [userId]);

  useEffect(() => {
    void load();
    const timer = setInterval(() => void load(), pollIntervalMs);
    return () => clearInterval(timer);
  }, [load, pollIntervalMs]);

  return { summary, loading, error, refresh: load };
}
