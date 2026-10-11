# Деплой фазы 2 (производительность)

Тег `v0.1.4`. Отличие от фаз 0 и 1: шагов «руками» нет, всё внутри образов —
но есть **миграция БД** и несколько изменений поведения, которые видно только на проде.

## Что меняется в поведении

- **Миграция, ревизия 55 → 56.** `api` при старте создаёт индексы `channels.recipients`
  и `channels.user`. Коллекция маленькая (десятки документов), блокировка на доли секунды.
  Миграции запускает **только `api`** (`crates/api/src/main.rs`), так что порядок запуска
  сервисов значения не имеет.
- **`GET /` кэшируется в процессе** и отдаётся с `Cache-Control: max-age=60`. Изменение
  конфига теперь видно в ответе только после `docker compose restart api`.
- **Пул MongoDB** 20 на сервис (было 10 по умолчанию; 50 не взял — шесть сервисов дали бы
  потолок 300 соединений при лимите Mongo ~409 и `mem_limit: 640m`, это риск того самого
  OOM, от которого страховала фаза 0), `min_pool_size=5` — на старте `api` держит
  5 соединений вместо ноля. `server_selection_timeout` 5 с вместо 30: при недоступной БД
  запросы будут падать быстро, а не висеть полминуты.
- **`ETag`/`304` в `files`.** Клиенты, у которых файл уже в кэше, получат 304 — в логах
  `files` станет меньше строк о скачивании из S3.
- **Поиск участников стал регистронезависимым** (`members_experimental_query`).
- **Логи presence тише:** создание и удаление сессии ушли в `debug!`. Если presence
  понадобится отлаживать — поднять уровень через `RUST_LOG`.

## Деплой

```sh
cd /opt/sonm
sed -i 's/^BACKEND_TAG=.*/BACKEND_TAG=v0.1.4/' .env
docker compose pull api gateway files embeds push scheduler voice
docker compose up -d api gateway files embeds push scheduler voice
```

`api` поднять можно первым и отдельно — чтобы миграция прошла до того, как остальные
начнут работать с БД:

```sh
docker compose up -d api && docker compose logs -f api | head -40
```

## Проверка после деплоя

- [ ] `docker compose ps` — все контейнеры `healthy`/`running`
- [ ] миграция прошла: ревизия 56 и индексы на месте
      ```sh
      docker compose exec -T database mongosh --quiet revolt --eval \
        'print(db.migrations.findOne().revision); print(db.channels.getIndexes().map(i=>i.name).join(","))' </dev/null
      ```
      ожидается `56` и `_id_,recipients,user`
- [ ] **регрессия 2.1 — главное:** участник, у которого роль закрывает часть каналов,
      после переподключения видит правильный набор **сразу**, без перезагрузки клиента.
      Это та самая ошибка, которую фаза 2 починила: раньше роль-оверрайды при сборке
      Ready не учитывались.
- [ ] `GET /` отвечает и содержит `Cache-Control`:
      `curl -sI https://chat.krynce.ru/api/ | grep -i cache-control`
- [ ] логин работает (argon2 ушёл в `spawn_blocking`) и смена пароля тоже
- [ ] загрузка файла 20 МБ и большого PNG; `docker stats files` во время загрузки —
      память не должна улетать, воркеры не залипают
- [ ] большой PNG (например 20000×20000 в пределах байтового лимита) теперь
      отбивается с `ImageProcessingFailed`, а не декодируется
- [ ] `ETag` отдаётся и 304 работает:
      ```sh
      U=https://chat.krynce.ru/files/avatars/<id>
      E=$(curl -sI "$U" | tr -d '\r' | awk -F': ' '/^etag/I{print $2}')
      curl -s -o /dev/null -w "%{http_code}\n" -H "If-None-Match: $E" "$U"   # 304
      ```
- [ ] голос: подключиться к голосовому каналу, подвигать участника между каналами,
      снять `can_publish` — ни одна из веток не должна отдавать 500
      (здесь же проверяются убранные `get_channel_node(...).unwrap()`)
- [ ] `docker compose logs api gateway files | grep -i panic` пусто

## Замеры, ради которых фаза делалась

Число запросов к Mongo на одно подключение к gateway — до и после:

```sh
docker compose exec -T database mongosh --quiet revolt --eval 'db.setProfilingLevel(1,{slowms:0})' </dev/null
# переподключиться клиентом, затем:
docker compose exec -T database mongosh --quiet revolt --eval \
  'print(db.system.profile.countDocuments({ns:"revolt.server_members",op:"query"}))' </dev/null
docker compose exec -T database mongosh --quiet revolt --eval 'db.setProfilingLevel(0)' </dev/null
```

Профайлер **обязательно выключить** — при `slowms:0` он пишет каждый запрос и съедает диск.

## Откат

Миграция только добавляет индексы, старый код их просто не использует — откат
безопасен и бэкап восстанавливать не нужно:

```sh
cd /opt/sonm
sed -i 's/^BACKEND_TAG=.*/BACKEND_TAG=v0.1.3/' .env
docker compose up -d api gateway files embeds push scheduler voice
```

Ревизия в БД останется 56; на `v0.1.3` блок `if revision <= 55` просто не выполнится.
Индексы при необходимости снимаются вручную:
`db.channels.dropIndex("recipients")`, `db.channels.dropIndex("user")`.
