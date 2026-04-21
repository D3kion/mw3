# TeknoMW3 Rust workspace — status and roadmap

Last updated for workspace version **0.1.0** (see root `Cargo.toml`).

## Snapshot

| Area | Status | Notes |
|------|--------|--------|
| TCP master protocol | Done | `mw3-protocol` encode/decode, limits on list size |
| TCP master server | Done | `mw3-master` binary, integration smoke test |
| UDP server query | Done | `mw3-query-cli`, POOL/POOP |
| Dedicated launcher | Done | `mw3-dedi-launch`: PE scan; Windows suspend/patch/resume |
| Loader utilities | In progress | `mw3-loader` lib + CLI (ini, argv, update-check); **no GUI** in this crate |
| Game DLL / hooks | Not started | Out of scope for this workspace until a separate design |

## Crates (release-ready tooling)

1. **`mw3-protocol`** — Safe (`#![forbid(unsafe_code)]`). Single source of truth for on-wire layouts used by master and query tools.
2. **`mw3-master`** — Thread-per-connection TCP server; configurable bind address; tracing logs.
3. **`mw3-query-cli`** — CLI for lab and server admins; requires network reachability to the target UDP port.
4. **`mw3-dedi-launch`** — Uses `unsafe` only on Windows for Win32 APIs. Linux-friendly for `--print-va` PE inspection.
5. **`mw3-loader`** — Safe. Optional HTTP to TeknoGods update endpoint; can be used headless or embedded later in a GUI.

## Near-term plans

- **Windows validation**: run release binaries against a real game directory (master registration, dedi launch, query).
- **Loader**: optional GUI crate or FFI; document embedding for third-party frontends.
- **Versioning**: bump workspace `version` and tag releases when protocol or CLI compatibility changes.
- **Repository URL**: set `[workspace.package] repository` in `rust/Cargo.toml` to your fork when publishing.

## Quality gate (before tagging a release)

```bash
cd rust
cargo fmt --check
cargo clippy -- -D warnings
cargo test
cargo build --release
```

## Testing matrix (suggested)

| Target | `mw3-master` | `mw3-query-cli` | `mw3-dedi-launch` | `mw3-loader` |
|--------|----------------|-----------------|-------------------|--------------|
| Linux CI | yes | yes (if UDP peer available) | `--print-va` only | yes (skip `update-check` or allow network) |
| Windows | yes | yes | full launch | yes |

## Known limitations

- `mw3-dedi-launch` depends on the game PE layout (markers in `.rdata`). New game builds may require adjustment or clearer diagnostics.
- `mw3-loader` update-check calls third-party HTTP endpoints; failures should be treated as non-fatal (same as typical launcher behavior).
