# Деплой фазы 1 (panic-поверхность и безопасность)

> **Статус: выполнено 2026-10-10** на `/opt/sonm`, тег `v0.1.3`. Пункты 1, 1a и 2 пройдены,
> креды MinIO и RabbitMQ ротированы — значения `minioautumn`/`rabbitpass` ниже оставлены
> как история, они больше не действуют. Документ нужен при переносе стека на новый хост.
>
> **Грабли:** `docker compose exec -T` читает stdin, поэтому внутри `ssh host 'bash -s' <<EOF`
> он съедает остаток скрипта. Каждому `exec` нужен `</dev/null`, иначе половина шагов
> молча не выполнится (у меня так пароль сменился в брокере, но не в `.env`).

Шаги, которые нельзя сделать из кода: выполняются на VPS руками при раскатке фазы 1
из [BACKEND_OPTIMIZATION_PLAN.md](../../BACKEND_OPTIMIZATION_PLAN.md).
Пункт 1 — **до** `docker compose up`, иначе compose откажется стартовать
(`${MINIO_ROOT_USER:?}` и `${RABBIT_USER:?}` теперь обязательны).

## 1. Креды MinIO и RabbitMQ — из репозитория в `.env`/`secrets.env`

Из `compose.yml` и `Sonm.toml` убраны `minioautumn`/`minioautumn` и
`rabbituser`/`rabbitpass`. Текущие (старые) значения нужно один раз переложить:

```sh
cd /opt/sonm
cat >> .env <<'EOF'
MINIO_ROOT_USER=minioautumn
MINIO_ROOT_PASSWORD=minioautumn
RABBIT_USER=rabbituser
RABBIT_PASS=rabbitpass
EOF
cat >> secrets.env <<'EOF'
SONM__FILES__S3__ACCESS_KEY_ID=minioautumn
SONM__FILES__S3__SECRET_ACCESS_KEY=minioautumn
SONM__RABBIT__USERNAME=rabbituser
SONM__RABBIT__PASSWORD=rabbitpass
EOF
chmod 600 .env secrets.env
```

`SONM__RABBIT__*` дописывать только если их там ещё нет — проверить
`grep RABBIT secrets.env`, дубль перебьёт прежнее значение.

После этого стек поднимается на прежних кредах, то есть деплой безопасен сам по себе.
Значения в `.env` и `secrets.env` должны совпадать: первое — то, с чем стартуют
MinIO и RabbitMQ, второе — то, с чем к ним подключается бэкенд.

## 1a. Проверить, что прод не сидит на дефолтных секретах из репозитория

`crates/core/config/Sonm.toml` (вкомпилирован в бинарь как слой дефолтов) содержит рабочие
**публично известные** значения: VAPID `private_key`, `files.encryption_key`, S3
`minioadmin`. Если какое-то из них не перекрыто в `secrets.env`, прод работает на нём.

```sh
cd /opt/sonm
grep -c SONM__PUSH__VAPID__PRIVATE_KEY secrets.env    # ожидается 1
grep -c SONM__FILES__ENCRYPTION_KEY secrets.env       # ожидается 1
grep -c SONM__API__LIVEKIT__NODES__WORLDWIDE__SECRET secrets.env  # ожидается 1
```

Ноль — секрет дефолтный. `encryption_key` менять **нельзя** на живых данных (старые файлы
расшифровываются только им); остальные меняются свободно. Дефолтный LiveKit-секрет —
`ZjCofRlfm6GGtjlifmNpCDkcQbEIIVC0` из `livekit.example.yml`, он же должен стоять в
`livekit.yml` на хосте.

## 2. Сменить креды (отдельным шагом, после успешного деплоя)

Старые значения лежали в git, их надо считать скомпрометированными. Оба сервиса
доступны только из docker-сети, поэтому это не аварийная, но обязательная уборка.

```sh
cd /opt/sonm
NEW_MINIO_PASS=$(openssl rand -hex 24)
NEW_RABBIT_PASS=$(openssl rand -hex 24)
```

RabbitMQ (пользователь живёт в его базе, `RABBITMQ_DEFAULT_*` действует только при
первом запуске — менять надо через `rabbitmqctl`):

```sh
docker compose exec -T rabbit rabbitmqctl change_password rabbituser "$NEW_RABBIT_PASS" </dev/null
sed -i "s/^RABBIT_PASS=.*/RABBIT_PASS=$NEW_RABBIT_PASS/" .env
sed -i "s/^SONM__RABBIT__PASSWORD=.*/SONM__RABBIT__PASSWORD=$NEW_RABBIT_PASS/" secrets.env
docker compose up -d api gateway push scheduler voice
```

MinIO: root-креды в образе `minio/minio` меняются только перезапуском с новыми
`MINIO_ROOT_*` (данные в `/data` при этом не трогаются):

```sh
sed -i "s/^MINIO_ROOT_PASSWORD=.*/MINIO_ROOT_PASSWORD=$NEW_MINIO_PASS/" .env
sed -i "s/^SONM__FILES__S3__SECRET_ACCESS_KEY=.*/SONM__FILES__S3__SECRET_ACCESS_KEY=$NEW_MINIO_PASS/" secrets.env
docker compose up -d minio && docker compose restart files api
```

Проверить сразу же: загрузка аватара и открытие уже загруженного вложения.
Если `files` отвечает 403 — креды разъехались, вернуть прежние из `.env`.

## 3. Проверка после деплоя

- [ ] `docker compose ps` — все контейнеры `healthy`/`running`
- [ ] загрузка файла и открытие старого вложения работают (S3-креды сошлись)
- [ ] сообщения в канале отправляются и помечаются прочитанными (RabbitMQ-креды сошлись)
- [ ] SSRF: все три должны вернуть ошибку, а не содержимое —
      ```sh
      for u in http://169.254.169.254/ http://100.64.0.1/ http://localtest.me/; do
        curl -s -o /dev/null -w "$u -> %{http_code}\n" \
          "https://chat.krynce.ru/embeds/proxy?url=$u"
      done
      ```
      (`localtest.me` резолвится в `127.0.0.1` — проверяет фильтр в резолвере)
- [ ] тело ошибки API больше не содержит путей исходников:
      `curl -s https://chat.krynce.ru/api/users/000000000000000000000000 | grep -c crates` → `0`
- [ ] GitHub-вебхук с кириллицей в commit message не роняет api
      (`docker compose logs api | grep -i panic` пусто)
