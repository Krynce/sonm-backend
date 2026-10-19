# Деплой фазы 3 (рефакторинг)

Тег `v0.1.6`. Миграций БД нет, шагов «руками» нет, кроме одного: на VPS меняется
`compose.yml` (метки Traefik), а не только `BACKEND_TAG`.

## Что меняется в поведении

- **scheduler чистит просроченные MFA-тикеты** раз в 5 минут (задача существовала, но не
  была подключена). В логах `scheduler` появится строка `Pruned N expired MFA tickets`.
- **Traefik перестаёт отвечать на `/autumn` и `/january`** — алиасы со времён перехода со
  Stoat. Клиент ходит на `/files` и `/embeds` (URL берутся из `GET /`).
- Остальное — рефакторинг без изменения API: те же коды и тексты ошибок, тот же порядок
  проверок (валидация тела, затем права).

## Деплой

```sh
cd /opt/sonm
cp compose.yml compose.yml.pre-v0.1.6
# compose.yml: убрать /autumn и /january (см. diff в коммите фазы 3, deploy/compose.yml)
sed -i 's/^BACKEND_TAG=.*/BACKEND_TAG=v0.1.6/' .env
docker compose config -q
docker compose pull api gateway files embeds push scheduler voice
docker compose up -d api gateway files embeds push scheduler voice
```

`files` и `embeds` пересоздаются в любом случае (новый образ), так что Traefik подхватит
новые метки сам.

## Проверка после деплоя

- [x] `docker compose ps` — все сервисы `healthy`/`running` на `v0.1.6`
- [x] `/files/` и `/embeds/` отвечают своим JSON-приветствием; `/autumn/` и `/january/`
      отдают **200 с SPA**, а не 404: роутер веб-клиента — `Host(DOMAIN)` без префикса, то
      есть catch-all, и снятые алиасы просто проваливаются в него. Проверять надо тело
      ответа (`<!doctype html>` вместо JSON файлового сервера), а не код.
- [x] в логах `scheduler` есть `Pruned … expired MFA tickets`
- [ ] в Chrome: вход, отправка сообщения, закрепление/открепление, вложение — за владельцем
- [x] `docker compose logs api gateway files embeds push scheduler | grep -i panic` пусто

Отдачу настоящего файла через `/files/<tag>/<id>` проверить не удалось: коллекция
`attachments` в прод-БД пуста.

## Откат

```sh
cd /opt/sonm
cp compose.yml.pre-v0.1.6 compose.yml
sed -i 's/^BACKEND_TAG=.*/BACKEND_TAG=v0.1.5/' .env
docker compose up -d api gateway files embeds push scheduler voice
```
