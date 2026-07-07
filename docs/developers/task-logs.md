# Task Logs

Latest entries first. Keep entries short and factual.

- Ran `npx repopact init` in `~/workgrid-tunnel` to generate the RepoPact contract files.
- Copied handoff docs into `docs/plan/specs.md`, `docs/plan/design.md`, and `docs/plan/tasks.md`.
- Updated `AGENTS.md` `## Project Map` to point at the actual WorkGridTunnel crate paths and plan files instead of the generic placeholder bullets.
- Created workspace `Cargo.toml` and `crates/workgrid-protocol` crate with `ControlMessage` enum for relay control-plane messages (`register`, `pair-request`, `pair-ack`).
- Committed scaffolding and pushed `main` to `git@github.com:eru123/workgrid-tunnel.git`.
- Added `workgrid-relay` crate with `Registry` and basic WebSocket accept loop.
- All crates compile with `cargo check --workspace` and `cargo build --workspace`.

