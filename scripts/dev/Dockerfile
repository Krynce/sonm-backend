FROM rust:1.92.0-slim-trixie
RUN apt-get update && apt-get install -y pkg-config libdav1d-dev libssl-dev make curl && rm -rf /var/lib/apt/lists/*
RUN curl -LsSf https://get.nexte.st/0.9.122/linux | tar zxf - -C /usr/local/cargo/bin
RUN rustup component add clippy rustfmt
