import { GeometryRequest, GeometryResponse, HealthResponse } from '@/types/geometry';

export const apiClient = {
  async getHealth(): Promise<HealthResponse> {
    const response = await fetch('/api/health');
    if (!response.ok) {
      throw new Error(`API error: ${response.statusText}`);
    }
    return response.json();
  },

  async getBoxGeometry(req: GeometryRequest): Promise<GeometryResponse> {
    const response = await fetch('/api/geometry/box', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(req),
    });
    if (!response.ok) {
      throw new Error(`API error: ${response.statusText}`);
    }
    return response.json();
  },

  async exportStep(req: GeometryRequest): Promise<void> {
    const response = await fetch('/api/geometry/export-step', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(req),
    });
    if (!response.ok) {
      throw new Error(`API error: ${response.statusText}`);
    }
    
    const blob = await response.blob();
    const url = window.URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'box.step';
    document.body.appendChild(a);
    a.click();
    window.URL.revokeObjectURL(url);
    document.body.removeChild(a);
  }
};
