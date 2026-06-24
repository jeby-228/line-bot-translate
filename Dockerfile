# ── Stage 1: builder ──────────────────────────────────────────────────────────
FROM rust:alpine AS builder

RUN apk add --no-cache musl-dev

WORKDIR /app

# 先複製依賴清單，讓 Docker layer cache 在程式碼未變動時直接重用
COPY Cargo.toml Cargo.lock ./
# 建立假 main 以快取依賴編譯
RUN mkdir src && echo 'fn main(){}' > src/main.rs && \
    cargo build --release && \
    rm -rf src

# 複製實際原始碼並正式編譯
COPY src ./src
RUN touch src/main.rs && cargo build --release

# ── Stage 2: runtime ──────────────────────────────────────────────────────────
FROM alpine:3.21

RUN apk add --no-cache ca-certificates

WORKDIR /app

COPY --from=builder /app/target/release/line-bot-translate .

EXPOSE 8000

CMD ["./line-bot-translate"]
