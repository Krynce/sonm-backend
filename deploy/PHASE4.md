# Деплой фазы 4 (CI, инфраструктура, обновление зависимостей)

Тег `v0.1.7`. Миграций БД нет. `compose.yml` на VPS **не меняется** — только `BACKEND_TAG`.

## Что меняется в поведении

- **gateway пишет сводку раз в минуту:** строка вида
  `gateway: N connections (+M accepted), K subscriptions, R ready (avg Xms), E events sent,
  errors: a decode / b send / c read`. Это единственная наблюдаемость у gateway.
- **Sentry больше не заливается пер-ивентом.** Было четыре `capture_message` на каждый сбой
  события и на каждое отключение клиента; стало максимум один агрегированный вызов за
  минуту, и только если за эту минуту были ошибки.
- **API не менялось:** те же маршруты, коды и тексты ошибок.

Под капотом — обновление зависимостей, которое стоит знать при разборе инцидента:

| Что | Было → стало | Чем рискует |
|---|---|---|
| тулчейн | 1.92.0 → 1.99.1 | пересобрано всё |
| WebSocket в gateway | async-tungstenite 0.17 → 0.35 | подключение клиентов |
| TLS на пути к S3 | rustls 0.23.41 → 0.23.46, выключен legacy-коннектор aws-sdk | загрузка файлов |
| почта | lettre 0.10 → 0.11 | письма подтверждения и сброса пароля |
| валидация | validator 0.16 → 0.21 | отказы на роутах с regex |
| SSRF-блоклист embeds | pdk-ip-filter-lib → ipnet | превью ссылок |
| профиль release | добавлен `codegen-units = 1` | только время сборки |

Образы сервисов теперь собираются `FROM` базы, тегированной по sha коммита, так что образы
тега гарантированно собраны из этого коммита (раньше `base:latest` был мутабельным, и два
пересекающихся пуша тегов могли смешать бинарники).

## Деплой

```sh
cd /opt/sonm
sed -i 's/^BACKEND_TAG=.*/BACKEND_TAG=v0.1.7/' .env
docker compose config -q
docker compose pull api gateway files embeds push scheduler voice
docker compose up -d api gateway files embeds push scheduler voice
```

## Проверка после деплоя

Порядок — по убыванию риска, первым то, что меняли глубже всего.

- [ ] `docker compose ps` — все сервисы `healthy`/`running` на `v0.1.7`
- [ ] **gateway: живые клиенты переподключились.** В логах `gateway` через минуту должна быть
      сводка с `connections` > 0 и `errors: 0 decode / 0 send / 0 read`. Это главная проверка:
      менялась сама WebSocket-библиотека, а автотестов на gateway нет.
      `docker compose logs --tail 50 gateway | grep metrics`
- [ ] **файлы:** загрузить вложение через клиент — это путь через S3 и обновлённый TLS-стек
- [ ] **почта:** запросить сброс пароля — это `lettre` 0.11
- [ ] **превью ссылки:** вставить внешнюю ссылку в сообщение — это новый блоклист на `ipnet`;
      заодно убедиться, что приватный адрес (например `http://127.0.0.1/`) по-прежнему
      отклоняется
- [ ] **валидация:** попробовать невалидное имя пользователя или цвет роли — должен прийти
      прежний отказ, а не 500
- [ ] `docker compose logs api gateway files embeds push scheduler | grep -i panic` пусто
- [ ] в Chrome: вход, отправка сообщения, вложение — за владельцем

## Откат

`compose.yml` не менялся, поэтому откат — только тег:

```sh
cd /opt/sonm
sed -i 's/^BACKEND_TAG=.*/BACKEND_TAG=v0.1.6/' .env
docker compose up -d api gateway files embeds push scheduler voice
```
