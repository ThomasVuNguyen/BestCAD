import React, { useState } from 'react';
import { GeometryRequest, GeometryResponse } from '@/types/geometry';

interface PropertyPanelProps {
  initialDims: GeometryRequest;
  onApply: (dims: GeometryRequest) => void;
  loading: boolean;
  data: GeometryResponse | null;
}

export const PropertyPanel: React.FC<PropertyPanelProps> = ({ initialDims, onApply, loading, data }) => {
  const [dims, setDims] = useState<GeometryRequest>(initialDims);

  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const { name, value } = e.target;
    setDims(prev => ({ ...prev, [name]: parseFloat(value) || 0 }));
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    onApply(dims);
  };

  return (
    <div className="sidebar-section">
      <h2>Properties</h2>
      <form onSubmit={handleSubmit}>
        <div className="form-group">
          <label htmlFor="width">Width</label>
          <input 
            type="number" 
            id="width" 
            name="width" 
            value={dims.width} 
            onChange={handleChange} 
            step="0.1"
            data-testid="width-input"
          />
        </div>
        <div className="form-group">
          <label htmlFor="height">Height</label>
          <input 
            type="number" 
            id="height" 
            name="height" 
            value={dims.height} 
            onChange={handleChange} 
            step="0.1"
            data-testid="height-input"
          />
        </div>
        <div className="form-group">
          <label htmlFor="depth">Depth</label>
          <input 
            type="number" 
            id="depth" 
            name="depth" 
            value={dims.depth} 
            onChange={handleChange} 
            step="0.1"
            data-testid="depth-input"
          />
        </div>
        <button type="submit" className="btn" disabled={loading} data-testid="apply-btn">
          {loading ? 'Generating...' : 'Apply Changes'}
        </button>
      </form>

      {data && (
        <div className="stats">
          <div>Volume: {data.volume.toFixed(2)}</div>
          <div>Faces: {data.face_count}</div>
          <div>Rev: {data.revision.substring(0, 8)}</div>
        </div>
      )}
    </div>
  );
};
