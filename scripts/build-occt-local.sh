#!/usr/bin/env bash
set -e

INSTALL_DIR="$HOME/.local/occt-8.0.1"
mkdir -p "$INSTALL_DIR"

echo "Building OCCT 8.0.1 from source to $INSTALL_DIR..."

WORKDIR=$(mktemp -d)
cd "$WORKDIR"

git clone --depth 1 --branch V8_0_1 https://github.com/Open-Cascade-SAS/OCCT.git
cd OCCT
mkdir build && cd build

cmake .. \
    -DCMAKE_BUILD_TYPE=Release \
    -DBUILD_MODULE_Draw=OFF \
    -DBUILD_MODULE_Visualization=OFF \
    -DBUILD_DOC_Overview=OFF \
    -DUSE_TBB=OFF \
    -DUSE_FREEIMAGE=OFF \
    -DUSE_VTK=OFF \
    -DCMAKE_INSTALL_PREFIX="$INSTALL_DIR"

make -j$(sysctl -n hw.ncpu) install

echo "OCCT installed to $INSTALL_DIR"
echo "Export these variables before running cargo:"
echo "export OCCT_ROOT=$INSTALL_DIR"
echo "export DYLD_LIBRARY_PATH=$INSTALL_DIR/lib:\$DYLD_LIBRARY_PATH"
