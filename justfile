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

# Format the code and sort dependencies
format:
    cargo fmt
    # cargo sort --workspace --grouped
    just --unstable --fmt

deny: _install_cargo-deny
    cargo deny check advisories
    cargo deny check bans licenses sources

# Lint the rust code
lint:
    just --unstable --fmt --check
    cargo fmt --all -- --check
    # cargo sort --workspace --grouped --check
    cargo clippy --workspace --all-features --all-targets -- --deny warnings --allow deprecated --allow unknown-lints

megalinter:
    @just _container run --pull always --rm -it -v "$PWD:/tmp/lint:rw" "megalinter/megalinter:v7"

# Launch tests
test: _install_cargo-nextest _install_cargo-hack
    cargo nextest run
    cargo test --doc
    cargo hack test --each-feature -- --test-threads=1

changelog: _install_git-cliff
    git-cliff -o "CHANGELOG.md"
    git add CHANGELOG.md && git commit -m "📝 update CHANGELOG"

release *arguments: _install_cargo-release _install_git-cliff
    cargo release --workspace --execute {{ arguments }}
    # git-cliff could not be used as `pre-release-hook` of cargo-release because it uses tag
    git-cliff -o "CHANGELOG.md"
    git add CHANGELOG.md && git commit -m "📝 update CHANGELOG" && git push

run_svc:
    cd cdviz-svc; cargo run
