# hadolint global ignore=DL3006,DL3008
# see [Multi-platform | Docker Docs](https://docs.docker.com/build/building/multi-platform/)
# see [Fast multi-arch Docker build for Rust projects - DEV Community](https://dev.to/vladkens/fast-multi-arch-docker-build-for-rust-projects-an1)
# see [How to create small Docker images for Rust](https://kerkour.com/rust-small-docker-image)
# alternative: https://edu.chainguard.dev/chainguard/chainguard-images/reference/rust/image_specs/

#---------------------------------------------------------------------------------------------------
# checkov:skip=CKV_DOCKER_7:Ensure the base image uses a non latest version tag
# trivy:ignore:AVD-DS-0001
FROM --platform=$BUILDPLATFORM rust:1.83.0-alpine as build
ARG PROFILE=release

ENV PKG_CONFIG_SYSROOT_DIR=/
RUN <<EOT
  set -eux
  # musl-dev is required for the musl target
  # zig + cargo-zigbuild are used to build cross platform C code
  # make is used by jmealloc and some C code
  apk add --no-cache musl-dev zig make # openssl-dev
  update-ca-certificates
EOT

RUN <<EOT
  set -eux
  rustup target add x86_64-unknown-linux-musl aarch64-unknown-linux-musl
  cargo install --locked cargo-zigbuild
EOT

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

# TODO explore alternative approache: packaged binaries downloaded from github release instead of building from source
RUN <<EOT
  set -eux
  cargo zigbuild --target x86_64-unknown-linux-musl --target aarch64-unknown-linux-musl "--$PROFILE"
  mkdir -p /app/linux
  ls target
  cp "target/aarch64-unknown-linux-musl/${PROFILE}/cdviz-collector" /app/linux/arm64
  cp "target/x86_64-unknown-linux-musl/${PROFILE}/cdviz-collector" /app/linux/amd64
EOT
# TODO RUN upx /work/target/${TARGET}/${PROFILE}/cdviz-collector

HEALTHCHECK NONE

#---------------------------------------------------------------------------------------------------
# checkov:skip=CKV_DOCKER_7:Ensure the base image uses a non latest version tag
# trivy:ignore:AVD-DS-0001
# TARGETPLATFORM usage to copy right binary from builder stage
# ARG populated by docker itself
FROM scratch as cdviz-collector
LABEL org.opencontainers.image.source="https://github.com/cdviz-dev/cdviz"
LABEL org.opencontainers.image.licenses="Apache-2.0"
ARG TARGETPLATFORM

COPY --from=build /etc/passwd /etc/passwd
COPY --from=build /etc/group /etc/group
USER nonroot
COPY --from=build /app/${TARGETPLATFORM} /app

ENV \
  OTEL_EXPORTER_OTLP_TRACES_ENDPOINT="http://127.0.0.1:4317" \
  OTEL_TRACES_SAMPLER="always_off"
HEALTHCHECK NONE
#see https://stackoverflow.com/questions/21553353/what-is-the-difference-between-cmd-and-entrypoint-in-a-dockerfile
ENTRYPOINT ["/app"]
CMD ["connect"]
