//! OCCT geometry bridge for BestCAD.
//!
//! Provides Rust-safe wrappers around OCCT operations via `cxx`.
//! M0 scope: box creation, tessellation, STEP export, B-rep validation.

use bestcad_types::{BoundingBox, GeometryResponse, MeshData};

#[cxx::bridge(namespace = "bestcad")]
mod ffi {
    /// Flat mesh result from the C++ side.
    struct MeshResult {
        positions: Vec<f32>,
        normals: Vec<f32>,
        indices: Vec<u32>,
        face_ids: Vec<u32>,
        face_count: u32,
        volume: f64,
        bounds_min_x: f64,
        bounds_min_y: f64,
        bounds_min_z: f64,
        bounds_max_x: f64,
        bounds_max_y: f64,
        bounds_max_z: f64,
    }

    unsafe extern "C++" {
        include!("geometry_bridge.h");

        /// Create a tessellated box and return mesh data.
        fn create_box_mesh(
            width: f64,
            height: f64,
            depth: f64,
            mesh_deflection: f64,
        ) -> Result<UniquePtr<MeshResult>>;

        /// Export a box as STEP file bytes.
        fn export_box_step(
            width: f64,
            height: f64,
            depth: f64,
        ) -> Result<Vec<u8>>;

        /// Get the OCCT version string.
        fn occt_version() -> String;

        /// Validate that a box produces a valid B-rep.
        fn validate_box(
            width: f64,
            height: f64,
            depth: f64,
        ) -> bool;
    }
}

/// Default mesh deflection for tessellation (smaller = finer mesh).
const DEFAULT_MESH_DEFLECTION: f64 = 0.1;

/// Error type for geometry operations.
#[derive(Debug, thiserror::Error)]
pub enum GeometryError {
    #[error("Invalid dimensions: {0}")]
    InvalidDimensions(String),
    #[error("OCCT operation failed: {0}")]
    OcctError(String),
}

/// Create a tessellated box and return a full geometry response.
pub fn create_box(
    width: f64,
    height: f64,
    depth: f64,
) -> Result<GeometryResponse, GeometryError> {
    if width <= 0.0 || height <= 0.0 || depth <= 0.0 {
        return Err(GeometryError::InvalidDimensions(
            format!("All dimensions must be positive: w={width}, h={height}, d={depth}")
        ));
    }

    let result = ffi::create_box_mesh(width, height, depth, DEFAULT_MESH_DEFLECTION)
        .map_err(|e| GeometryError::OcctError(e.to_string()))?;

    Ok(GeometryResponse {
        revision: GeometryResponse::new_revision(),
        mesh: MeshData {
            positions: result.positions.clone(),
            normals: result.normals.clone(),
            indices: result.indices.clone(),
            face_ids: result.face_ids.clone(),
        },
        face_count: result.face_count,
        volume: result.volume,
        bounds: BoundingBox {
            min: [result.bounds_min_x, result.bounds_min_y, result.bounds_min_z],
            max: [result.bounds_max_x, result.bounds_max_y, result.bounds_max_z],
        },
    })
}

/// Export a box to STEP format, returning the file bytes.
pub fn export_step(
    width: f64,
    height: f64,
    depth: f64,
) -> Result<Vec<u8>, GeometryError> {
    if width <= 0.0 || height <= 0.0 || depth <= 0.0 {
        return Err(GeometryError::InvalidDimensions(
            format!("All dimensions must be positive: w={width}, h={height}, d={depth}")
        ));
    }

    ffi::export_box_step(width, height, depth)
        .map_err(|e| GeometryError::OcctError(e.to_string()))
}

/// Get the OCCT version string.
pub fn version() -> String {
    ffi::occt_version()
}

/// Validate that dimensions produce a valid B-rep.
pub fn validate(width: f64, height: f64, depth: f64) -> bool {
    ffi::validate_box(width, height, depth)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_occt_version() {
        let ver = version();
        assert!(!ver.is_empty(), "OCCT version should not be empty");
        println!("OCCT version: {ver}");
    }

    #[test]
    fn test_create_box() {
        let resp = create_box(50.0, 30.0, 20.0).expect("box creation failed");

        // A box has 6 faces
        assert_eq!(resp.face_count, 6, "box should have 6 faces");

        // Mesh should have data
        assert!(!resp.mesh.positions.is_empty(), "mesh must have positions");
        assert!(!resp.mesh.normals.is_empty(), "mesh must have normals");
        assert!(!resp.mesh.indices.is_empty(), "mesh must have indices");
        assert!(!resp.mesh.face_ids.is_empty(), "mesh must have face_ids");

        // Positions count should be divisible by 3
        assert_eq!(resp.mesh.positions.len() % 3, 0);
        assert_eq!(resp.mesh.normals.len() % 3, 0);

        // Indices count should be divisible by 3 (triangles)
        assert_eq!(resp.mesh.indices.len() % 3, 0);

        // One face_id per triangle
        assert_eq!(
            resp.mesh.face_ids.len(),
            resp.mesh.indices.len() / 3,
            "should have one face_id per triangle"
        );

        // Volume should be approximately width * height * depth
        let expected_volume = 50.0 * 30.0 * 20.0;
        let tolerance = expected_volume * 0.001;
        assert!(
            (resp.volume - expected_volume).abs() < tolerance,
            "volume {:.1} should be close to {:.1}",
            resp.volume,
            expected_volume
        );

        println!("Box: {} vertices, {} triangles, {} faces, volume={:.1}",
            resp.mesh.positions.len() / 3,
            resp.mesh.indices.len() / 3,
            resp.face_count,
            resp.volume,
        );
    }

    #[test]
    fn test_validate_box() {
        assert!(validate(10.0, 10.0, 10.0));
        assert!(!validate(-1.0, 10.0, 10.0));
        assert!(!validate(0.0, 0.0, 0.0));
    }

    #[test]
    fn test_export_step() {
        let step_bytes = export_step(50.0, 30.0, 20.0).expect("STEP export failed");
        assert!(!step_bytes.is_empty(), "STEP file should not be empty");

        // STEP files start with "ISO-10303"
        let header = String::from_utf8_lossy(&step_bytes[..50.min(step_bytes.len())]);
        assert!(
            header.contains("ISO-10303"),
            "STEP file should contain ISO-10303 header, got: {header}"
        );

        println!("STEP export: {} bytes", step_bytes.len());
    }

    #[test]
    fn test_step_roundtrip_volume() {
        // Create box and check volume, then export STEP and verify it's non-empty
        let resp = create_box(100.0, 50.0, 25.0).unwrap();
        let expected = 100.0 * 50.0 * 25.0;
        assert!(
            (resp.volume - expected).abs() < expected * 0.001,
            "volume roundtrip: got {}, expected {}", resp.volume, expected
        );

        let step = export_step(100.0, 50.0, 25.0).unwrap();
        assert!(step.len() > 100, "STEP should be substantial");
    }
}
