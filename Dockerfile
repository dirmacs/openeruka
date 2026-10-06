# syntax=docker/dockerfile:1.7
# openeruka — server binary (openeruka-server crate, bin "openeruka").
#
# Rust workspace; rusqlite is bundled (no system sqlite). No native libraries.
# Build:
#   docker build -t openeruka:local .
# Run (mount a volume to persist the sqlite db):
#   docker run --rm -v openeruka-data:/home/openeruka/.local/share openeruka:local

ARG RUST_IMAGE=rust:1.99-bookworm
ARG RUNTIME_IMAGE=debian:bookworm-slim

FROM ${RUST_IMAGE} AS chef
RUN cargo install cargo-chef --locked
WORKDIR /workspace/openeruka

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
ENV CARGO_TERM_COLOR=always
ENV RUSTFLAGS="-C strip=symbols"
COPY --from=planner /workspace/openeruka/recipe.json recipe.json
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/workspace/openeruka/target \
    cargo chef cook --release --recipe-path recipe.json

COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/workspace/openeruka/target \
    cargo build --release --bin openeruka \
 && cp target/release/openeruka /usr/local/bin/openeruka

FROM ${RUNTIME_IMAGE} AS runtime
RUN apt-get update \
 && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
      ca-certificates \
 && rm -rf /var/lib/apt/lists/* \
 && useradd --create-home --uid 10001 --shell /usr/sbin/nologin openeruka

COPY --from=builder /usr/local/bin/openeruka /usr/local/bin/openeruka

USER openeruka
WORKDIR /home/openeruka
ENV HOME=/home/openeruka

ENTRYPOINT ["openeruka"]
CMD ["--help"]
