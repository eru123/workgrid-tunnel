# WorkGridTunnel — Design

## Workspace layout (Cargo workspace, single language: Rust)

```
workgrid-tunnel/
  crates/
    workgrid-protocol/   # shared framed RPC + wire types, used by all 3 binaries
    workgrid-relay/      # broker binary
    workgrid-daemon/     # remote-side binary (embeds russh server)
    workgrid-cli/        # client binary + TUI
  docs/
    plan/
      specs.md
      design.md
      tasks.md
    developers/
      task-logs.md
  AGENTS.md
  README.md
```

One language, one toolchain, static musl builds for daemon + cli so they can
be dropped onto any target server/desktop with no runtime deps.

## Components

### 1. workgrid-relay

- Single always-on process, one instance, self-hosted (Jericho's existing
  VPS).
- Holds a registry: `server_id -> public key`, populated by the CLI during
  bootstrap (client calls relay's registration endpoint directly over
  HTTPS/TLS — the remote server does not need to reach the relay yet at
  registration time, only at daemon-start time).
- Accepts inbound WebSocket connections from both daemons and clients.
  Pairs them by `server_id` + validates each side's signature against the
  registry before forwarding any bytes.
- Purely forwards opaque encrypted frames after pairing — never inspects or
  terminates the SSH/RPC content inside. This keeps the relay's trust
  surface minimal: a compromised relay can disrupt connections but not read
  session content.
- No session state beyond "who's paired with whom right now." Restart-safe.

### 2. workgrid-daemon

- Runs on each remote server. On start: loads its Ed25519 identity + relay
  endpoint from a local config file (written during bootstrap), dials the
  relay over WSS, and holds that connection open with reconnect/backoff.
- Embeds an SSH server (`russh` crate) bound to the WebSocket byte stream
  instead of a raw TCP socket. This gives channel multiplexing for free:
  one relay-brokered connection maps to N independent SSH channels
  (PTY sessions, exec, later port-forward channels).
- Client authentication at this layer is standard SSH public-key auth
  against an allowlist file (`authorized_agents` — same shape as
  `authorized_keys`), so revoking one agent/human's access means removing
  one line, no daemon restart required (reload on SIGHUP or file-watch).
- Logs every channel-open and exec invocation with the authenticated
  client key fingerprint to a local audit log file.
- Runs unprivileged, as whatever user the bootstrap SSH login used —
  deliberately the same blast radius as that user's normal shell access,
  no elevation.

### 3. workgrid-cli

- **Bootstrap mode** (`workgrid bootstrap`): the only place raw SSH
  credentials are used, and only transiently (not persisted after success).
  Steps:
  1. Plain SSH connect with given host/user/key or password.
  2. `uname -sm` to pick the right static daemon binary from a local
     bundled set (no internet access required on the remote box).
  3. SFTP-push the binary to a fixed path.
  4. Generate Ed25519 keypair locally, write private key + relay endpoint
     into the daemon's config path on the remote box over the same SSH
     session.
  5. Register the new public key with the relay directly from the CLI
     (client already has network access to the relay).
  6. Start the daemon: `systemctl --user enable --now workgrid-daemon` if a
     user systemd session is available; otherwise
     `setsid nohup workgrid-daemon &` plus a small watchdog cron/loop
     fallback the bootstrap script installs.
  7. Poll relay for "daemon online," confirm, close the plain SSH session.
  8. Idempotent: re-running against an already-bootstrapped server detects
     matching version and no-ops; detects older version and offers an
     in-place upgrade (push new binary, restart, keep same identity).
- **Connect / exec / session commands**: talk to the relay only, never
  plain SSH again. `exec --json` returns
  `{ "stdout": ..., "stderr": ..., "exit_code": ... }` with no TTY/ANSI
  artifacts, so an agent invoking `workgrid` as a subprocess can parse it
  directly.
- **Interactive TUI** (`ratatui`): tabs/panes over the same multiplexed
  channels; PTY resize forwarded as SSH window-change requests.
- **`forget <alias>`**: revokes the key at the relay, optionally does one
  more plain-SSH pass to stop/remove the daemon.

## Auth model summary

| Link | Mechanism |
|---|---|
| Bootstrap (one-time) | Plain SSH, user's existing credentials |
| Daemon ↔ Relay | Per-server Ed25519 keypair vs. relay registry |
| Client ↔ Daemon | SSH pubkey auth vs. daemon's `authorized_agents` allowlist |

No OAuth/device-code flow — this is single-owner infrastructure, not a
multi-tenant public service, so that whole layer of VS Code's design is
intentionally skipped.

## Wire protocol

WebSocket (TLS) carries the byte stream between client/daemon and the
relay. Inside that stream, `russh` speaks real SSH — channel multiplexing,
PTY requests, exec requests, and (later) direct-tcpip channels for
port-forwarding all come from SSH's existing channel model rather than a
custom RPC format. `workgrid-protocol` only defines the relay's own
pairing/control-plane messages (register, pair-request, pair-ack), not the
session content itself.

## Phased build order

1. Relay — pairing + byte forwarding only, single daemon/client pair.
2. Daemon — `russh`-embedded server over WS, pubkey auth, multi-channel.
3. Bootstrap flow — plain SSH install, binary push, key provisioning,
   handoff to relay-based connection.
4. CLI core — `connect` + `exec` via relay, JSON output mode.
5. Terminal app — interactive multi-session TUI.
6. Hardening — reconnect/resilience, audit logs, versioned upgrade,
   `forget`/unregister, packaging (static musl builds, install script).

## Explicit simplifications (mark in code with `minimal:`)

- No cross-relay federation — one relay instance only.
- No Windows daemon target.
- Port-forwarding channels deferred past v1; only PTY/exec channels in v1.
