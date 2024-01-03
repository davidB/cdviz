# hadolint global ignore=DL3006

#---------------------------------------------------------------------------------------------------
# see https://edu.chainguard.dev/chainguard/chainguard-images/reference/rust/image_specs/
# checkov:skip=CKV_DOCKER_7:Ensure the base image uses a non latest version tag
# trivy:ignore:AVD-DS-0001
FROM cgr.dev/chainguard/rust as build
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
ARG PROFILE=release
USER nonroot
COPY --from=build /work/target/${PROFILE}/cdviz-collector /usr/local/bin/cdviz-collector

ENV \
  OTEL_EXPORTER_OTLP_TRACES_ENDPOINT="http://127.0.0.1:4317" \
  OTEL_TRACES_SAMPLER="always_off"
HEALTHCHECK NONE
CMD ["cdviz-collector"]

#---------------------------------------------------------------------------------------------------
# checkov:skip=CKV_DOCKER_7:Ensure the base image uses a non latest version tag
# trivy:ignore:AVD-DS-0001
FROM cgr.dev/chainguard/rust as build-sqlx
USER nonroot
RUN cargo install sqlx-cli --no-default-features --features rustls,postgres
HEALTHCHECK NONE

#---------------------------------------------------------------------------------------------------
# checkov:skip=CKV_DOCKER_7:Ensure the base image uses a non latest version tag
# trivy:ignore:AVD-DS-0001
FROM cgr.dev/chainguard/glibc-dynamic AS cdviz-dbmigration
USER nonroot
COPY --from=build-sqlx /home/nonroot/.cargo/bin/sqlx /usr/local/bin/sqlx
COPY migrations /migrations
HEALTHCHECK NONE
ENTRYPOINT ["/usr/local/bin/sqlx"]
