# Thin wrappers for local Python SDK release checks (publish is GitHub OIDC only).
ROOT := $(abspath ..)
SCRIPTS := $(ROOT)/scripts

.PHONY: verify-tag check-pypi build-wheels lint test-publish-gate

verify-tag:
	@test -n "$(TAG)" || (echo "Usage: make verify-tag TAG=sdk-python/v0.3.0" && exit 1)
	TAG="$(TAG)" bash "$(SCRIPTS)/verify-python-sdk-tag.sh"

check-pypi:
	@test -n "$(VERSION)" || (echo "Usage: make check-pypi VERSION=0.3.0" && exit 1)
	VERSION="$(VERSION)" bash "$(SCRIPTS)/check-pypi-version-absent.sh"

build-wheels:
	bash "$(SCRIPTS)/build-python-sdk-wheels.sh"

lint:
	python -m ruff check arcflow tests
	python -m black --check arcflow tests
	python -m mypy arcflow

test-publish-gate:
	python -m pip install --upgrade pip maturin pytest "mypy<1.19" black ruff pytest-httpserver
	python -m maturin build --release --out dist
	python -m pip install dist/arcflow_sdk-*.whl
	@tmpdir=$$(mktemp -d 2>/dev/null || python -c "import tempfile; print(tempfile.mkdtemp())"); \
	cd "$$tmpdir" && PYTHONPATH="" python -m pytest "$(CURDIR)/tests" -v
