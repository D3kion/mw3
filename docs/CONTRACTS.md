# TeknoMW3 contracts and inventory (phase 0)

This document reflects the C++ sources in the repository. It is the reference for the Rust workspace (`mw3-protocol`, `mw3-master`, `mw3-query-cli`, `mw3-dedi-launch`, `mw3-loader`).

---

## 1. Master server (TCP)

**C++ sources:** `mw3_master_server/include/MasterServerProtocol.hpp`, `master_server.cpp`, `ServerList.cpp`.

### Transport

| Parameter | Value |
|-----------|--------|
| Protocol | TCP |
| Default port | **27017** (`MW3_MS_LISTEN_PORT`) |
| Model | Listening socket; each `accept` spawns a thread, **up to 512 bytes** read with one `recv`, then the socket is closed |

### Timing

| Constant | Value | Meaning |
|----------|-------|---------|
| `MW3_MS_CLEANUP_RATE` | **30000** (ms) | Garbage-collection interval; entries older than this since last heartbeat are removed |

### Magic values (DWORD, little-endian on the wire)

C uses multichar literals; on typical GCC/VC LE builds the numeric values match the wire:

| Role | Macro | `uint32` (hex) | First 4 bytes on wire (LE: low byte first) |
|------|-------|----------------|---------------------------------------------|
| Server registration | `MW3_MS_SERVER_MAGIC4CC` | **0x424F4F42** | `42 4F 4F 42` (reads as “BOOB”) |
| Client list request | `MW3_MS_CLIENT_MAGIC4CC` | **0x434F4B45** | **`45 4B 4F 43`** (LE encoding of that `u32`; not ASCII “COKE” in order) |

### Structures (`#pragma pack(1)`)

Win32 types: `DWORD` = 32-bit, `USHORT` = 16-bit.

| Structure | Size (bytes) | Fields in order |
|-----------|--------------|-----------------|
| `MW3_MS_SERVER_ENTRY` | **6** | `IpAddress` (4), `QPort` (2) |
| `MW3_MS_SERVER_REQUEST` | **10** | `Magic4CC` (4), `Version` (4), `QPort` (2) |
| `MW3_MS_CLIENT_REQUEST` | **8** | `Magic4CC` (4), `Version` (4) |
| `MW3_MS_CLIENT_RESPONSE` (header) | **4** | `NumberOfEntries` (4); then **exactly** `NumberOfEntries * 6` bytes of `Entries` |

**Minimum inbound length** (full packet, not “after magic” only):

- Server: **10** bytes (`MW3_MS_SERVER_REQUEST`).
- Client: **8** bytes (`MW3_MS_CLIENT_REQUEST`).

### Semantics

- **Version** (`DWORD`): separates server lists; one `ServerList` instance per version value.
- **Registered server IP** comes from the **`accept` peer** (`sockaddr_in::sin_addr`), **not** from the packet body; `handleEntry` uses `ntohl(peer.sin_addr.s_addr)` — **host byte order** `DWORD`.
- **QPort** for registration is taken from the request payload (`pRequest->QPort`).
- Client response layout: `NumberOfEntries` + array; original allocates with `malloc`, size `4 + 6 * N`.

### Client DLL linkage

`steam_api_emu/game_server_items.h` includes the same `MasterServerProtocol.hpp`. **`MW3_MASTER_SERVER_UPDATE_MSG`** (magic `MW3_SERVER_4BB` = `'BOOB'`, fields `LongVersion`, `QPort`) matches the **10-byte** TCP server registration layout (field name `Magic4BB` vs `Magic4CC` only).

Other magics in `game_server_items.h` (non–master-TCP traffic):

| Macro | Hex (multichar LE) | Role in code |
|-------|-------------------|--------------|
| `MW3_SERVER_4CC` `'POOL'` | **0x4C4F4F50** | UDP server query / info |
| `MW3_SERVER_4CC_old` `'POOP'` | **0x504F4F50** | Legacy variant |
| `MW3_SERVER_4BB` `'BOOB'` | **0x424F4F42** | Same as master registration magic |

### UDP query / `MW3_SERVER_INFO` (lab / `mw3_nettest`)

**Sources:** `steam_api_emu/game_server_items.h`, `mw3_nettest/mw3_nettest.cpp`.

| Structure | Size | Layout (`#pragma pack(1)`) |
|-----------|------|----------------------------|
| `MW3_SERVER_QUERY` | **8** | `Magic4CC` (u32 LE), `TimeStamp` (u32 LE) — magic usually POOL |
| `MW3_SERVER_INFO` fixed prefix | **81** | Fields through `RawDataSize` (u16); then `RawData[]` up to **2048** bytes in the C struct (datagram may be shorter) |

`mw3_nettest` binds UDP (sample port **27057**), sends a POOL query to a hard-coded host, then `recvfrom` and reads `ServerInfos` via `RawData[ServerInfos_ptr]` (NUL-terminated C string).

**CLI:** `mw3-query-cli` (`rust/crates/mw3-query-cli`) exposes `--bind`, `--target`, `--magic pool|poop`, `--timestamp`, `--timeout-ms`, `--recv-buf`.

---

## 2. `teknogods.ini`

**Path:** `.\teknogods.ini` (relative to the game process working directory).

**Format:** Classic INI via `CIniReader` / `CIniWriter` (`steam_api_emu/util_ini.cpp`): `[Section]`, `Key=value`.

### `[Settings]`

| Key | Type | Read | Written | Default / notes |
|-----|------|------|---------|-----------------|
| `Name` | string | yes | yes | Empty → Steam registry / Windows username / `^3TeknoSlave` |
| `ID` | string (hex) | yes | yes | Empty → HWID; invalid hex → `[Errors]` and reset |
| `FOV` | int | yes | no (in this block) | `0` |
| `tkdev` | bool | yes | no | `false` |

### `[Network]`

| Key | Type | Read | Written | Default / notes |
|-----|------|------|---------|-----------------|
| `NetworkInterface` | int | yes | yes (if value was 255) | `255` = any interface |
| `NetworkInterfaceList` | string | no | yes | Informational list, rewritten on startup |
| `GlobalBans` | bool | yes | no | `false` |
| `OnlineMode` | bool | yes | no | `true`; if `false`, master from ini is not used for online path |
| `ExternalIP` | string | yes | no | Empty → special IP branch (see `steam_api_emu.cpp`) |
| `MasterServer` | string | yes | no | **`teknogods.com:27017`** — `host:port` for `parseIpPort` |

### `[Errors]`

| Key | When written |
|-----|----------------|
| `LastError` | Invalid hex in `Settings\ID` |

### Profile dumper

`ProfileDumper_thread` may write **`Settings\ID`** as `%08X` (profile SteamID) after user confirms in a message box.

---

## 3. `TeknoMW3.dll` exports (replaces `steam_api.dll`)

Launcher module name: **`TeknoMW3.dll`** (`mw3_dedi_launcher/codmw3_dedi_launcher.cpp`). The game expects **Steam API** export names.

### Steam API (`steam_api_emu/steam_api_emu_exports.cpp`)

Exported (`__cdecl`, except global pointers), including:

`GetHSteamPipe`, `GetHSteamUser`, `SteamAPI_GetHSteamPipe`, `SteamAPI_GetHSteamUser`, `SteamAPI_GetSteamInstallPath`, `SteamAPI_Init`, `SteamAPI_InitSafe`, `SteamAPI_RegisterCallResult`, `SteamAPI_RegisterCallback`, `SteamAPI_RunCallbacks`, `SteamAPI_SetMiniDumpComment`, `SteamAPI_SetTryCatchCallbacks`, `SteamAPI_Shutdown`, `SteamAPI_UnregisterCallResult`, `SteamAPI_UnregisterCallback`, `SteamAPI_WriteMiniDump`, `SteamApps`, `SteamClient`, `SteamContentServer`, `SteamContentServerUtils`, `SteamContentServer_Init`, `SteamContentServer_RunCallbacks`, `SteamContentServer_Shutdown`, `SteamGameServerNetworking`, `SteamGameServerStats`, `SteamFriends`, `SteamGameServer`, `SteamGameServerUtils`, `SteamGameServer_BSecure`, `SteamGameServer_GetHSteamPipe`, `SteamGameServer_GetHSteamUser`, `SteamGameServer_GetIPCCallCount`, `SteamGameServer_GetSteamID`, `SteamGameServer_Init`, `SteamGameServer_InitSafe`, `SteamGameServer_RunCallbacks`, `SteamGameServer_Shutdown`, `SteamMasterServerUpdater`, `SteamMatchmaking`, `SteamMatchmakingServers`, `SteamNetworking`, `SteamRemoteStorage`, `SteamUser`, `SteamUserStats`, `SteamUtils`, `Steam_GetHSteamUserCurrent`, `Steam_RegisterInterfaceFuncs`, `Steam_RunCallbacks`, `SteamAPI_RestartApp`, `SteamAPI_RestartAppIfNecessary`, `SteamAPI_IsSteamRunning`, plus **`g_pSteamClientGameServer`**, **`V_2_7_0_5`**.

Full list: search `__declspec(dllexport)` in that file.

### Tekno internals (not `dllexport`)

Declared in `steam_api_emu/steam_api_emu.h` for in-project linking:

- `SteamAPI_Main`
- `TeknoGodzMW2_SteamSetup` — `int __cdecl`
- `TeknoGodzMW2_SetNickname` — `void __cdecl`
- `TeknoGodzMW2_SetPendingConnection` — `void __cdecl` (`ipaddr`, `port`)

They are **not** marked `dllexport` in the sources reviewed; external loaders rely on **Steam** symbol names.

---

## 4. Process and file names

| Artifact | Value in code |
|----------|----------------|
| Dedicated executable | `iw5mp_server.exe` (`codmw3_dedi_launcher.cpp`, `g_DediExeName`) |
| Replacement DLL | `TeknoMW3.dll` (`g_DllName`) |
| PE marker | String `steam_api.dll` (search in `.rdata`) |

---

## 5. VMProtect and build modes

- Header: `steam_api_emu/VMProtectSDK.h` — **imports** VMProtect from an external DLL.
- **Release** `SteamAPI_Main` (`steam_api_emu.cpp`): without `DEBUGGING_ENABLED`, `VMProtectIsValidImageCRC()` and `VMProtectIsDebuggerPresent(false)` may terminate the process.
- Macros `VU` / `VM` tie into VMProtect obfuscation markers.

**Open-source Rust builds:** do not link VMProtect; replace or drop anti-debug / image CRC behavior explicitly.

---

## 6. Other INI files

| File | Role |
|------|------|
| `.\main\permanent_ex.ban` | Bans (`game_admin_base.cpp`), `[Bans]` section |

---

## Rust implementation

| Component | Path | Role |
|-----------|------|------|
| Protocol | `crates/mw3-protocol` | Master TCP + UDP query / `MW3_SERVER_INFO` header |
| Master server | `crates/mw3-master` | TCP listener (`mw3-master` binary), state + stale purge |
| Query CLI | `crates/mw3-query-cli` | UDP `MW3_SERVER_QUERY` → parse response (`mw3-query-cli` binary) |
| Dedi launcher | `crates/mw3-dedi-launch` | PE tail scan for `steam_api.dll` after `pc\iw5mp_server` in `.rdata*`; Windows: launch `iw5mp_server.exe` suspended, patch string, resume (`mw3-dedi-launch` binary). |
| Loader core | `crates/mw3-loader` | `teknogods.ini` (`[Section]`, `key=value`, pre-section keys under `ROOT`, CRLF on save); CLI argv helpers (`args-dedicated`, `args-client-lan`, `args-client-connect`, `args-sp-coop`, `args-sp`); optional `update-check` GET `http://teknogods.com:8080/updatecheck/?project=mw3` (body `version;url`). Graphical launcher not included. |

**Dedi launcher notes:** The Rust tool scans every `.rdata` / `.rdata*` section (some linkers split read-only data) until `pc\iw5mp_server` and `steam_api.dll` are found in the tail window. Mutex name: `TeknoMW3` + `(pid ^ 0x57)` as 8 hex digits. Replacement DLL default: `TeknoMW3.dll` (13-byte slot including padding), overridable with `--dll`.

```bash
cd rust && cargo test -p mw3-protocol -p mw3-master -p mw3-query-cli -p mw3-dedi-launch -p mw3-loader
cargo run -p mw3-master -- --bind 0.0.0.0:27017
cargo run -p mw3-query-cli -- --target 192.0.2.1:27015 --bind 0.0.0.0:0
cargo run -p mw3-dedi-launch -- --print-va /path/to/iw5mp_server.exe
cargo run -p mw3-loader -- args-dedicated 27015 --usekeys
```

---

## Related documentation

- Repository root `docs/PROJECT_OVERVIEW.md` — architecture (including Mermaid diagrams), Russian.
- Repository root `docs/RUST_MIGRATION_PLAN.md` — migration phases, Russian.
