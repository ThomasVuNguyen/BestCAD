#include "bestcad-geometry/src/lib.rs.h"
#include "geometry_bridge.h"

#include <fstream>
#include <sstream>
#include <stdexcept>
#include <cstdio>

namespace bestcad {

std::unique_ptr<MeshResult> create_box_mesh(
    double width, double height, double depth, double mesh_deflection) {

    if (width <= 0.0 || height <= 0.0 || depth <= 0.0) {
        throw std::invalid_argument("Box dimensions must be positive");
    }

    // Create box centered at X/Y origin, base on Z=0
    gp_Pnt corner(-width / 2.0, -depth / 2.0, 0.0);
    BRepPrimAPI_MakeBox box_maker(corner, width, depth, height);
    box_maker.Build();

    if (!box_maker.IsDone()) {
        throw std::runtime_error("Failed to create box solid");
    }

    TopoDS_Shape shape = box_maker.Shape();

    // Validate the B-rep
    BRepCheck_Analyzer checker(shape);
    if (!checker.IsValid()) {
        throw std::runtime_error("Generated box has invalid B-rep");
    }

    // Compute volume
    GProp_GProps props;
    BRepGProp::VolumeProperties(shape, props);
    double volume = props.Mass();

    // Compute bounding box
    Bnd_Box bbox;
    BRepBndLib::Add(shape, bbox);
    double xmin, ymin, zmin, xmax, ymax, zmax;
    bbox.Get(xmin, ymin, zmin, xmax, ymax, zmax);

    // Tessellate the shape
    BRepMesh_IncrementalMesh mesher(shape, mesh_deflection);
    mesher.Perform();

    if (!mesher.IsDone()) {
        throw std::runtime_error("Tessellation failed");
    }

    // Collect triangulated mesh data from all faces
    auto result = std::make_unique<MeshResult>();
    result->volume = volume;
    result->bounds_min_x = xmin;
    result->bounds_min_y = ymin;
    result->bounds_min_z = zmin;
    result->bounds_max_x = xmax;
    result->bounds_max_y = ymax;
    result->bounds_max_z = zmax;

    uint32_t face_id = 0;
    uint32_t vertex_offset = 0;

    for (TopExp_Explorer explorer(shape, TopAbs_FACE); explorer.More();
         explorer.Next()) {

        const TopoDS_Face& face = TopoDS::Face(explorer.Current());
        TopLoc_Location location;

        Handle(Poly_Triangulation) triangulation =
            BRep_Tool::Triangulation(face, location);

        if (triangulation.IsNull()) {
            face_id++;
            continue;
        }

        const gp_Trsf& trsf = location.Transformation();
        bool has_transform = !location.IsIdentity();

        int nb_nodes = triangulation->NbNodes();
        int nb_triangles = triangulation->NbTriangles();
        bool has_normals = triangulation->HasNormals();

        // Determine face orientation for normal flipping
        bool reversed = (face.Orientation() == TopAbs_REVERSED);

        // Add vertices and normals
        for (int i = 1; i <= nb_nodes; i++) {
            gp_Pnt node = triangulation->Node(i);
            if (has_transform) {
                node.Transform(trsf);
            }

            result->positions.push_back(static_cast<float>(node.X()));
            result->positions.push_back(static_cast<float>(node.Y()));
            result->positions.push_back(static_cast<float>(node.Z()));

            if (has_normals) {
                gp_Dir normal = triangulation->Normal(i);
                if (has_transform) {
                    normal.Transform(trsf);
                }
                float sign = reversed ? -1.0f : 1.0f;
                result->normals.push_back(
                    static_cast<float>(normal.X()) * sign);
                result->normals.push_back(
                    static_cast<float>(normal.Y()) * sign);
                result->normals.push_back(
                    static_cast<float>(normal.Z()) * sign);
            } else {
                // Fallback: zero normals (should not happen for a box)
                result->normals.push_back(0.0f);
                result->normals.push_back(0.0f);
                result->normals.push_back(1.0f);
            }
        }

        // Add triangles (OCCT uses 1-based indexing)
        for (int i = 1; i <= nb_triangles; i++) {
            int n1, n2, n3;
            triangulation->Triangle(i).Get(n1, n2, n3);

            // Convert to 0-based and add vertex offset
            if (reversed) {
                // Flip winding order for reversed faces
                result->indices.push_back(
                    vertex_offset + static_cast<uint32_t>(n1 - 1));
                result->indices.push_back(
                    vertex_offset + static_cast<uint32_t>(n3 - 1));
                result->indices.push_back(
                    vertex_offset + static_cast<uint32_t>(n2 - 1));
            } else {
                result->indices.push_back(
                    vertex_offset + static_cast<uint32_t>(n1 - 1));
                result->indices.push_back(
                    vertex_offset + static_cast<uint32_t>(n2 - 1));
                result->indices.push_back(
                    vertex_offset + static_cast<uint32_t>(n3 - 1));
            }

            result->face_ids.push_back(face_id);
        }

        vertex_offset += static_cast<uint32_t>(nb_nodes);
        face_id++;
    }

    result->face_count = face_id;
    return result;
}

rust::Vec<uint8_t> export_box_step(double width, double height, double depth) {
    if (width <= 0.0 || height <= 0.0 || depth <= 0.0) {
        throw std::invalid_argument("Box dimensions must be positive");
    }

    gp_Pnt corner(-width / 2.0, -depth / 2.0, 0.0);
    BRepPrimAPI_MakeBox box_maker(corner, width, depth, height);
    box_maker.Build();

    if (!box_maker.IsDone()) {
        throw std::runtime_error("Failed to create box for STEP export");
    }

    TopoDS_Shape shape = box_maker.Shape();

    // Write STEP to a temporary file, then read bytes
    char tmp_file[256];
    snprintf(tmp_file, sizeof(tmp_file), "/tmp/bestcad_step_%p.step",
             static_cast<void*>(&shape));

    STEPControl_Writer writer;
    IFSelect_ReturnStatus status =
        writer.Transfer(shape, STEPControl_AsIs);

    if (status != IFSelect_RetDone) {
        throw std::runtime_error("STEP transfer failed");
    }

    IFSelect_ReturnStatus write_status = writer.Write(tmp_file);
    if (write_status != IFSelect_RetDone) {
        throw std::runtime_error("STEP file write failed");
    }

    // Read the file back into memory
    std::ifstream file(tmp_file, std::ios::binary | std::ios::ate);
    if (!file.is_open()) {
        throw std::runtime_error("Failed to read STEP temporary file");
    }

    auto size = file.tellg();
    file.seekg(0, std::ios::beg);

    rust::Vec<uint8_t> bytes;
    bytes.reserve(static_cast<size_t>(size));

    char buffer[4096];
    while (file.read(buffer, sizeof(buffer))) {
        for (std::streamsize i = 0; i < file.gcount(); i++) {
            bytes.push_back(static_cast<uint8_t>(buffer[i]));
        }
    }
    // Handle remaining bytes
    for (std::streamsize i = 0; i < file.gcount(); i++) {
        bytes.push_back(static_cast<uint8_t>(buffer[i]));
    }

    file.close();
    std::remove(tmp_file);

    return bytes;
}

rust::String occt_version() {
    return rust::String(OCC_VERSION_COMPLETE);
}

bool validate_box(double width, double height, double depth) {
    if (width <= 0.0 || height <= 0.0 || depth <= 0.0) {
        return false;
    }

    gp_Pnt corner(-width / 2.0, -depth / 2.0, 0.0);
    BRepPrimAPI_MakeBox box_maker(corner, width, depth, height);
    box_maker.Build();

    if (!box_maker.IsDone()) {
        return false;
    }

    BRepCheck_Analyzer checker(box_maker.Shape());
    return checker.IsValid();
}

} // namespace bestcad
