#!/usr/bin/env bash
# Бэкап MongoDB и MinIO для прод-стека Sonm.
#
# Установка на VPS (в /opt/sonm лежит compose.yml и .env с DATA_DIR):
#
#   install -m 755 backup.sh /opt/sonm/backup.sh
#   (crontab -l 2>/dev/null; echo '17 4 * * * cd /opt/sonm && ./backup.sh >> /var/log/sonm-backup.log 2>&1') | crontab -
#
# Данные лежат не в /opt/sonm, а по старому Stoat-пути (DATA_DIR из .env, сейчас
# /opt/stoat/data) — бэкап читает его оттуда же, куда смотрит compose.
#
# Проверка восстановления в отдельную БД (не трогая прод):
#
#   gzip -dc /opt/stoat/data/backups/mongo-YYYY-mm-dd.archive.gz \
#     | docker compose exec -T database mongorestore --archive --nsFrom 'revolt.*' --nsTo 'restore_test.*'
#   docker compose exec -T database mongosh restore_test --quiet --eval 'db.getCollectionNames()'
#   docker compose exec -T database mongosh restore_test --quiet --eval 'db.dropDatabase()'

set -euo pipefail

cd "$(dirname "$0")"

# shellcheck disable=SC1091
[ -f .env ] && . ./.env

DATA_DIR=${DATA_DIR:?DATA_DIR is not set}
KEEP_DAYS=${KEEP_DAYS:-7}
BACKUP_DIR="${DATA_DIR}/backups"
DATE=$(date +%F)

mkdir -p "$BACKUP_DIR"

# Mongo: поток прямо на хост, чтобы не занимать место внутри контейнера.
docker compose exec -T database mongodump --archive --quiet \
    | gzip > "${BACKUP_DIR}/mongo-${DATE}.archive.gz.part"
mv "${BACKUP_DIR}/mongo-${DATE}.archive.gz.part" "${BACKUP_DIR}/mongo-${DATE}.archive.gz"

# MinIO: файлы лежат на хосте, достаточно снимка каталога.
tar -czf "${BACKUP_DIR}/minio-${DATE}.tar.gz.part" -C "$DATA_DIR" minio
mv "${BACKUP_DIR}/minio-${DATE}.tar.gz.part" "${BACKUP_DIR}/minio-${DATE}.tar.gz"

# Образ minio/minio пропал из Docker Hub и существует только на этом VPS.
if [ ! -f "${BACKUP_DIR}/minio-image.tar.gz" ]; then
    docker save minio/minio | gzip > "${BACKUP_DIR}/minio-image.tar.gz"
fi

find "$BACKUP_DIR" -maxdepth 1 -name '*-????-??-??.*' -mtime "+${KEEP_DAYS}" -delete

ls -la "$BACKUP_DIR"
