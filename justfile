default:
    @just --list --unsorted

_install_cargo-binstall:
    cargo install cargo-binstall

_install_cargo-nextest: _install_cargo-binstall
    cargo binstall cargo-nextest -y

_install_cargo-insta: _install_cargo-binstall
    cargo binstall cargo-insta -y

_install_cargo-release: _install_cargo-binstall
    cargo binstall cargo-release -y

_install_cargo-hack: _install_cargo-binstall
    cargo binstall cargo-hack -y

_install_cargo-deny: _install_cargo-binstall
    cargo binstall cargo-deny -y

_install_git-cliff: _install_cargo-binstall
    cargo binstall git-cliff -y

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

# Lint all the code (via runing megalinter locally + `lint_rust`)
lint: lint_rust
    docker run --pull always --rm -it -v "$PWD:/tmp/lint:rw" "megalinter/megalinter:v7"

# Lint the rust code
lint_rust:
    just --unstable --fmt --check
    cargo fmt --all -- --check
    # cargo sort --workspace --grouped --check
    cargo clippy --workspace --all-features --all-targets -- --deny warnings --allow deprecated --allow unknown-lints

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

k8s_delete:
    # k3d cluster delete "$CLUSTER_NAME"
    # kind delete cluster --name "$CLUSTER_NAME"
    ctlptl delete cluster "$CLUSTER_NAME"
    ctlptl delete registry ctlptl-registry
