# Migration checklist (summary)

Phased migration is tracked for the **whole** TeknoMW3 repository, not only Rust. This file is a short index; the canonical **status and roadmap** for the Rust release is **[PROGRESS.md](PROGRESS.md)**.

## Phase status

| Phase | Status | Rust deliverable |
|-------|--------|------------------|
| 0 | Done | Inventory → [CONTRACTS.md](CONTRACTS.md) |
| 1 | Done | `mw3-protocol` |
| 2 | Done | `mw3-master` |
| 3 | Done | UDP in `mw3-protocol` + `mw3-query-cli` |
| 4 | Done | `mw3-dedi-launch` |
| 5 | In progress | `mw3-loader` (core + CLI); GUI optional / later |
| 6+ | Pending | DLL / hook strategy (outside current workspace scope) |

## Layout

```
rust/
  Cargo.toml
  crates/
    mw3-protocol/
    mw3-master/
    mw3-query-cli/
    mw3-dedi-launch/
    mw3-loader/
```

For a longer narrative plan (including diagrams), maintainers may keep a separate document in the repository root; it is not required reading for building or releasing this workspace.
