export interface GeometryRequest {
  width: number;
  height: number;
  depth: number;
}

export interface BoundingBox {
  min: [number, number, number];
  max: [number, number, number];
}

export interface MeshData {
  positions: number[];
  normals: number[];
  indices: number[];
  face_ids: number[];
}

export interface GeometryResponse {
  revision: string;
  mesh: MeshData;
  face_count: number;
  volume: number;
  bounds: BoundingBox;
}

export interface HealthResponse {
  status: string;
  occt_version: string;
}
