# Переезд chat.krynce.ru со Stoat на Sonm

Данные остаются на месте: та же БД `revolt`, тот же бакет `revolt-uploads` и та же папка `data/`. В БД записана ревизия 54. При старте Sonm выполнит одну миграцию: «revision 54», которая ставит бит `UseExternalEmojis` в `default_permissions` серверов. Миграция идемпотентная, старый Stoat этот бит игнорирует, поэтому откат делается одной командой.

Что меняется для пользователей: бренд и логотип, один раз перезагрузится PWA, на время окна пропадёт связь. Сессии, сообщения, файлы и push-подписки сохраняются.

Ниже `STOAT` — это папка self-hosted на VPS (та, где лежат `compose.yml`, `Revolt.toml`, `secrets.env`, `data/`).

## 1. GitHub и образы (до дня X)

1. Переименовать форки: Settings → Repository name.
   - `Krynce/stoat-backend` → `sonm-backend`
   - `Krynce/stoat-for-web` → `sonm-web`
2. В обоих репозиториях открыть вкладку Actions и включить workflows (в форках они выключены по умолчанию).
3. Запушить локальные репозитории `sonm-backend` и `sonm-web` в `main`.
4. Собрать образы:
   - бэкенд: `git tag v0.1.0 && git push origin v0.1.0` → `ghcr.io/krynce/sonm-backend/<service>:v0.1.0`;
   - веб собирается сам на push в `main` → `ghcr.io/krynce/sonm-web:<sha>` и `:latest`.
5. Пакеты в ghcr приватные. Есть два варианта:
   - на VPS выполнить `docker login ghcr.io -u krynce` с PAT (scope `read:packages`);
   - или сделать пакеты публичными: Package settings → Change visibility.

## 2. Подготовка на VPS (без простоя)

```sh
STOAT=~/stoat            # поправить на реальный путь
mkdir ~/sonm && cd ~/sonm
# сюда положить deploy/compose.yml и deploy/Sonm.toml

# Секреты: REVOLT__PUSHD__* → SONM__PUSH__*, остальное REVOLT__ → SONM__
sed -e 's/^REVOLT__PUSHD__/SONM__PUSH__/' -e 's/^REVOLT__/SONM__/' $STOAT/secrets.env > secrets.env
chmod 600 secrets.env
grep -c '^SONM__' secrets.env   # ожидается 5: VAPID x2, FILES__ENCRYPTION_KEY, LIVEKIT KEY/SECRET

# LiveKit: вебхук теперь идёт в сервис voice
sed 's#http://voice-ingress:8500#http://voice:8500#' $STOAT/livekit.yml > livekit.yml

cat > .env <<EOF
DOMAIN=chat.krynce.ru
DATA_DIR=$STOAT/data
BACKEND_TAG=v0.1.0
WEB_TAG=<sha из Actions sonm-web>
EOF

docker compose config -q && docker compose pull scheduler api gateway files embeds push voice web   # minio/minio пропал с Docker Hub, берётся локальный
```

Сверить `cat $STOAT/Revolt.toml` с `Sonm.toml`:
- если в `Revolt.toml` есть секреты (`encryption_key`, `[pushd.vapid]`, `key`/`secret` у livekit), а в `secrets.env` их нет, перенести их в `secrets.env` как `SONM__...`;
- любые свои секции (smtp, лимиты и т.п.) перенести в `Sonm.toml`. При этом `[pushd]` становится `[push]`, `[january]` становится `[embeds]`.

**Без ключа файлов не откроются вложения, а без VAPID отвалятся push-подписки.**

Бэкап:
- снапшот VPS в панели Timeweb;
- дамп БД: `cd $STOAT && docker compose exec -T database mongodump --archive --gzip > ~/revolt-$(date +%F).archive.gz`.

## 3. Переключение (окно минут на 10)

```sh
cd $STOAT
docker compose stop caddy web api events autumn january gifbox voice-ingress   # трафик больше не принимается
sleep 30                                   # pushd/crond дочитывают очереди
docker compose down                        # обязательно: две Mongo на одних файлах = порча БД
docker compose up -d desktop               # /desktop продолжает отдавать zip

cd ~/sonm
docker compose up -d
docker compose logs -f api scheduler
```

**Стоп-сигнал:** в логах `api` должны быть строки `Running migration [revision 54 ...]` и `Migration complete. Currently at revision 55.`

Если вместо неё написано `Creating database.`, значит, имя БД не подхватилось и сервер видит пустую базу. В этом случае сразу выполнить `docker compose down` и откатиться.

## 4. Проверка

- [ ] `curl -s https://chat.krynce.ru/api/` → в `features` есть `files.url` с `/files`, есть `embeds`, нет `gifs`
- [ ] сайт открывается уже залогиненным (сессия от Stoat), бренд Sonm
- [ ] старые сообщения, аватары и вложения грузятся
- [ ] загрузка нового файла, превью ссылки
- [ ] голосовой звонок с двух устройств
- [ ] push: включить уведомления, отправить сообщение со второго аккаунта
- [ ] `curl -sI https://chat.krynce.ru/autumn/` и `/january/` отвечают (алиасы)
- [ ] `https://chat.krynce.ru/desktop` отдаёт `sonm-setup.exe`

## 5. Откат

```sh
cd ~/sonm && docker compose down
cd $STOAT && docker compose up -d
```

Данные общие и совместимые в обе стороны. Сообщения, написанные на Sonm, сохранятся.

## 6. Потом

- Через 1–2 недели:
  - убрать алиасы `/autumn` и `/january` из `compose.yml`;
  - удалить старые контейнеры и образы `ghcr.io/stoatchat/*`;
  - удалить папку `STOAT`, кроме `data/`, и перенести `data/` в `~/sonm`.
- При переезде БД и файлов на отдельные серверы:
  - переименовать `revolt` → `sonm` и `revolt-uploads` → `sonm-uploads` через `scripts/migrate-from-revolt.sh`;
  - выставить `database.name` и `files.s3.default_bucket` в новые имена.
- `/desktop` отдаёт установщик из `rust-stoat-desktop`. Чтобы обновить: `cargo tauri build`, потом `scp target/release/bundle/nsis/Sonm_*_x64-setup.exe root@VPS:/opt/sonm/sonm-setup.exe`.
