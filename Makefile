.PHONY: develop build publish test

# Convenience targets for working with the project using uv and maturin.
develop:
	uv run --extra dev maturin develop

build:
	uv run --extra dev maturin build --release

publish:
	uv run --extra dev maturin publish --skip-existing

test:
	uv run --extra dev pytest
