#!/usr/bin/env bash
# Verify git tag sdk-python/vX.Y.Z matches sdk-python/pyproject.toml version.
set -euo pipefail

TAG="${TAG:-${GITHUB_REF_NAME:-}}"
if [[ -z "$TAG" ]]; then
  echo "ERROR: set TAG (e.g. sdk-python/v0.3.0) or run from a tag push workflow"
  exit 1
fi

PREFIX="sdk-python/v"
if [[ "$TAG" != "${PREFIX}"* ]]; then
  echo "ERROR: tag must start with ${PREFIX}, got: $TAG"
  exit 1
fi

VERSION="${TAG#${PREFIX}}"
if [[ ! "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.]+)?$ ]]; then
  echo "ERROR: invalid semver in tag: $VERSION"
  exit 1
fi

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PYPROJECT="${ROOT}/sdk-python/pyproject.toml"
if [[ ! -f "$PYPROJECT" ]]; then
  echo "ERROR: missing $PYPROJECT"
  exit 1
fi

PY_VERSION="$(grep -E '^version = ' "$PYPROJECT" | head -1 | sed -E 's/^version = "(.*)"/\1/')"
if [[ "$PY_VERSION" != "$VERSION" ]]; then
  echo "ERROR: tag version $VERSION != pyproject version $PY_VERSION"
  exit 1
fi

echo "OK: tag $TAG matches pyproject version $PY_VERSION"
echo "VERSION=$VERSION"
