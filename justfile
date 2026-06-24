set shell := ["bash", "-c"]

default:
    @just --list

# 開發模式啟動（需要 .env）
run:
    cargo run

# 編譯 release 版本
build:
    cargo build --release

# 格式化
fmt:
    cargo fmt

# 格式化檢查（CI 用）
fmt-check:
    cargo fmt --check

# Clippy lint
lint:
    cargo clippy -- -D warnings

# 執行單元測試
test:
    cargo test

# Hurl 整合測試（需要 Docker）
test-hurl:
    docker compose -f docker-compose.test.yml up --build --abort-on-container-exit --exit-code-from hurl
    docker compose -f docker-compose.test.yml down

# Docker 建置
docker-build:
    docker compose build

# 啟動服務（背景）
docker-up:
    docker compose up -d --build

# 停止服務
docker-down:
    docker compose down

# 追蹤 logs
docker-logs:
    docker compose logs -f
