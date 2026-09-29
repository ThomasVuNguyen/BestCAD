use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Parameters for creating a box primitive.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoxParams {
    pub width: f64,
    pub height: f64,
    pub depth: f64,
}

impl Default for BoxParams {
    fn default() -> Self {
        Self {
            width: 50.0,
            height: 30.0,
            depth: 20.0,
        }
    }
}

/// A 3D bounding box.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundingBox {
    pub min: [f64; 3],
    pub max: [f64; 3],
}

/// Tessellated mesh data ready for browser rendering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshData {
    /// Flat array of vertex positions: [x0, y0, z0, x1, y1, z1, ...]
    pub positions: Vec<f32>,
    /// Flat array of vertex normals: [nx0, ny0, nz0, ...]
    pub normals: Vec<f32>,
    /// Triangle indices into the positions/normals arrays
    pub indices: Vec<u32>,
    /// Per-triangle face ID (one entry per triangle, not per vertex)
    pub face_ids: Vec<u32>,
}

/// Response from a geometry operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeometryResponse {
    /// Unique revision identifier for this geometry state
    pub revision: String,
    /// Tessellated mesh for rendering
    pub mesh: MeshData,
    /// Number of topological faces
    pub face_count: u32,
    /// Solid volume in cubic mm
    pub volume: f64,
    /// Axis-aligned bounding box
    pub bounds: BoundingBox,
}

impl GeometryResponse {
    pub fn new_revision() -> String {
        Uuid::new_v4().to_string()
    }
}

/// Health check response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub occt_version: String,
}

/// Error response from the API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub detail: Option<String>,
}
