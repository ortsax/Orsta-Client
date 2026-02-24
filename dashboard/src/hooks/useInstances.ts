import { useCallback, useEffect, useState } from 'react';
import { activateInstance, createInstance, deactivateInstance, fetchInstances } from '../api';
import type { Instance, NewInstance } from '../types';

interface UseInstancesResult {
  instances: Instance[];
  loading: boolean;
  error: string | null;
  refresh: () => void;
  create: (data: NewInstance) => Promise<void>;
  activate: (id: number) => Promise<void>;
  deactivate: (id: number) => Promise<void>;
}

/**
 * Hook that manages WhatsApp bot instance state for a given user.
 * Re-fetches every `pollIntervalMs` milliseconds to stay in sync with
 * real-time instance status changes.
 */
export function useInstances(
  userId: number,
  pollIntervalMs = 10_000,
): UseInstancesResult {
  const [instances, setInstances] = useState<Instance[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const load = useCallback(async () => {
    setLoading(true);
    try {
      const data = await fetchInstances(userId);
      setInstances(data);
      setError(null);
    } catch (e) {
      setError((e as Error).message);
    } finally {
      setLoading(false);
    }
  }, [userId]);

  // Initial load + polling
  useEffect(() => {
    void load();
    const timer = setInterval(() => void load(), pollIntervalMs);
    return () => clearInterval(timer);
  }, [load, pollIntervalMs]);

  const create = useCallback(
    async (data: NewInstance) => {
      await createInstance(data);
      await load();
    },
    [load],
  );

  const activate = useCallback(
    async (id: number) => {
      await activateInstance(id);
      await load();
    },
    [load],
  );

  const deactivate = useCallback(
    async (id: number) => {
      await deactivateInstance(id);
      await load();
    },
    [load],
  );

  return { instances, loading, error, refresh: load, create, activate, deactivate };
}
