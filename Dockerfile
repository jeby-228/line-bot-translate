# ── Stage 1: builder ──────────────────────────────────────────────────────────
# rust:bookworm 預裝 gcc/cmake/perl 等完整 C 工具鏈，aws-lc-rs 需要這些才能編譯
FROM rust:bookworm AS builder

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

# ── Stage 2: busybox（取出靜態 wget 供 healthcheck 使用）────────────────────
FROM busybox:stable-musl AS busybox

# ── Stage 3: runtime ─────────────────────────────────────────────────────────
# distroless/cc 只含最小化 glibc，無 shell / perl / curl，大幅降低攻擊面
FROM gcr.io/distroless/cc-debian12

COPY --from=busybox /bin/wget /bin/wget
COPY --from=builder /app/target/release/line-bot-translate /app/line-bot-translate

EXPOSE 8000

CMD ["/app/line-bot-translate"]
