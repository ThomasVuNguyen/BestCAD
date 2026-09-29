import { useEffect, useState } from 'react';
import { Header } from '@/components/Header';
import { FeatureTree } from '@/components/FeatureTree';
import { PropertyPanel } from '@/components/PropertyPanel';
import { Viewport } from '@/components/Viewport';
import { useGeometry } from '@/hooks/useGeometry';
import { GeometryRequest } from '@/types/geometry';

function App() {
  const { data, loading, error, fetchGeometry, exportStep } = useGeometry();
  
  const [currentDims, setCurrentDims] = useState<GeometryRequest>({
    width: 10,
    height: 10,
    depth: 10
  });

  useEffect(() => {
    fetchGeometry(currentDims);
  }, []);

  const handleApply = (dims: GeometryRequest) => {
    setCurrentDims(dims);
    fetchGeometry(dims);
  };

  const handleExport = (req: GeometryRequest) => {
    exportStep(req);
  };

  return (
    <div className="app-container">
      <Header onExport={handleExport} currentDims={currentDims} />
      
      {error && (
        <div className="error-banner">
          {error}
        </div>
      )}

      <div className="main-content">
        <aside className="sidebar">
          <FeatureTree />
          <PropertyPanel 
            initialDims={currentDims} 
            onApply={handleApply} 
            loading={loading}
            data={data}
          />
        </aside>
        <main className="viewport-container" style={{ flex: 1 }}>
          <Viewport data={data} />
        </main>
      </div>
    </div>
  );
}

export default App;
