# Build Stage
FROM --platform="${BUILDPLATFORM}" rust:1.92.0-slim-trixie
USER 0:0
WORKDIR /home/rust/src

ARG TARGETARCH

ARG CARGO_BUILD_JOBS=10
ENV CARGO_BUILD_JOBS=${CARGO_BUILD_JOBS}

# Install build requirements
RUN dpkg --add-architecture "${TARGETARCH}"
RUN apt-get update && \
    apt-get install -y \
    make \
    pkg-config \
    libdav1d-dev:"${TARGETARCH}" \
    libssl-dev:"${TARGETARCH}"
COPY scripts/build-image-layer.sh /tmp/
RUN sh /tmp/build-image-layer.sh tools

# Build all dependencies
COPY Cargo.toml Cargo.lock ./
COPY crates/api/Cargo.toml ./crates/api/
COPY crates/gateway/Cargo.toml ./crates/gateway/
COPY crates/files/Cargo.toml ./crates/files/
COPY crates/embeds/Cargo.toml ./crates/embeds/
COPY crates/scheduler/Cargo.toml ./crates/scheduler/
COPY crates/push/Cargo.toml ./crates/push/
COPY crates/voice/Cargo.toml ./crates/voice/
COPY crates/core/config/Cargo.toml ./crates/core/config/
COPY crates/core/database/Cargo.toml ./crates/core/database/
COPY crates/core/storage/Cargo.toml ./crates/core/storage/
COPY crates/core/models/Cargo.toml ./crates/core/models/
COPY crates/core/parser/Cargo.toml ./crates/core/parser/
COPY crates/core/permissions/Cargo.toml ./crates/core/permissions/
COPY crates/core/presence/Cargo.toml ./crates/core/presence/
COPY crates/core/result/Cargo.toml ./crates/core/result/
COPY crates/core/coalesced/Cargo.toml ./crates/core/coalesced/
COPY crates/core/ratelimits/Cargo.toml ./crates/core/ratelimits/
RUN sh /tmp/build-image-layer.sh deps

# Build all apps
COPY crates ./crates
RUN sh /tmp/build-image-layer.sh apps
