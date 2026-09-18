FROM rust:1.92-bookworm AS development
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY config ./config
COPY src ./src
EXPOSE 3000
CMD ["cargo", "run", "--locked"]

FROM development AS build
RUN cargo build --release --locked

FROM build AS validation
COPY tests ./tests
RUN rustup component add rustfmt clippy
RUN cargo fmt --check
RUN cargo clippy --release --locked --all-targets -- -D warnings
RUN cargo test --release --locked

FROM validation AS integration
CMD ["cargo", "test", "--release", "--locked", "--test", "integration", "--", "--ignored"]

FROM debian:bookworm-slim AS production
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --uid 10001 --create-home app
WORKDIR /app
COPY --from=build /app/target/release/tap-backend /usr/local/bin/tap-backend
USER app
EXPOSE 3000
CMD ["tap-backend"]
