# syntax=docker/dockerfile:1

ARG RUST_VERSION=1.88.0

FROM lukemathwalker/cargo-chef:latest-rust-${RUST_VERSION} AS chef
# 依存関係をキャッシュするcargo-chefを使用
WORKDIR /app

FROM chef AS planner
# 依存関係の解析
COPY . .
RUN  cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
# 依存関係のビルド
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# アプリケーションのビルド
COPY . .
RUN cargo build --release --bin tatsumi-exporter

FROM chef AS test
COPY . .
RUN cargo test

FROM gcr.io/distroless/cc-debian12 AS runtime
COPY --from=builder /app/target/release/tatsumi-exporter /usr/local/bin
ENTRYPOINT [ "/usr/local/bin/tatsumi-exporter" ]
