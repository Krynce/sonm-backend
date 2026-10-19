# Build Stage
# Pinned by the Docker workflow to the base image built from this very commit; two
# overlapping tag pushes used to mix binaries from different base builds.
ARG BASE_IMAGE=ghcr.io/krynce/sonm-backend/base:latest
FROM ${BASE_IMAGE} AS builder
FROM debian:13 AS debian

# Bundle Stage
FROM gcr.io/distroless/cc-debian13:nonroot
COPY --from=builder /home/rust/src/target/release/sonm-scheduler ./
# sentry-contexts pulls in os_info, which shells out to `uname` for the OS context, and
# distroless ships no coreutils (originally revoltchat/self-hosted#104). Still needed.
COPY --from=debian /usr/bin/uname /usr/bin/uname

USER nonroot
CMD ["./sonm-scheduler"]