# Task Logs

Latest entries first. Keep entries short and factual.

- Updated `crates/workgrid-relay/src/server.rs` to verify registering public keys via `Registry::check_signing` before accepting them, and to send a `PairAck` control message on successful pairing. Added `Registry::check_signing` in `crates/workgrid-relay/src/registry.rs` to ensure submitted public keys decode to Ed25519 public-key length. Verified with `cargo check --workspace`; task 3 checkbox updated in `docs/plan/tasks.md`.

- Resolved the prior `ws.next().await` borrow-check issue in `crates/workgrid-relay/src/server.rs` by reading the peer control message directly from the same owned `WebSocketStream` instead of trying to split a borrowed stream.
- Removed dead `PairGuard`/`PendingEntry` scaffolding and the `run_checks` stub that was introduced in the blocked implementation attempt. Added a small `verify_pair` helper using `Registry::verify_signature`.
- Confirmed `cargo check --workspace` compiles cleanly after the change and pushed to `origin/main` as `4729504`.
- Doc blocker remains for full task 3 verification: on this Windows host the relay end-to-end harness still lacks a viable local WebSocket test path because the full-disk MSYS `git-bash` environment here does not provide `nohup`, and task 37 explicitly requires verification against the real `skiddph` server; without SSH credentials I cannot execute that harness here. No fabricated verification output was added.

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
