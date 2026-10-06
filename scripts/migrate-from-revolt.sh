#!/bin/sh
# One-off migration of an existing Revolt/Stoat deployment to Sonm names.
# Stop every backend service before running this. The source data is copied,
# not moved, so the old database and bucket stay as a backup.
#
# Required env:
#   MONGODB_URI   e.g. mongodb://localhost:27017
#   S3_ALIAS      mc alias pointing at your S3 (see `mc alias set`)
# Optional env:
#   OLD_DB=revolt NEW_DB=sonm OLD_BUCKET=revolt-uploads NEW_BUCKET=sonm-uploads
set -eu

: "${MONGODB_URI:?set MONGODB_URI}"
: "${S3_ALIAS:?set S3_ALIAS (mc alias)}"
OLD_DB="${OLD_DB:-revolt}"
NEW_DB="${NEW_DB:-sonm}"
OLD_BUCKET="${OLD_BUCKET:-revolt-uploads}"
NEW_BUCKET="${NEW_BUCKET:-sonm-uploads}"

echo "==> MongoDB: copying $OLD_DB -> $NEW_DB"
if [ "$(mongosh "$MONGODB_URI" --quiet --eval "db.getSiblingDB('$NEW_DB').getCollectionNames().length")" != "0" ]; then
  echo "Database $NEW_DB is not empty, refusing to overwrite." >&2
  exit 1
fi
mongodump --uri="$MONGODB_URI" --db="$OLD_DB" --archive |
  mongorestore --uri="$MONGODB_URI" --archive --nsFrom="$OLD_DB.*" --nsTo="$NEW_DB.*"

echo "==> S3: mirroring $OLD_BUCKET -> $NEW_BUCKET"
mc mb --ignore-existing "$S3_ALIAS/$NEW_BUCKET"
mc mirror --preserve "$S3_ALIAS/$OLD_BUCKET" "$S3_ALIAS/$NEW_BUCKET"

echo "==> MongoDB: pointing stored file hashes at $NEW_BUCKET"
mongosh "$MONGODB_URI" --quiet --eval "
  const r = db.getSiblingDB('$NEW_DB').attachment_hashes.updateMany(
    { bucket_id: '$OLD_BUCKET' }, { \$set: { bucket_id: '$NEW_BUCKET' } });
  print('updated ' + r.modifiedCount + ' file hashes');"

cat <<EOF

Done. Before starting Sonm services:
  * rename Revolt.toml -> Sonm.toml (and Revolt.overrides.toml -> Sonm.overrides.toml)
  * rename env vars REVOLT__* -> SONM__*
  * in the config, rename sections: [pushd] -> [push], [january] -> [embeds];
    hosts keys events/autumn/january -> gateway/files/embeds;
    sentry keys events/proxy/pushd/crond/gifbox/voice_ingress -> gateway/embeds/push/scheduler/(removed)/voice;
    remove [pushd.fcm], [pushd.apn], ack_queue and voso_legacy*
  * set database.name = "$NEW_DB" and files.s3.default_bucket = "$NEW_BUCKET"
  * if you overrode rabbit.default_exchange or pushd.exchange, change them to sonm.default / sonm.notifications
  * RabbitMQ: let the old services drain queues bound to revolt.default / revolt.notifications;
    Sonm declares sonm.default / sonm.notifications on start. Old exchanges can then be deleted.
EOF
