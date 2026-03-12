
游戏要开发成中文的！！！
---

### File 2: `AGENTS.md` (Implementation Spec)

这份文档针对开发者，详细说明了“如何做 TUI”以及“如何为 Web 预留接口”。

```markdown
# AGENTS.md - Technical Implementation Steps

## 1. Core Architecture (The "Headless" Server)
The application must be architected as a **Local Server first**, with a **TUI Client attached**.
This ensures the Web UI can be added later without rewriting logical code.

*   **Language**: Rust (Edition 2021)
*   **Binary**: Single executable deployment.

### 1.1 Technology Stack
*   **Core Logic**: Pure Rust structs (`Farm`, `Crop`, `Inventory`).
*   **API Server**: `axum` (Lightweight, robust HTTP server).
*   **TUI Rendering**: `ratatui` + `crossterm`.
*   **Async Runtime**: `tokio` (Multi-threaded).
*   **Discovery**: `mdns-sd` (Multicast DNS) or simple UDP Broadcast for LAN discovery.
*   **Storage**: `serde_json` saving to `~/.lanvalley/save.json` (Atomic writes).

---

## 2. API Design (Web-Ready)
Even the TUI interaction should ideally tap into internal state via clear boundaries, but the HTTP API is mandatory for the future Web/Lan features.

**Endpoints to implement:**
*   `GET /api/v1/status` -> Returns full JSON of grid, inventory, and wallet.
*   `POST /api/v1/plant` -> Body: `{ "x": 1, "y": 2, "seed_id": "parsnip" }`
*   `POST /api/v1/harvest` -> Body: `{ "x": 1, "y": 2 }`
*   `POST /api/v1/trade/send` -> Body: `{ "target_ip": "...", "item": "potato", "amount": 10 }`
*   `POST /api/v1/trade/receive` -> (Webhook for other peers to call)

---

## 3. Development Phases

### Phase 1: The Engine & TUI (Rust + Ratatui)
1.  **State Logic**: Create `struct Grid` and `enum CropType`. Implement growth tick mechanism (independent of UI).
2.  **TUI Render**: Build the dashboard layout.
    *   *Requirement*: Detect terminal capabilities. If modern terminal, use Emoji (🌽, 🥔). If legacy (cmd.exe), fallback to Colored ASCII (`P` for Parsnip, `C` for Corn).
3.  **Input Loop**: Handle arrow keys for cursor movement and simple hotkeys (`h`arvest, `w`ater).

### Phase 2: The Background Server (Axum)
1.  Spawn `axum::serve` in a separate `tokio::task`.
2.  Wrap GameState in `Arc<RwLock<GameState>>` so both TUI (Main Thread) and AXUM (Async Thread) can access it safely.
3.  **Persistence**: Auto-save every 60 seconds and on `SIGINT` (Ctrl+C).

### Phase 3: LAN Networking (P2P)
1.  **Discovery**: On start, broadcast `UDP` packet: `HERE_IS_FARM:{IP}:{BIOME_TYPE}`.
2.  **Listing**: Maintain a concurrent `HashMap` of active neighbors in the last 5 minutes.
3.  **Trading**: When Player A sends a truck, the app performs a `POST` request to Player B's IP: `http://<B-IP>:port/api/v1/trade/receive`.

---

## 4. CI/CD & Build (GitHub Actions)
Fully automated build pipeline is required.

**File**: `.github/workflows/build.yml`
*   **Triggers**: Push to `main`, Tags `v*`.
*   **Jobs**:
    *   **Build Linux**: `ubuntu-latest` -> `cargo build --release`
    *   **Build Windows**: `windows-latest` -> `cargo build --release` (Generates `.exe`)
    *   **Build MacOS**: `macos-latest` -> `cargo build --release` (Universal binary preferred or x86_64/arm64 split).
*   **Release**: Use `softprops/action-gh-release` to upload binaries automatically.

---

## 5. Specific Constraints
*   **No Database**: Use a local JSON file.
*   **No External Assets**: All "graphics" must be code-generated (ANSI/Unicode chars).
*   **Cross-Platform Paths**: Use `directories` crate to find valid config paths on Win/Mac/Linux.
```*
