# syntax=docker/dockerfile:1
# NeoTrix — Docker Image
# Build:  docker build -t neotrix/neotrix .
# Run:    docker run -it --rm neotrix/neotrix
#
# Multi-stage build:
#   1. rust:bookworm — compile neotrix from source
#   2. debian:bookworm-slim — minimal runtime image
#   3. kb-crawl — KB crawl daemon binary

FROM rust:bookworm AS builder

WORKDIR /build
COPY . .

RUN cargo build --release --bin neotrix

FROM rust:bookworm AS kb-crawl-builder

WORKDIR /build
COPY . .

RUN cargo build --release --bin neotrix-kb-crawl

FROM rust:bookworm AS dev

WORKDIR /build
COPY . .

RUN cargo build --bin neotrix

EXPOSE 3000
EXPOSE 2345

ENV RUST_LOG=debug
ENV RUST_BACKTRACE=1

ENTRYPOINT ["cargo", "run", "--bin"]
CMD ["neotrix", "--help"]

# ---- Runtime ----
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/target/release/neotrix /usr/local/bin/neotrix

EXPOSE 3000

# NeoTrix stores brain state at ~/.neotrix by default
ENV HOME=/home/neotrix
RUN useradd -m -d $HOME neotrix
USER neotrix
WORKDIR $HOME

ENTRYPOINT ["neotrix"]
CMD ["--help"]

# ---- KB Crawl Daemon ----
FROM debian:bookworm-slim AS kb-crawl

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=kb-crawl-builder /build/target/release/neotrix-kb-crawl /usr/local/bin/neotrix-kb-crawl

ENV HOME=/home/neotrix
RUN useradd -m -d $HOME neotrix
USER neotrix
WORKDIR $HOME

ENTRYPOINT ["neotrix-kb-crawl"]
CMD ["daemon", "--evolve"]
