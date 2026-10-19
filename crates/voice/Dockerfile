# Build Stage
# Pinned by the Docker workflow to the base image built from this very commit; two
# overlapping tag pushes used to mix binaries from different base builds.
ARG BASE_IMAGE=ghcr.io/krynce/sonm-backend/base:latest
FROM ${BASE_IMAGE} AS builder
FROM debian:13 AS debian

# Bundle Stage
FROM gcr.io/distroless/cc-debian13:nonroot
COPY --from=builder /home/rust/src/target/release/sonm-voice ./
# sentry-contexts pulls in os_info, which shells out to `uname` for the OS context, and
# distroless ships no coreutils (originally revoltchat/self-hosted#104). Still needed.
COPY --from=debian /usr/bin/uname /usr/bin/uname
# Static shell+wget for the container healthcheck; distroless has neither.
COPY --from=busybox:1.37.0-uclibc /bin/busybox /bin/busybox

USER nonroot
CMD ["./sonm-voice"]