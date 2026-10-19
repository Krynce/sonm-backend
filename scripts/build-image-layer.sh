#!/bin/sh
# If you're having trouble building this locally or on your CI, try lowering
# the job count via CARGO_BUILD_JOBS. It defaults to 10.
#
# The crate lists are derived from the manifests already in the tree: apps are crates/<name>,
# libraries are crates/core/<name>. The only list of crates is the set of COPY lines in the
# Dockerfile; adding a crate here is not needed and used to silently break layer caching.

app_crates() {
  for manifest in crates/*/Cargo.toml; do
    [ -f "$manifest" ] || continue
    name="${manifest#crates/}"
    echo "${name%/Cargo.toml}"
  done
}

lib_crates() {
  for manifest in crates/core/*/Cargo.toml; do
    [ -f "$manifest" ] || continue
    name="${manifest#crates/core/}"
    echo "${name%/Cargo.toml}"
  done
}

if [ -z "$TARGETARCH" ]; then
  :
else
  case "${TARGETARCH}" in
    "amd64")
      LINKER_NAME="x86_64-linux-gnu-gcc"
      LINKER_PACKAGE="gcc-x86-64-linux-gnu"
      BUILD_TARGET="x86_64-unknown-linux-gnu" ;;
    "arm64")
      LINKER_NAME="aarch64-linux-gnu-gcc"
      LINKER_PACKAGE="gcc-aarch64-linux-gnu"
      BUILD_TARGET="aarch64-unknown-linux-gnu" ;;
  esac
fi

tools() {
  apt-get install -y "${LINKER_PACKAGE}"
  rustup target add "${BUILD_TARGET}"
}

deps() {
  for app in $(app_crates); do
    mkdir -p "crates/$app/src"
    echo 'fn main() { panic!("stub"); }' > "crates/$app/src/main.rs"
  done
  for lib in $(lib_crates); do
    mkdir -p "crates/core/$lib/src"
    echo '' > "crates/core/$lib/src/lib.rs"
  done

  if [ -z "$TARGETARCH" ]; then
    cargo build -j "${CARGO_BUILD_JOBS:-10}" --locked --release
  else
    cargo build -j "${CARGO_BUILD_JOBS:-10}" --locked --release --target "${BUILD_TARGET}"
  fi
}

apps() {
  for app in $(app_crates); do touch -am "crates/$app/src/main.rs"; done
  for lib in $(lib_crates); do touch -am "crates/core/$lib/src/lib.rs"; done

  if [ -z "$TARGETARCH" ]; then
    cargo build -j "${CARGO_BUILD_JOBS:-10}" --locked --release
  else
    cargo build -j "${CARGO_BUILD_JOBS:-10}" --locked --release --target "${BUILD_TARGET}"
    mv target _target && mv _target/"${BUILD_TARGET}" target
  fi
}

if [ -z "$TARGETARCH" ]; then
  :
else
  export RUSTFLAGS="-C linker=${LINKER_NAME}"
  export PKG_CONFIG_ALLOW_CROSS="1"
  export PKG_CONFIG_PATH="/usr/lib/pkgconfig:/usr/lib/aarch64-linux-gnu/pkgconfig:/usr/lib/x86_64-linux-gnu/pkgconfig"
fi

"$@"
