use std::env;
use std::path::PathBuf;

fn main() {
    // Locate OCCT installation
    let occt_root = env::var("OCCT_ROOT").unwrap_or_else(|_| {
        // Try homebrew opencascade as fallback (macOS dev)
        let brew_prefix = std::process::Command::new("brew")
            .args(["--prefix", "opencascade"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string());

        if let Some(prefix) = brew_prefix {
            if PathBuf::from(&prefix).exists() {
                return prefix;
            }
        }

        // Try common local install paths
        let home = env::var("HOME").unwrap_or_default();
        let local_path = format!("{home}/.local/occt-8.0.1");
        if PathBuf::from(&local_path).exists() {
            return local_path;
        }

        // Try /usr/local/occt (Docker)
        if PathBuf::from("/usr/local/occt").exists() {
            return "/usr/local/occt".to_string();
        }

        panic!(
            "OCCT not found. Set OCCT_ROOT env var, install via homebrew \
             (`brew install opencascade`), or run scripts/build-occt-local.sh"
        );
    });

    let occt_root = PathBuf::from(&occt_root);
    println!("cargo:warning=Using OCCT from: {}", occt_root.display());

    // Find include directory
    let include_dir = if occt_root.join("include/opencascade").exists() {
        occt_root.join("include/opencascade")
    } else if occt_root.join("include").exists() {
        occt_root.join("include")
    } else {
        panic!(
            "Cannot find OCCT include directory in {}",
            occt_root.display()
        );
    };

    // Find library directory
    let lib_dir = if occt_root.join("lib").exists() {
        occt_root.join("lib")
    } else if occt_root.join("lib64").exists() {
        occt_root.join("lib64")
    } else {
        panic!(
            "Cannot find OCCT lib directory in {}",
            occt_root.display()
        );
    };

    println!("cargo:rustc-link-search=native={}", lib_dir.display());

    // Helper: check if a library exists in the lib directory
    let lib_exists = |name: &str| -> bool {
        let dylib = lib_dir.join(format!("lib{name}.dylib"));
        let so = lib_dir.join(format!("lib{name}.so"));
        let a = lib_dir.join(format!("lib{name}.a"));
        dylib.exists() || so.exists() || a.exists()
    };

    // Core OCCT libraries (same across versions)
    let core_libs = [
        "TKernel", "TKMath", "TKG2d", "TKG3d", "TKGeomBase", "TKBRep",
        "TKGeomAlgo", "TKTopAlgo", "TKPrim", "TKMesh", "TKShHealing",
        "TKXSBase", "TKXCAF", "TKLCAF", "TKService", "TKBO", "TKFillet",
    ];

    for lib in &core_libs {
        println!("cargo:rustc-link-lib=dylib={lib}");
    }

    // STEP export libraries: naming changed between OCCT versions.
    // - OCCT <= 7.8: TKSTEP, TKSTEPBase, TKSTEPAttr, TKSTEP209
    // - OCCT >= 7.9: TKDESTEP, TKDE
    if lib_exists("TKDESTEP") {
        // OCCT 7.9+ / 8.x new modular data exchange
        println!("cargo:rustc-link-lib=dylib=TKDESTEP");
        println!("cargo:rustc-link-lib=dylib=TKDE");
        if lib_exists("TKXml") {
            println!("cargo:rustc-link-lib=dylib=TKCDF");
        }
    } else if lib_exists("TKSTEP") {
        // OCCT <= 7.8 classic layout
        println!("cargo:rustc-link-lib=dylib=TKSTEP");
        println!("cargo:rustc-link-lib=dylib=TKSTEPBase");
        println!("cargo:rustc-link-lib=dylib=TKSTEPAttr");
        println!("cargo:rustc-link-lib=dylib=TKSTEP209");
    } else {
        panic!("Cannot find STEP export libraries (TKDESTEP or TKSTEP)");
    }

    // TKVCAF may or may not exist depending on build flags
    if lib_exists("TKVCAF") {
        println!("cargo:rustc-link-lib=dylib=TKVCAF");
    }

    // macOS-specific: link to C++ standard library
    if cfg!(target_os = "macos") {
        println!("cargo:rustc-link-lib=dylib=c++");
    } else {
        println!("cargo:rustc-link-lib=dylib=stdc++");
    }

    // Build the cxx bridge with our C++ wrapper
    cxx_build::bridge("src/lib.rs")
        .file("cpp/geometry_bridge.cc")
        .include(&include_dir)
        .include("cpp")
        .std("c++17")
        .flag_if_supported("-Wno-deprecated-declarations")
        .flag_if_supported("-Wno-unused-parameter")
        .compile("bestcad_geometry_bridge");

    // Re-run if C++ sources change
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=cpp/geometry_bridge.h");
    println!("cargo:rerun-if-changed=cpp/geometry_bridge.cc");
    println!("cargo:rerun-if-env-changed=OCCT_ROOT");
}
