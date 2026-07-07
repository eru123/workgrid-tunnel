# Task Logs

Latest entries first. Keep entries short and factual.

- Ran `npx repopact init` in `~/workgrid-tunnel` to generate the RepoPact contract files.
- Copied handoff docs into `docs/plan/specs.md`, `docs/plan/design.md`, and `docs/plan/tasks.md`.
- Updated `AGENTS.md` `## Project Map` to point at the actual WorkGridTunnel crate paths and plan files instead of the generic placeholder bullets.
- Created workspace `Cargo.toml` and `crates/workgrid-protocol` crate with `ControlMessage` enum for relay control-plane messages (`register`, `pair-request`, `pair-ack`).
- Committed scaffolding and pushed `main` to `git@github.com:eru123/workgrid-tunnel.git`.
- Added `workgrid-relay` crate with `Registry` and basic WebSocket accept loop.
- All crates compile with `cargo check --workspace` and `cargo build --workspace`.
- Attempted to add relay bidirectional forwarding; the first cleanup pass hit a borrowed-move error in tokio-tungstenite stream handling (`ws.next().await` yielded `[Message]` in one branch but `Message` in the compiled path). The code path blocks until the stream semantics are made consistent.
- Hit a hard Windows environment blocker: `nohup` is unavailable in this session's PATH (`nohup: command not found`) before verifying the end-to-end relay test harness against localhost. The docs already warn against repeating this in full-disk msys git-bash, but it still blocks local verification here.
- Documented blocker; implementation paused pending a Windows-capable harness path.

