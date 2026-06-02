#!/usr/bin/env bash
# Fail if the given version is already published on PyPI for project arcflow.
set -euo pipefail

VERSION="${VERSION:-}"
PROJECT="${PYPI_PROJECT:-arcflow}"
if [[ -z "$VERSION" ]]; then
  echo "ERROR: set VERSION (e.g. 0.3.0)"
  exit 1
fi

URL="https://pypi.org/pypi/${PROJECT}/json"
HTTP_CODE="$(curl -sS -o /tmp/pypi-arcflow.json -w "%{http_code}" "$URL" || true)"

if [[ "$HTTP_CODE" == "404" ]]; then
  echo "OK: project ${PROJECT} not on PyPI yet (or no releases)"
  exit 0
fi

if [[ "$HTTP_CODE" != "200" ]]; then
  echo "ERROR: PyPI API returned HTTP $HTTP_CODE for $URL"
  exit 1
fi

if python3 -c "
import json, sys
data = json.load(open('/tmp/pypi-arcflow.json'))
releases = data.get('releases', {})
if '${VERSION}' in releases and releases['${VERSION}']:
    sys.exit(1)
"; then
  echo "OK: version ${VERSION} is not published on PyPI for ${PROJECT}"
else
  echo "ERROR: version ${VERSION} already exists on PyPI for ${PROJECT}"
  exit 1
fi
