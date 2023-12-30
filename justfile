requirements:
  cargo install  cargo-binstall
  cargo binstall cargo-nextest
  cargo binstall cargo-sort
  cargo binstall cargo-hack
  # cargo binstall cargo-insta
  cargo binstall cargo-release
  cargo binstall git-cliff

check:
  cargo hack check --each-feature --no-dev-deps

build:
  cargo build

# Format the code and sort dependencies
format:
  cargo fmt
  cargo sort --workspace --grouped

deny:
  cargo deny check advisories
  cargo deny check bans licenses sources

# Lint the rust code
lint:
  cargo fmt --all -- --check
  cargo sort --workspace --grouped --check
  cargo clippy --workspace --all-features --all-targets -- --deny warnings --allow deprecated --allow unknown-lints

megalinter:
  @just _container run --pull always --rm -it -v "$PWD:/tmp/lint:rw" "megalinter/megalinter:v7"

# Launch tests
test:
  cargo nextest run
  cargo test --doc
  cargo hack test --each-feature -- --test-threads=1

changelog:
  git-cliff -o "CHANGELOG.md"
  git add CHANGELOG.md && git commit -m "📝 update CHANGELOG"

release *arguments:
  cargo release --workspace --execute {{arguments}}
  # git-cliff could not be used as `pre-release-hook` of cargo-release because it uses tag
  git-cliff -o "CHANGELOG.md"
  git add CHANGELOG.md && git commit -m "📝 update CHANGELOG" && git push
