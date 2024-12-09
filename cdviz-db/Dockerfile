# hadolint global ignore=DL3006,DL3007

#---------------------------------------------------------------------------------------------------
# checkov:skip=CKV_DOCKER_7:Ensure the base image uses a non latest version tag
# trivy:ignore:AVD-DS-0001
FROM arigaio/atlas:latest AS cdviz-db
LABEL org.opencontainers.image.source="https://github.com/cdviz-dev/cdviz"
LABEL org.opencontainers.image.licenses="MIT OR Apache-2.0"
COPY migrations /migrations
HEALTHCHECK NONE
