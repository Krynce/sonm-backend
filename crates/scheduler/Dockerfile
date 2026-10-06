# Build Stage
FROM ghcr.io/krynce/sonm-backend/base:latest AS builder
FROM debian:13 AS debian

# Bundle Stage
FROM gcr.io/distroless/cc-debian13:nonroot
COPY --from=builder /home/rust/src/target/release/sonm-scheduler ./
COPY --from=debian /usr/bin/uname /usr/bin/uname

USER nonroot
CMD ["./sonm-scheduler"]