# WorkGridTunnel — Specifications

## Problem statement

Jericho needs a way for terminal sessions (opened by himself or by an AI agent
acting on his behalf) to reach SSH servers that may be behind NAT/firewalls
with no inbound port open, without maintaining a VPN. A single logical
connection to a server must support multiple concurrent terminal
sessions/exec calls (multiplexing), and first-time setup on a new server
should not require manually installing anything by hand — the client should
install itself, mirroring how VS Code's Remote Tunnels bootstraps its
remote server component over a one-time plain SSH connection.

## Actors

- **Owner (Jericho)**: runs the relay, owns all remote servers, revokes keys.
- **CLI operator**: a human or an AI agent invoking the `workgrid` binary as
  a subprocess. Agents primarily use the non-interactive `exec` mode with
  structured (JSON) output; humans primarily use the interactive TUI.
- **Relay**: a single self-hosted broker process (one per Jericho's
  infrastructure, not multi-tenant, not public).
- **Remote server**: a Linux host Jericho controls, reachable via normal SSH
  today, which will run the `workgrid-daemon` after bootstrap.

## Functional requirements

1. **Bootstrap**: given normal SSH connection parameters (host, user, key or
   password), the CLI connects once over plain SSH, detects OS/arch, pushes
   the matching static `workgrid-daemon` binary, provisions a per-daemon
   Ed25519 identity, registers that identity with the relay, starts the
   daemon (systemd --user unit preferred, nohup fallback), confirms it comes
   online via the relay, then never uses plain SSH again for that server
   unless the user explicitly re-bootstraps (e.g. binary upgrade).
2. **Outbound-only remote connectivity**: the daemon never listens on a
   public/inbound port. It dials out to the relay and holds a persistent,
   auto-reconnecting (exponential backoff) authenticated WebSocket.
3. **Relay**: pairs an inbound client request with the correct daemon
   connection by tunnel/server ID and forwards bytes bidirectionally. The
   relay never terminates SSH or decrypts session content — it is a dumb,
   auditable broker.
4. **Session multiplexing**: one relay-brokered connection to a server
   supports many independent concurrent terminal sessions / exec calls /
   (later) port-forwards, using SSH's native channel model inside the
   daemon (embedded `russh` server bound to the WebSocket stream instead of
   a TCP socket).
5. **CLI surface**:
   - `workgrid bootstrap <alias> --host <h> --user <u> [-i keyfile]`
   - `workgrid connect <alias>`
   - `workgrid exec <alias> "<cmd>" [--json]` — non-interactive, structured
     stdout/stderr/exit-code for agent consumption.
   - `workgrid session new|list|attach <id>|kill <id>`
   - `workgrid forget <alias>` — revokes the daemon's key at the relay and
     optionally uninstalls the daemon over one more plain SSH connection.
6. **Interactive terminal app**: a TUI (tabs/panes) for concurrently
   attached sessions, using the same multiplexed channels as the
   non-interactive path.
7. **Auth**:
   - Daemon ↔ relay: per-server Ed25519 keypair, checked against a registry
     the relay holds.
   - Client ↔ daemon: standard SSH public-key auth, one keypair per human or
     per agent identity, independently revocable.
8. **Audit logging**: every channel open and exec call logged in the daemon
   with the client key that opened it.

## Non-functional requirements

- No inbound ports ever required on a bootstrapped server.
- Daemon and relay ship as static (musl) Rust binaries — no runtime deps on
  the target server, no internet access required on the target server
  beyond reaching the relay.
- Auto-reconnect on network loss; sessions marked stale/reattachable rather
  than silently dropped where possible.
- `exec --json` output must be stable and parseable by an agent without any
  ANSI/TTY artifacts.
- Per-key revocation must not require restarting the daemon or relay.

## Explicit non-goals (v1)

- Not a multi-tenant/public SaaS tunneling service.
- No GUI beyond the terminal TUI.
- No Windows daemon target — remote servers targeted are Linux
  (client CLI itself may run on Windows/macOS/Linux).
- Full arbitrary port-forwarding is a later phase, not v1.
- OAuth/device-code flows are explicitly not needed (single-owner infra).
