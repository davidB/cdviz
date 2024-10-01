# hadolint global ignore=DL3006
# see [How to create small Docker images for Rust](https://kerkour.com/rust-small-docker-image)
# alternative: https://edu.chainguard.dev/chainguard/chainguard-images/reference/rust/image_specs/

#---------------------------------------------------------------------------------------------------
# checkov:skip=CKV_DOCKER_7:Ensure the base image uses a non latest version tag
# trivy:ignore:AVD-DS-0001
FROM rust:1.81.0 as build
ARG PROFILE=release
ARG TARGET=x86_64-unknown-linux-musl

RUN rustup target add x86_64-unknown-linux-musl
RUN apt-get update && apt-get install -y musl-tools musl-dev
RUN update-ca-certificates

# Create appuser
ENV USER=nonroot
ENV UID=10001

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    "${USER}"


WORKDIR /work

COPY ./ .

RUN cargo build --target ${TARGET} "--$PROFILE"
# TODO RUN upx /work/target/${TARGET}/${PROFILE}/cdviz-collector
HEALTHCHECK NONE

#---------------------------------------------------------------------------------------------------
# checkov:skip=CKV_DOCKER_7:Ensure the base image uses a non latest version tag
# trivy:ignore:AVD-DS-0001
FROM scratch as cdviz-collector
LABEL org.opencontainers.image.source="https://github.com/davidB/cdviz"
LABEL org.opencontainers.image.licenses="Apache-2.0"
ARG PROFILE=release
ARG TARGET=x86_64-unknown-linux-musl

COPY --from=build /etc/passwd /etc/passwd
COPY --from=build /etc/group /etc/group
USER nonroot
COPY --from=build /work/target/${TARGET}/${PROFILE}/cdviz-collector /cdviz-collector

ENV \
  OTEL_EXPORTER_OTLP_TRACES_ENDPOINT="http://127.0.0.1:4317" \
  OTEL_TRACES_SAMPLER="always_off"
HEALTHCHECK NONE
CMD ["/cdviz-collector"]
