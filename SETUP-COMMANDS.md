# Setup commands — run these yourself

Claude's sandbox has no access to your Windows desktop, your GitHub
account, or the `skiddph` SSH host, so this part has to be run by you.
Everything below is copy-paste.

## 1. Create the GitHub repo and clone it locally

```bash
gh repo create eru123/workgrid-tunnel --public --clone
cd workgrid-tunnel
git remote -v   # confirm origin points at eru123/workgrid-tunnel
```

If you'd rather init an existing empty repo instead of letting `gh` clone
it fresh:

```bash
mkdir ~/workgrid-tunnel && cd ~/workgrid-tunnel
git init
git remote add origin https://github.com/eru123/workgrid-tunnel.git
```

## 2. Drop in the handoff files

Copy the four files from this handoff package into place:

```bash
mkdir -p docs/plan docs/developers
cp /path/to/handoff/specs.md   docs/plan/specs.md
cp /path/to/handoff/design.md  docs/plan/design.md
cp /path/to/handoff/tasks.md   docs/plan/tasks.md
```

(`agents-md-additions.md` isn't copied in as-is — its instructions get
applied to `AGENTS.md` in step 3 below, ideally by Hermes itself per the
instructions inside it.)

## 3. Apply the RepoPact contract

```bash
npx repopact init
```

Then either apply `agents-md-additions.md`'s edits to `AGENTS.md` yourself,
or leave that as Hermes's first task — the kickoff prompt below tells it
to do this before anything else.

## 4. Cargo workspace skeleton (optional — Hermes can also do this as its first task)

```bash
cargo new --lib crates/workgrid-protocol
cargo new --bin crates/workgrid-relay
cargo new --bin crates/workgrid-daemon
cargo new --bin crates/workgrid-cli
```

Add a root `Cargo.toml` with a `[workspace]` section listing all four
members if `cargo new` didn't create one automatically.

## 5. Commit and push

```bash
git add -A
git commit -m "Scaffold WorkGridTunnel project with RepoPact contract"
git push -u origin main
```

## 6. Notes on skiddph

`skiddph` (ubuntu26.04) is your real target server for the end-to-end
verification task at the bottom of `tasks.md` — Hermes will need working
SSH access to it (whatever's already in your Windows SSH config, or an
equivalent set of credentials it can use) to run the actual bootstrap flow
against a real machine. Nothing in the repo scaffold above touches
`skiddph` directly; that only happens once Hermes reaches the bootstrap
implementation and verification tasks.
