# Stage 1: occt-builder
FROM debian:bookworm AS occt-builder
ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update && apt-get install -y --no-install-recommends \
    cmake g++ make git ca-certificates \
    libfreetype-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build
RUN git clone --depth 1 --branch V8_0_1 https://github.com/Open-Cascade-SAS/OCCT.git
WORKDIR /build/OCCT/build
RUN cmake .. \
    -DCMAKE_BUILD_TYPE=Release \
    -DBUILD_MODULE_Draw=OFF \
    -DBUILD_MODULE_Visualization=OFF \
    -DBUILD_DOC_Overview=OFF \
    -DUSE_TBB=OFF \
    -DUSE_FREEIMAGE=OFF \
    -DUSE_VTK=OFF \
    -DUSE_TCL=OFF \
    -DUSE_TK=OFF \
    -DUSE_XLIB=OFF \
    -DCMAKE_INSTALL_PREFIX=/usr/local/occt
RUN make -j2 install

# Stage 2: web-builder
FROM node:22-bookworm-slim AS web-builder
WORKDIR /app/web
COPY web/package.json web/pnpm-lock.yaml* ./
RUN corepack enable
RUN pnpm install
COPY web/ ./
RUN pnpm build

# Stage 3: rust-builder
FROM rust:1-bookworm AS rust-builder
COPY --from=occt-builder /usr/local/occt /usr/local/occt
ENV OCCT_ROOT=/usr/local/occt
ENV LD_LIBRARY_PATH=/usr/local/occt/lib
WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
COPY crates ./crates
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    CARGO_BUILD_JOBS=2 cargo build --release && \
    cp target/release/bestcad-api /app/bestcad-api

# Stage 4: runtime
FROM debian:bookworm-slim AS runtime
COPY --from=occt-builder /usr/local/occt/lib /usr/local/occt/lib
COPY --from=rust-builder /app/bestcad-api /app/bestcad-api
COPY --from=web-builder /app/web/dist /app/web/dist

ENV LD_LIBRARY_PATH=/usr/local/occt/lib
ENV BESTCAD_STATIC_DIR=/app/web/dist
EXPOSE 3001

RUN apt-get update && apt-get install -y --no-install-recommends curl && rm -rf /var/lib/apt/lists/*
HEALTHCHECK --interval=30s --timeout=3s \
  CMD curl -f http://localhost:3001/api/health || exit 1

CMD ["/app/bestcad-api"]
