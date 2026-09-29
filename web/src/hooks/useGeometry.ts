import { useState, useCallback } from 'react';
import { GeometryRequest, GeometryResponse } from '@/types/geometry';
import { apiClient } from '@/api/client';

export function useGeometry() {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [data, setData] = useState<GeometryResponse | null>(null);

  const fetchGeometry = useCallback(async (req: GeometryRequest) => {
    setLoading(true);
    setError(null);
    try {
      const response = await apiClient.getBoxGeometry(req);
      setData(response);
    } catch (err: any) {
      setError(err.message || 'Failed to fetch geometry');
    } finally {
      setLoading(false);
    }
  }, []);

  const exportStep = useCallback(async (req: GeometryRequest) => {
    try {
      await apiClient.exportStep(req);
    } catch (err: any) {
      setError(err.message || 'Failed to export STEP');
    }
  }, []);

  return { data, loading, error, fetchGeometry, exportStep };
}
