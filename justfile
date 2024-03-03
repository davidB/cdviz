pg_offline_pwd := "mysecretpassword"
pg_offline_user := "me"
pg_offline_url := "postgres://" + pg_offline_user + ":" + pg_offline_pwd + "@127.0.0.1:5432/" + pg_offline_user

default:
    @just --list --unsorted

_install_cargo-binstall:
    @# cargo install --locked cargo-binstall
    @(cargo-binstall -V > /dev/null) || (curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash)

_binstall ARG: _install_cargo-binstall
    @(cargo binstall -y {{ ARG }} || cargo install --locked {{ ARG }})

_install_cargo-nextest:
    @just _binstall cargo-nextest

_install_cargo-insta:
    @just _binstall cargo-insta

_install_cargo-release:
    @just _binstall cargo-release

_install_cargo-deny:
    @just _binstall cargo-deny

_install_git-cliff:
    @just _binstall git-cliff

_install_cargo-hack:
    @just _binstall cargo-hack

_install_rustfmt_clippy:
    rustup component add rustfmt
    rustup component add clippy

_install_sqlx-cli:
    # use Rustls rather than OpenSSL (be sure to add the features for the databases you intend to use!)
    # no binstall available
    cargo install sqlx-cli --no-default-features --features rustls,postgres --locked

check: _install_cargo-hack
    cargo hack check --each-feature --no-dev-deps

build:
    cargo build

alias fmt := format

# Format the code and sort dependencies
format:
    cargo fmt
    # cargo sort --workspace --grouped
    just --unstable --fmt

deny: _install_cargo-deny
    cargo deny check advisories
    cargo deny check bans licenses sources

# Lint all the code (megalinter + lint_rust)
lint: lint_rust megalinter

# Lint the rust code
lint_rust:
    just --unstable --fmt --check
    cargo fmt --all -- --check
    # cargo sort --workspace --grouped --check
    cargo clippy --workspace --all-features --all-targets -- --deny warnings --allow deprecated --allow unknown-lints

# Lint with megalinter (locally via docker)
megalinter:
    # rm -rf megalinter-reports
    docker run --rm --name megalinter -it --env "DEFAULT_WORKSPACE=/tmp/lint" -v "${DOCKER_HOST:-/var/run/docker.sock}:/var/run/docker.sock:rw" -v "$PWD:/tmp/lint:rw" "oxsecurity/megalinter:v7"

# Launch tests
test: _install_cargo-nextest
    cargo nextest run
    # cargo test --doc
    # cargo hack nextest --each-feature -- --test-threads=1

changelog: _install_git-cliff
    git-cliff -o "CHANGELOG.md"
    git add CHANGELOG.md && git commit -m "📝 update CHANGELOG"

release *arguments: _install_cargo-release _install_git-cliff
    cargo release --workspace --execute {{ arguments }}
    # git-cliff could not be used as `pre-release-hook` of cargo-release because it uses tag
    git-cliff -o "CHANGELOG.md"
    git add CHANGELOG.md && git commit -m "📝 update CHANGELOG" && git push

# local_run_cdviz-collector:
#     cd cdviz-collector; cargo run

k8s_create:
    # sudo systemctl start docker
    # k3d cluster create "$CLUSTER_NAME" --agents 2
    # kind create cluster --name "$CLUSTER_NAME"
    ctlptl create registry ctlptl-registry --port=5005
    ctlptl create cluster kind --name "$CLUSTER_NAME" --registry=ctlptl-registry
    kubectl cluster-info --context "$CLUSTER_NAME"

k8s_dev:
    skaffold dev --port-forward

k8s_delete_cdviz:
    helm delete cdviz -n cdviz --cascade foreground || true
    kubectl delete namespace cdviz

k8s_delete:
    # k3d cluster delete "$CLUSTER_NAME"
    # kind delete cluster --name "$CLUSTER_NAME"
    ctlptl delete cluster "$CLUSTER_NAME"
    ctlptl delete registry ctlptl-registry

sqlx_create_migration name: _install_sqlx-cli
    sqlx migrate add -r {{ name }}

sqlx_prepare_offline: _install_sqlx-cli
    docker rm -f postgres || true
    docker run --name postgres -e POSTGRES_PASSWORD={{ pg_offline_pwd }} -e POSTGRES_USER={{ pg_offline_user }} -p 5432:5432 -d postgres:16.1 && sleep 3
    sqlx database create --database-url {{ pg_offline_url }}
    sqlx migrate run --database-url {{ pg_offline_url }}
    cargo sqlx prepare --workspace --database-url {{ pg_offline_url }}
    sqlx database drop -y --database-url {{ pg_offline_url }}
    docker rm -f postgres

run_collector:
    cd cdviz-collector; cargo run -- --config ./cdviz-collector.toml
