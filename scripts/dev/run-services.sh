#!/bin/sh
# Run the built debug services in one container against `docker compose` deps.
# Prereqs: docker compose up -d redis database minio createbuckets rabbit maildev
#          sh scripts/dev/cargo.sh build --bin sonm-api --bin sonm-gateway --bin sonm-files \
#            --bin sonm-embeds --bin sonm-scheduler --bin sonm-push
# The scheduler must start first: it declares the sonm.default exchange.
ROOT="$(cd "$(dirname "$0")/../.." && { pwd -W 2>/dev/null || pwd; })"
docker rm -f sonm-services >/dev/null 2>&1
MSYS_NO_PATHCONV=1 docker run -d --name sonm-services --network sonm-backend_default \
  -p 14702:14702 -p 14703:14703 -p 14704:14704 -p 14705:14705 \
  -v "$ROOT:/src" -v sonm-target:/target -w /src \
  -e "SONM__DATABASE__MONGODB=mongodb://database/?directConnection=true" \
  -e SONM__DATABASE__REDIS=redis://redis/ -e REDIS_URI=redis://redis/ \
  -e SONM__RABBIT__HOST=rabbit -e SONM__API__SMTP__HOST=maildev -e SONM__API__SMTP__PORT=25 \
  -e SONM__FILES__S3__ENDPOINT=http://minio:9000 -e SONM__FILES__S3__PATH_STYLE_BUCKETS=true \
  -e SONM__FEATURES__WEBHOOKS_ENABLED=true -e ROCKET_ADDRESS=0.0.0.0 -e ROCKET_PORT=14702 -e RUST_LOG=info,rocket=warn \
  sonm-dev sh -c '/target/debug/sonm-scheduler & /target/debug/sonm-push & sleep 5;
    /target/debug/sonm-api & /target/debug/sonm-gateway & /target/debug/sonm-files & /target/debug/sonm-embeds & wait'
