FROM rust:1.98-trixie AS dev-base

RUN apt-get update \
  && apt-get install --yes --no-install-recommends git libsqlite3-dev pkg-config \
  && rm -rf /var/lib/apt/lists/*

WORKDIR /app

FROM dev-base AS dev-build

RUN --mount=type=cache,target=/usr/local/cargo/registry \
  --mount=type=cache,target=/usr/local/cargo/target \
  --mount=type=cache,target=/usr/local/cargo/git \
  cargo install sea-orm-cli@^2

RUN rustup component add rustfmt

FROM dev-base AS dev-auth

CMD ["/app/target/debug/langcities-auth"]

FROM dev-base AS dev-dc

CMD ["/app/target/debug/langcities-dc"]
