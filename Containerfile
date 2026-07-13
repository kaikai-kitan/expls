# ------------------------------
# Stage 1. Build an app
# ------------------------------
FROM rust:1.96.0 AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

# ------------------------------
# Stage 2. Build for runtime
# ------------------------------
FROM dhi.io/debian-base:trixie

# 構築時にコマンドライン引数で受け取る変数
ARG GIT_REVISION
ARG BUILD_DATE
ARG VERSION

# イメージのメタデータ（アノテーション）を付与
LABEL org.opencontainers.image.title="expls" \
      org.opencontainers.image.description="Expand ls command" \
      org.opencontainers.image.url="https://kaikai-kitan.github.io/expls" \
      org.opencontainers.image.source="https://github.com/kaikai-kitan/expls" \
      org.opencontainers.image.version=${VERSION} \
      org.opencontainers.image.revision=${GIT_REVISION} \
      org.opencontainers.image.created=${BUILD_DATE} \
      org.opencontainers.image.licenses="MIT"

# builderステージからコンパイル済みバイナリのみをコピーして軽量化
COPY --from=builder /app/target/release/expls /app/expls
WORKDIR /opt
ENTRYPOINT [ "/app/expls" ]
