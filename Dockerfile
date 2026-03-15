# ============================================================
# Stage 1: Build the server binary with static musl linking
# ============================================================
FROM rust:1.85-alpine AS builder

RUN apk add --no-cache musl-dev

WORKDIR /build

# -- Copy workspace manifests and lockfile first (cache layer) --
# COPY Cargo.toml Cargo.lock ./
# COPY server/Cargo.toml server/Cargo.toml
# COPY algos/Cargo.toml algos/Cargo.toml
#
# # Create stub workspace members so Cargo can resolve the full workspace.
# # headless and tui are not needed for the server build, but the workspace
# # definition references them.
# RUN mkdir -p headless/src && echo "fn main(){}" > headless/src/main.rs \
#  && mkdir -p tui/src     && echo "fn main(){}" > tui/src/main.rs
#
# # Create dummy entry points for the crates we *do* build, so that
# # `cargo build` downloads and compiles all dependencies in a cacheable layer.
# RUN mkdir -p server/src && echo "fn main(){}" > server/src/main.rs \
#  && mkdir -p algos/src  && echo "" > algos/src/lib.rs
#
# RUN cargo build --release -p server
#
# # -- Copy real source and rebuild only the application code --
COPY algos algos
COPY server server

WORKDIR /build/server

# Touch the crate roots so Cargo knows they changed (timestamps matter).
RUN cargo build --release

# ============================================================
# Stage 2: Minimal runtime image
# ============================================================
FROM alpine:3.21 AS runtime

# Create a non-root user for security
# RUN addgroup -S app && adduser -S app -G app

# Create a data directory for the server's working files (.md / .md.structure)
RUN mkdir -p /data 
# && chown app:app /data

COPY --from=builder /build/server/target/release/server /usr/local/bin/server

# USER app
WORKDIR /data

EXPOSE 9001

ENTRYPOINT ["server"]
