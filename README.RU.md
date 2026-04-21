# TeknoMW3 — Rust workspace (RU)

Rust-реализация инструментов **Modern Warfare 3** (TeknoGods): TCP master server, UDP-запрос сервера, лаунчер дедикейтда и утилиты для `teknogods.ini` / аргументов запуска. Лицензия workspace: **GPL-3.0-only** (см. `Cargo.toml`).

## Состав

| Крейт | Бинарник | Назначение |
|-------|----------|------------|
| `mw3-protocol` | — | Типы TCP (BOOB/COKE), UDP query/info, константы. |
| `mw3-master` | `mw3-master` | Master server: регистрация, списки по версии, таймаут записей. |
| `mw3-query-cli` | `mw3-query-cli` | Отправка `MW3_SERVER_QUERY`, вывод строки информации о сервере. |
| `mw3-dedi-launch` | `mw3-dedi-launch` | Поиск строки `steam_api.dll` в PE; на Windows — запуск с патчем памяти. |
| `mw3-loader` | `mw3-loader` | INI, сборка argv для игры, опциональная проверка обновлений по HTTP. |

## Что чем запускается (важно)

| Что вы хотите | Какой инструмент | Комментарий |
|---------------|------------------|-------------|
| **Список серверов в интернете** (регистрация / выдача IP) | `mw3-master` | Это **не** процесс игры. Слушает TCP (по умолчанию порт **27017**). Клиенты и дедик шлют сюда пакеты BOOB/COKE. |
| **Игровой dedicated-сервер** (`iw5mp_server.exe` + подмена DLL в памяти) | `mw3-dedi-launch` | Только **Windows**. Запускать из **папки с игрой**, рядом должны быть `iw5mp_server.exe`, `TeknoMW3.dll` и остальные файлы модификации. |
| **Клиент MW3** (`iw5mp.exe`) | Rust-лоадер **не запускает** | `mw3-loader` лишь **печатает** строку аргументов или правит `teknogods.ini`. Саму игру нужно стартовать вручную или через старый GUI-лаунчер `mw3_loader/` в этом репозитории. |
| **Проверить, «жив» ли UDP-порт сервера** | `mw3-query-cli` | Шлёт запрос на **игровой** UDP-порт сервера (не порт master 27017). |

### Запуск dedicated-сервера (Windows)

1. Соберите или скопируйте `mw3-dedi-launch.exe` в каталог с игрой (где лежит `iw5mp_server.exe`).
2. Убедитесь, что **`TeknoMW3.dll`** на месте (имя можно сменить флагом `--dll`).
3. Пример командной строки (порт игры **27015**, без `+usekeys`):

```text
mw3-dedi-launch.exe iw5mp_server.exe +set dedicated 1 +set net_port 27015
```

С `+usekeys` (как в старом лаунчере) аргументы должны начинаться с `+usekeys`, затем остальное:

```text
mw3-dedi-launch.exe iw5mp_server.exe +usekeys +set dedicated 1 +set net_port 27015
```

Чтобы **только посмотреть** виртуальный адрес патча в PE (без запуска процесса), на любой ОС:

```text
mw3-dedi-launch.exe --print-va iw5mp_server.exe
```

### «Лоадер» `mw3-loader` — это не окно с кнопками

Он **не** вызывает `iw5mp.exe`. Примеры:

- Показать строку аргументов для дедика (скопировать в документацию или сравнить с `mw3-dedi-launch`):

```bash
mw3-loader args-dedicated 27015
mw3-loader args-dedicated 27015 --usekeys
```

- Аргументы для **клиента** в LAN: `mw3-loader args-client-lan` или с ключами: `mw3-loader args-client-lan --usekeys`.
- Подключиться к серверу: `mw3-loader args-client-connect 192.168.0.10 27015` → в выводе будет что-то вроде `+server 192.168.0.10:27015`. Дальше вручную:

```text
iw5mp.exe +server 192.168.0.10:27015
```

(плюс при необходимости `+usekeys` и путь к `TeknoMW3.dll` по инструкции мода).

Графический лаунчер с кнопками по-прежнему в проекте: **`mw3_loader/`** (WPF / .NET), это отдельно от Rust `mw3-loader`.

## Требования

- **Rust** stable, edition **2021**.
- Полный функционал **`mw3-dedi-launch`** (патч процесса) — **только Windows**.
- На **Linux/macOS** доступны тесты, `mw3-master`, `mw3-query-cli`, большая часть `mw3-loader`; у дедик-лаунчера — как минимум `cargo run -p mw3-dedi-launch -- --print-va путь\к\iw5mp_server.exe`.

## Сборка и проверка

```bash
cd rust
cargo test
cargo clippy -- -D warnings
cargo build --release
```

Подробные команды и релизные заметки — в английском **[README.md](README.md)**.

## Документация

- **[docs/RUNBOOK.ru.md](docs/RUNBOOK.ru.md)** — флоу: мастер-сервер → dedicated → клиент (русский).
- **[docs/CONTRACTS.md](docs/CONTRACTS.md)** — форматы протокола и конфигурации (English).
- **[docs/PROGRESS.md](docs/PROGRESS.md)** — статус и планы (English).

Расширенный план миграции всего репозитория (включая диаграммы) на русском лежит в корне проекта: `docs/RUST_MIGRATION_PLAN.md` — это **не** часть английского README, а материал для мейнтейнеров.

## Лицензия

GPL-3.0-only.
