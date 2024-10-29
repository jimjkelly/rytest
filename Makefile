bench:
	@echo "Running benchmarks..."
	python3 -m venv .venv
	cargo install --locked hyperfine
	cargo build --release --package rytest
	bash scripts/benchmark.sh pallets/flask
	cargo render-benchmark output/pallets/flask/result.json --title "Flask - Test Collection"
	bash scripts/benchmark.sh fastapi/fastapi requirements-tests.txt
	cargo render-benchmark output/fastapi/fastapi/result.json --title "FastAPI - Test Collection"

bench-update: bench
	@echo "Updating benchmarks..."

format:
	cargo fmt
	cargo clippy --fix --allow-dirty

test:
	cargo test

prepush: format test