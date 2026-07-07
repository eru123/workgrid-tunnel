# Instructions for Hermes: post-`repopact init` edits

After running `npx repopact init` in the repo root, it will scaffold the
base `AGENTS.md` contract plus `docs/plan/` and `docs/developers/`. Do the
following before starting any implementation work:

1. Copy the contents of `specs.md`, `design.md`, and `tasks.md` from this
   handoff package into `docs/plan/specs.md`, `docs/plan/design.md`, and
   `docs/plan/tasks.md` respectively, replacing the placeholder content
   `repopact init` generated.

2. In the generated `AGENTS.md`, find the `## Project Map` section and
   replace its generic placeholder bullets with:

   ```
   - `crates/workgrid-protocol/`: shared framed RPC + wire types.
   - `crates/workgrid-relay/`: broker binary — pairing + byte forwarding only.
   - `crates/workgrid-daemon/`: remote-side binary, embeds a `russh` server
     bound to the relay WebSocket stream instead of a TCP socket.
   - `crates/workgrid-cli/`: client binary — bootstrap, connect, exec,
     session management, interactive TUI.
   - `docs/plan/specs.md`, `docs/plan/design.md`, `docs/plan/tasks.md`:
     already populated — treat as authoritative, do not regenerate from
     scratch.
   ```

3. Do not otherwise deviate from the rest of the generated `AGENTS.md`
   contract — the minimal-build-discipline, planning-file, testing,
   security, and commit rules apply exactly as scaffolded.

4. Follow `docs/plan/tasks.md` top-to-bottom, checking items off as
   completed, and log each meaningful decision or completed chunk of work
   in `docs/developers/task-logs.md` (latest entry first), per the
   contract's existing rules.

5. The task list intentionally ends with an end-to-end verification task
   against the real `skiddph` server. Do not mark the project complete
   until that task is checked off with real (not fabricated) verification
   output recorded in the task log.
