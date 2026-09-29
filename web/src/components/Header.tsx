import React from 'react';
import { GeometryRequest } from '@/types/geometry';

interface HeaderProps {
  onExport: (req: GeometryRequest) => void;
  currentDims: GeometryRequest;
}

export const Header: React.FC<HeaderProps> = ({ onExport, currentDims }) => {
  return (
    <header className="header">
      <h1>BestCAD</h1>
      <button 
        className="btn btn-secondary" 
        onClick={() => onExport(currentDims)}
        data-testid="export-btn"
      >
        Export STEP
      </button>
    </header>
  );
};
