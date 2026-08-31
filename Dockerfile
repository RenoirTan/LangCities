FROM rust:1.98.0-trixie AS dev-base

RUN apt-get update \
  && apt-get install --yes --no-install-recommends git libsqlite3-dev pkg-config \
  && rm -rf /var/lib/apt/lists/*

WORKDIR /app

FROM dev-base AS dev-build

FROM dev-base AS dev-auth

CMD ["/app/target/debug/langcities-auth"]

FROM dev-base AS dev-dc

CMD ["/app/target/debug/langcities-dc"]
