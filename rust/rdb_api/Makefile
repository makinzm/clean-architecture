.PHONY: all up down stop build e2e logs check-tools

E2E_DIR   := $(CURDIR)/.e2e
SERVER_LOG := $(E2E_DIR)/server.log
SERVER_PID := $(E2E_DIR)/server.pid

# Full flow: docker-compose up → wait for DB → build → e2e
all: check-tools up db e2e

check-tools:
	@command -v jq >/dev/null 2>&1 || { echo "ERROR: jq not found. Run 'devbox shell' first."; exit 1; }

# ── Infrastructure ──────────────────────────────────────────

up:
	docker compose up -d

down: stop
	docker compose down

stop:
	@if [ -f $(SERVER_PID) ]; then kill $$(cat $(SERVER_PID)) 2>/dev/null; rm -f $(SERVER_PID); fi
	@lsof -ti:8080 | xargs kill 2>/dev/null || true
	@for i in $$(seq 1 10); do lsof -ti:8080 >/dev/null 2>&1 || break; sleep 1; done

logs:
	tail -f $(SERVER_LOG)

db:
	@echo "==> Waiting for TiDB to be ready..."
	@for i in $$(seq 1 30); do \
		(echo > /dev/tcp/127.0.0.1/4000) 2>/dev/null \
			&& echo "  TiDB ready." && break; \
		echo "  [$$i/30] retrying in 2s..."; sleep 2; \
	done

# ── Build ────────────────────────────────────────────────────

build:
	cd server && cargo build

# ── E2E ─────────────────────────────────────────────────────

e2e: build
	@mkdir -p $(E2E_DIR)
	@if [ -f $(SERVER_PID) ]; then kill $$(cat $(SERVER_PID)) 2>/dev/null; rm -f $(SERVER_PID); fi
	@lsof -ti:8080 | xargs kill 2>/dev/null || true
	@for i in $$(seq 1 10); do lsof -ti:8080 >/dev/null 2>&1 || break; sleep 1; done
	@echo "==> Resetting DB..."
	@cd server && RUST_LOG=off ./target/debug/server --reset
	@echo "==> Starting server (log: $(SERVER_LOG))..."
	@cd server && RUST_LOG=warn ./target/debug/server >$(SERVER_LOG) 2>&1 & \
		echo $$! > $(SERVER_PID)
	@echo "==> Waiting for server to be healthy..."
	@for i in $$(seq 1 30); do \
		curl -sf http://localhost:8080/api-docs/openapi.json >/dev/null 2>&1 \
			&& echo "  Server ready." && break; \
		sleep 1; \
	done

	@echo ""
	@echo "━━━ [1] GET /api/v1/users  →  expect [] ━━━"
	@curl -s http://localhost:8080/api/v1/users | jq .

	@echo ""
	@echo "━━━ [2] POST /api/v1/users  →  create Alice ━━━"
	@curl -s -X POST http://localhost:8080/api/v1/users \
		-H 'Content-Type: application/json' \
		-d '{"name":"Alice","email":"alice@example.com"}' | jq .

	@echo ""
	@echo "━━━ [3] POST /api/v1/users  →  create Bob ━━━"
	@curl -s -X POST http://localhost:8080/api/v1/users \
		-H 'Content-Type: application/json' \
		-d '{"name":"Bob","email":"bob@example.com"}' | jq .

	@echo ""
	@echo "━━━ [4] GET /api/v1/users  →  expect [Alice, Bob] ━━━"
	@curl -s http://localhost:8080/api/v1/users | jq .

	@echo ""
	@echo "━━━ [5] GET /api/v1/users/1  →  expect Alice ━━━"
	@curl -s http://localhost:8080/api/v1/users/1 | jq .

	@echo ""
	@echo "━━━ [6] POST /api/v1/orders  →  Alice buys Widget × 3 (cross-repo tx) ━━━"
	@curl -s -X POST http://localhost:8080/api/v1/orders \
		-H 'Content-Type: application/json' \
		-d '{"user_id":1,"item_name":"Widget","quantity":3}' | jq .

	@echo ""
	@echo "━━━ [7] GET /api/v1/users/1  →  expect order_count=1 ━━━"
	@curl -s http://localhost:8080/api/v1/users/1 | jq .

	@echo ""
	@echo "━━━ [8] POST /api/v1/orders (user_id=999)  →  404 Not Found ━━━"
	@STATUS=$$(curl -s -o $(E2E_DIR)/resp.json -w '%{http_code}' \
		-X POST http://localhost:8080/api/v1/orders \
		-H 'Content-Type: application/json' \
		-d '{"user_id":999,"item_name":"Ghost","quantity":1}'); \
		jq . $(E2E_DIR)/resp.json; echo "  → HTTP $$STATUS"

	@echo ""
	@echo "━━━ [9] POST /api/v1/users (dup email)  →  409 Conflict ━━━"
	@STATUS=$$(curl -s -o $(E2E_DIR)/resp.json -w '%{http_code}' \
		-X POST http://localhost:8080/api/v1/users \
		-H 'Content-Type: application/json' \
		-d '{"name":"Alice2","email":"alice@example.com"}'); \
		jq . $(E2E_DIR)/resp.json; echo "  → HTTP $$STATUS"

	@echo ""
	@echo "==> Done. Stopping server..."
	@if [ -f $(SERVER_PID) ]; then kill $$(cat $(SERVER_PID)) 2>/dev/null; rm -f $(SERVER_PID); fi
	@lsof -ti:8080 | xargs kill 2>/dev/null || true
	@for i in $$(seq 1 10); do lsof -ti:8080 >/dev/null 2>&1 || break; sleep 1; done
