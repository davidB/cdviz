# see https://edu.chainguard.dev/chainguard/chainguard-images/reference/rust/image_specs/
FROM cgr.dev/chainguard/rust as build
ARG PROFILE=release
COPY . .
RUN cargo build "--$PROFILE"

# https://edu.chainguard.dev/chainguard/chainguard-images/reference/glibc-dynamic/image_specs/
FROM cgr.dev/chainguard/glibc-dynamic as cdviz-collector
ARG PROFILE=release
USER nonroot
COPY --from=build /work/target/${PROFILE}/cdviz-collector /usr/local/bin/cdviz-collector

ENV \
  OTEL_EXPORTER_OTLP_TRACES_ENDPOINT="http://127.0.0.1:4317" \
  OTEL_TRACES_SAMPLER="always_off"

CMD ["cdviz-collector"]
