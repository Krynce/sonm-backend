# Build Stage
FROM ghcr.io/krynce/sonm-backend/base:latest AS builder
FROM debian:13 AS debian

# Bundle Stage
FROM gcr.io/distroless/cc-debian13:nonroot
COPY --from=builder /home/rust/src/target/release/sonm-api ./
COPY --from=debian /usr/bin/uname /usr/bin/uname
# Static shell+wget for the container healthcheck; distroless has neither.
COPY --from=busybox:1.37.0-uclibc /bin/busybox /bin/busybox

EXPOSE 14702
ENV ROCKET_ADDRESS=0.0.0.0
USER nonroot
CMD ["./sonm-api"]
