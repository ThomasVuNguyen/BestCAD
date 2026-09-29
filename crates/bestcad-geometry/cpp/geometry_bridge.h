#pragma once

// OCCT headers
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepMesh_IncrementalMesh.hxx>
#include <BRepGProp.hxx>
#include <GProp_GProps.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Shape.hxx>
#include <TopoDS_Face.hxx>
#include <TopExp_Explorer.hxx>
#include <BRep_Tool.hxx>
#include <Poly_Triangulation.hxx>
#include <TopLoc_Location.hxx>
#include <Bnd_Box.hxx>
#include <BRepBndLib.hxx>
#include <STEPControl_Writer.hxx>
#include <Standard_Version.hxx>
#include <gp_Pnt.hxx>
#include <gp_Dir.hxx>
#include <BRepCheck_Analyzer.hxx>

#include "rust/cxx.h"
#include <memory>
#include <cstdint>

namespace bestcad {

// MeshResult is defined by cxx from the Rust bridge definition.
// Forward-declare it here; the actual struct definition is generated.
struct MeshResult;

/// Create a box, tessellate it, and return mesh data.
std::unique_ptr<MeshResult> create_box_mesh(
    double width, double height, double depth, double mesh_deflection);

/// Create a box and export it as STEP. Returns STEP file bytes.
rust::Vec<uint8_t> export_box_step(double width, double height, double depth);

/// Return the OCCT version string.
rust::String occt_version();

/// Validate a box shape (returns true if B-rep is valid).
bool validate_box(double width, double height, double depth);

} // namespace bestcad
