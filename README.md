# TeknoMW3 — Rust workspace

Rust implementation of TeknoGods **Modern Warfare 3** server tooling: TCP master list, UDP server query, dedicated launcher, and small launcher utilities. The workspace is **GPL-3.0-only** (see `Cargo.toml`).

## Contents

| Crate | Binary | Role |
|-------|--------|------|
| `mw3-protocol` | — (library) | TCP master messages (BOOB/COKE), UDP query/info types, constants. |
| `mw3-master` | `mw3-master` | TCP master server: registration, version-scoped lists, stale purge. |
| `mw3-query-cli` | `mw3-query-cli` | Send `MW3_SERVER_QUERY`, print server info string from the response. |
| `mw3-dedi-launch` | `mw3-dedi-launch` | Find `steam_api.dll` string in `iw5mp_server.exe`; on Windows, suspended launch + patch + resume. |
| `mw3-loader` | `mw3-loader` | `teknogods.ini` helpers, game argv builders, optional HTTP update-check. |

### What actually starts the game?

| Goal | Tool | Note |
|------|------|------|
| Internet **server list** (register / browse) | `mw3-master` | **Not** the game. TCP service (default **27017**). |
| **Dedicated** `iw5mp_server.exe` with in-memory DLL patch | `mw3-dedi-launch` | **Windows only.** Run from the **game directory** next to `iw5mp_server.exe` and `TeknoMW3.dll`. |
| **Client** `iw5mp.exe` | Rust `mw3-loader` does **not** spawn it | `mw3-loader` only **prints** argv or edits `teknogods.ini`. Start the exe yourself or use the legacy GUI under `mw3_loader/`. |
| Probe a server’s **game UDP** port | `mw3-query-cli` | Not the master TCP port. |

**Dedicated example (Windows, from game folder):**

```text
mw3-dedi-launch.exe iw5mp_server.exe +set dedicated 1 +set net_port 27015
```

With `+usekeys` first: `... iw5mp_server.exe +usekeys +set dedicated 1 +set net_port 27015`.

**Preview argv only (any OS):** `mw3-loader args-dedicated 27015` — compare with what you pass to `mw3-dedi-launch`.

## Requirements

- **Rust** stable, **2021** edition (`rustup` recommended).
- **Windows** (full): `mw3-dedi-launch` process patch, game binaries.  
- **Linux / macOS**: `cargo test`, `mw3-master`, `mw3-query-cli`, `mw3-loader` (incl. `ini-*`, `args-*`, `update-parse`); `mw3-dedi-launch --print-va` on PE files; full dedi launch is Windows-only.

## Quick start

```bash
cd rust

# Verify the workspace
cargo test
cargo clippy -- -D warnings

# Master server (default TCP **27017**)
cargo run -p mw3-master -- --bind 0.0.0.0:27017

# UDP query (replace host/port)
cargo run -p mw3-query-cli -- --target 192.0.2.1:27015 --bind 0.0.0.0:0

# Dedicated: show virtual address from PE (no game start)
cargo run -p mw3-dedi-launch -- --print-va /path/to/iw5mp_server.exe

# Launcher helpers
cargo run -p mw3-loader -- --help
```

## Release builds

```bash
cd rust
cargo build --release -p mw3-master -p mw3-query-cli -p mw3-dedi-launch -p mw3-loader
```

Artifacts land under `target/release/`. For Windows cross-build from Linux you need the appropriate `rustup target` (e.g. `x86_64-pc-windows-msvc` or `gnu`) and a working linker.

### Windows smoke checklist

1. `mw3-master`: bind port, register a server from the game or a test client, request list.
2. `mw3-query-cli`: query a running game server UDP port.
3. `mw3-dedi-launch`: from the game directory, run without `--print-va` with valid `iw5mp_server.exe` and DLL; confirm process starts.
4. `mw3-loader`: `ini-dump` / `args-dedicated` / `update-check` (network).

## Documentation

| Document | Description |
|----------|-------------|
| [docs/CONTRACTS.md](docs/CONTRACTS.md) | Wire formats, `teknogods.ini` keys, DLL names — English reference. |
| [docs/PROGRESS.md](docs/PROGRESS.md) | Release status, roadmap, phase checklist. |
| [docs/README.md](docs/README.md) | Index of files under `rust/docs/`. |

Russian overview for this tree: [README.RU.md](README.RU.md).

## License

GPL-3.0-only, in line with the parent TeknoMW3 repository.
