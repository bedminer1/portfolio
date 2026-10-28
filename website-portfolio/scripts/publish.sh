#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
: "${SITE_ROOT:?Set SITE_ROOT to the static Caddy web root, normally /var/www/portfolio}"
if [[ "$SITE_ROOT" != /var/www/portfolio ]]; then
  echo "Unexpected SITE_ROOT: $SITE_ROOT" >&2
  exit 1
fi
npm run build
install -d -m 755 /var/backups/portfolio "$SITE_ROOT"
stamp=$(date -u +%Y%m%d-%H%M%S)
tar -C /var/www -czf "/var/backups/portfolio/portfolio-${stamp}.tar.gz" portfolio
rsync -a --delete --exclude='*.map' build/ "$SITE_ROOT/"
test -s "$SITE_ROOT/index.html"
