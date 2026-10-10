# Деплой фазы 0 (предохранители)

Шаги, которые нельзя сделать из кода: выполняются на VPS руками один раз при
раскатке фазы 0 из [BACKEND_OPTIMIZATION_PLAN.md](../../BACKEND_OPTIMIZATION_PLAN.md).
Порядок важен: пункт 1 — **до** старта нового `push`.

## 1. Удалить очереди notifications (обязательно, иначе push не поднимется)

Теперь `push` объявляет очереди с `x-dead-letter-exchange`. RabbitMQ не позволяет
переобъявить существующую очередь с другими аргументами — отвечает 406
PRECONDITION_FAILED, канал падает, сервис уходит в рестарт-луп.

Очереди пустые (до фазы 0 стояло `no_ack: true`, сообщения разбирались мгновенно),
так что удаление ничего не теряет. Делать при **остановленном** push:

```sh
cd /opt/sonm
docker compose stop push
for q in notifications.generic notifications.message notifications.fr_received \
         notifications.fr_accepted notifications.mass_mention notifications.dm_call \
         notifications.outbound.vapid; do
  docker compose exec -T rabbit rabbitmqctl delete_queue "${q}-prd" || true
done
```

Имена очередей — из `[push]` в `Sonm.toml`, суффикс `-prd` добавляет сам сервис
(`production = true`). После деплоя появятся они же плюс `sonm.notifications-dlq`
(имя = `push.exchange` + `-dlq`), куда уходят необработанные уведомления:

```sh
docker compose exec -T rabbit rabbitmqctl list_queues name messages
```

Непустой DLQ = push-уведомления, которые не удалось разобрать: смотреть логи `push`.

## 2. Освободить диск перед рестартом

```sh
docker system prune -a --volumes=false   # ~7 ГБ образов + 1.4 ГБ build-кэша
```

## 3. Бэкапы

```sh
install -m 755 backup.sh /opt/sonm/backup.sh
cd /opt/sonm && ./backup.sh            # первый прогон вручную
(crontab -l 2>/dev/null; echo '17 4 * * * cd /opt/sonm && ./backup.sh >> /var/log/sonm-backup.log 2>&1') | crontab -
```

Проверка восстановления в отдельную БД — в комментарии внутри `backup.sh`.
Сделать один раз, пока база маленькая (0.26 МБ).

## 4. Проверка после деплоя

- [ ] `docker compose ps` — у `api`, `gateway`, `files`, `embeds`, `voice` статус `healthy`
      (у `push` и `scheduler` healthcheck'а нет: они умирают при обрыве AMQP и
      поднимаются через `restart: always`)
- [ ] `docker stats --no-stream` — у контейнеров виден LIMIT
- [ ] `docker compose restart rabbit` → `api` и `push` перезапускаются сами и
      продолжают работать; в логах `Lost connection to RabbitMQ, exiting`
- [ ] `curl -s "https://chat.krynce.ru/embeds/proxy?url=<ссылка на файл >20 МБ"` → ошибка,
      а не съеденная память
- [ ] `docker compose exec -T rabbit rabbitmqctl status | grep -A2 memory` — watermark 256 MiB
- [ ] `docker compose logs --tail 5 database` — лог жив, файл лога не растёт бесконечно
      (`docker system df` / `du -sh /var/lib/docker/containers`)
