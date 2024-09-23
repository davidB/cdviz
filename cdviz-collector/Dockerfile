# hadolint global ignore=DL3006

#---------------------------------------------------------------------------------------------------
# see https://edu.chainguard.dev/chainguard/chainguard-images/reference/rust/image_specs/
# checkov:skip=CKV_DOCKER_7:Ensure the base image uses a non latest version tag
# trivy:ignore:AVD-DS-0001
FROM cgr.dev/chainguard/rust:1.81.0 as build
ARG PROFILE=release
USER nonroot
WORKDIR /work
COPY . .
RUN cargo build "--$PROFILE"
HEALTHCHECK NONE

#---------------------------------------------------------------------------------------------------
# https://edu.chainguard.dev/chainguard/chainguard-images/reference/glibc-dynamic/image_specs/
# checkov:skip=CKV_DOCKER_7:Ensure the base image uses a non latest version tag
# trivy:ignore:AVD-DS-0001
FROM cgr.dev/chainguard/glibc-dynamic as cdviz-collector
LABEL org.opencontainers.image.source="https://github.com/davidB/cdviz"
LABEL org.opencontainers.image.licenses="Apache-2.0"
ARG PROFILE=release
USER nonroot
COPY --from=build /work/target/${PROFILE}/cdviz-collector /usr/local/bin/cdviz-collector

ENV \
  OTEL_EXPORTER_OTLP_TRACES_ENDPOINT="http://127.0.0.1:4317" \
  OTEL_TRACES_SAMPLER="always_off"
HEALTHCHECK NONE
CMD ["cdviz-collector"]
