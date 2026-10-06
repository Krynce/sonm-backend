# Sonm Backend

Services and libraries that power Sonm. Based on [Revolt / Stoat](https://github.com/stoatchat/stoatchat); the REST and WebSocket APIs stay compatible with the Stoat web client.

## Layout

| Crate | Path | What it does | Port |
|---|---|---|---|
| `sonm-api` | [crates/api](crates/api) | REST API (Rocket) | 14702 |
| `sonm-gateway` | [crates/gateway](crates/gateway) | WebSocket events gateway | 14703 |
| `sonm-files` | [crates/files](crates/files) | File uploads and serving (S3) | 14704 |
| `sonm-embeds` | [crates/embeds](crates/embeds) | Link previews and media proxy | 14705 |
| `sonm-scheduler` | [crates/scheduler](crates/scheduler) | Timed clean-up jobs and ack processing | — |
| `sonm-push` | [crates/push](crates/push) | Web Push notifications (RabbitMQ consumer) | — |
| `sonm-voice` | [crates/voice](crates/voice) | LiveKit webhook receiver | 8500 |

Shared libraries live in `crates/core/*`: `config`, `database` (MongoDB), `models` (API types), `result` (errors), `permissions`, `presence`, `ratelimits`, `parser`, `storage` (S3 + encryption), `coalesced`.

Docker images: `ghcr.io/krynce/sonm-backend/<service>`, built by [.github/workflows/docker.yaml](.github/workflows/docker.yaml) on tag push.

## Development

Requires [mise](https://mise.jdx.dev) and Docker.

```bash
mise install
cp livekit.example.yml livekit.yml
mise start          # docker services + all Sonm services
```

Development dependencies (from `compose.yml`):

| Service | Port |
|---|---|
| MongoDB | 27017 |
| Redis (KeyDB) | 6379 |
| S3 (MinIO-compatible) | 14009, console 14010 |
| RabbitMQ | 5672, UI 15672 |
| Maildev | SMTP 14025, UI 14080 |

Run a single service: `mise service:api`, `mise service:gateway`, … Stop containers: `mise docker:stop`.

### Configuration

Defaults are in `crates/core/config/Sonm.toml`; `Sonm.toml` in the repo root holds dev overrides. Put local changes in `Sonm.overrides.toml` (git-ignored). Any key can be set from the environment with the `SONM__` prefix, e.g. `SONM__DATABASE__MONGODB=mongodb://db`.

For the web client, point it at `http://localhost:14702` (API) and `ws://localhost:14703` (gateway).

### Tests

```bash
mise docker:start
mise test           # cargo nextest, TEST_DB=MONGODB
```

Use `nextest` rather than `cargo test`: config overrides are process-global.

## Migrating from Revolt / Stoat

Internal names changed: MongoDB database `revolt` → `sonm`, RabbitMQ exchanges `revolt.*` → `sonm.*`, S3 bucket `revolt-uploads` → `sonm-uploads`, config `Revolt.toml` → `Sonm.toml`, env prefix `REVOLT__` → `SONM__`. Stop all services and run [scripts/migrate-from-revolt.sh](scripts/migrate-from-revolt.sh).

Mobile push (APNs/FCM) is not supported; only Web Push.

## License

AGPL-3.0-or-later ([LICENSE](LICENSE)), except crates that carry their own MIT `LICENSE` file. Original copyright notices are kept.
