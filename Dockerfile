# see https://edu.chainguard.dev/chainguard/chainguard-images/reference/rust/image_specs/
FROM cgr.dev/chainguard/rust as build
ARG PROFILE=release
COPY . .
RUN cargo build "--$PROFILE"

# https://edu.chainguard.dev/chainguard/chainguard-images/reference/glibc-dynamic/image_specs/
# checkov:skip=CKV_DOCKER_7:Ensure the base image uses a non latest version tag
FROM cgr.dev/chainguard/glibc-dynamic as cdviz-collector
ARG PROFILE=release
COPY --from=build /work/target/${PROFILE}/cdviz-collector /usr/local/bin/cdviz-collector

ENV \
  OTEL_EXPORTER_OTLP_TRACES_ENDPOINT="http://127.0.0.1:4317" \
  OTEL_TRACES_SAMPLER="always_off"

CMD ["cdviz-collector"]

FROM cgr.dev/chainguard/rust as build-sqlx
RUN cargo install sqlx-cli --no-default-features --features rustls,postgres

# checkov:skip=CKV_DOCKER_7:Ensure the base image uses a non latest version tag
FROM cgr.dev/chainguard/glibc-dynamic AS cdviz-dbmigration
COPY --from=build-sqlx /home/nonroot/.cargo/bin/sqlx /usr/local/bin/sqlx
COPY migrations /migrations
ENTRYPOINT ["/usr/local/bin/sqlx"]

# # For now we use sqlx for DB migration, later we may switch to atlas.
# # checkov:skip=CKV_DOCKER_7:Ensure the base image uses a non latest version tag
# FROM arigaio/atlas:0.10.1 AS db-migration
# COPY migrations /migrations
