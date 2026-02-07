.PHONY: test test-frontend test-api

test: test-frontend test-api

test-frontend:
	node --test tests/frontend/fund_calc_core.test.js

test-api:
	@set -euo pipefail; \
	cargo run --bin blog >/tmp/myblog-test-server.log 2>&1 & \
	SERVER_PID=$$!; \
	trap 'kill $$SERVER_PID >/dev/null 2>&1 || true' EXIT; \
	for i in $$(seq 1 40); do \
		if curl -fsS http://127.0.0.1:8080/ >/dev/null 2>&1; then \
			break; \
		fi; \
		sleep 0.25; \
	done; \
	if ! curl -fsS http://127.0.0.1:8080/ >/dev/null 2>&1; then \
		echo "server did not become ready; check /tmp/myblog-test-server.log"; \
		exit 1; \
	fi; \
	cargo test --test api_tests -- --ignored --test-threads=1
