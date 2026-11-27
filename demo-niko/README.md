# Symposium 0.7.0 walkthrough

A self-guided pass over 0.7.0, one topic per journey on the project board. Every command was run on macOS on 2026-11-23 against released 0.7.0. The outputs are verbatim, trimmed to the lines that matter. The CLI, hook payloads and `codex debug prompt-input` were checked; the agent TUI steps were not (the sandbox had no logins). About 30 minutes, plus 15 for the bonus topics. Topic 6 comes after the bonus topics because it installs a plugin for every workspace.

In the outputs, `<DEMO>` is `~/symposium-demo`, and `~` in symposium's output is the sandbox home `<DEMO>/home`.

## 0. Setup

Run this once in every terminal you use for the demo:

```bash
export DEMO=~/symposium-demo REAL_HOME=$HOME
export RUSTUP_HOME=${RUSTUP_HOME:-$HOME/.rustup} CARGO_HOME=${CARGO_HOME:-$HOME/.cargo}
export HOME=$DEMO/home PATH=$DEMO/bin:$PATH
mkdir -p $HOME
```

Why `HOME` has to move:
- The agent relocation variables (`CLAUDE_CONFIG_DIR`, `CODEX_HOME`, `SYMPOSIUM_HOME`) are not enough. The Antigravity and Kiro hook files always live under `~`.
- `init` removes symposium's hooks from every agent you don't select, so with the real `HOME` it would edit those files.
- `RUSTUP_HOME` and `CARGO_HOME` keep cargo working.
- If `claude` or `codex` come from asdf, mise or nvm, export `ASDF_DATA_DIR`, `MISE_DATA_DIR` or `NVM_DIR` at the real path before the `HOME` line.

Install 0.7.0 into the sandbox. Without cargo-binstall, use `cargo install symposium --version 0.7.0 --locked --root "$DEMO"`.

```bash
cargo binstall symposium --version 0.7.0 --root "$DEMO" -y
```

`$DEMO/bin` now holds `symposium`, `cargo-agents` and `symposium-pm-cargo`. Log the agents in inside the sandbox:
- Claude: run `claude auth login`. Alternatively, run `claude setup-token` in a normal shell and export `CLAUDE_CODE_OAUTH_TOKEN` here.
- Codex: `mkdir -p ~/.codex && cp "$REAL_HOME/.codex/auth.json" ~/.codex/`.

### Fixtures

The fixtures live in `demo-niko/fixtures/` on this branch. Clone the branch into the sandbox and copy them into `$DEMO/work`. The clone also serves as the symposium checkout for B3.

```bash
git clone -q -b demo-niko https://github.com/fnzko/symposium "$DEMO/src"
mkdir -p "$DEMO/work" && cp -R "$DEMO/src/demo-niko/fixtures/." "$DEMO/work/"
```

| Fixture | |
|---|---|
| `demo-app` | Bin crate depending on `anyhow` and on the path dependency `acme-retry`. Its `Symposium.toml` declares a team hook and a team subcommand. |
| `acme-retry`, `acme-metrics` | Path dependencies that ship a `skills/` folder and no manifest. |
| `plugins/rust-review` | Agent Plugins package. |
| `plugins/claude-changelog` | Claude Code plugin with one command and one `Stop` hook. |

## 1. `init` (Set up Symposium, AT-SETUP-01, [#424](https://github.com/symposium-dev/symposium/issues/424))

```bash
cd "$DEMO/work/demo-app"
symposium init
```

Take the defaults. The agents detected from their config folders come preselected:

```text
ℹ️  Detected: Claude Code, Codex CLI
...
Install hooks and agent configuration: Globally (recommended)
Automatic updates: Auto-update (recommended)
...
✅ ~/.symposium/config.toml: wrote user config (agents: Claude Code, Codex CLI)
✅ ~/.claude/settings.json: added hooks (PreToolUse, PostToolUse, UserPromptSubmit, SessionStart, Stop)
✅ ~/.codex/hooks.json: added hooks (PreToolUse, PostToolUse, UserPromptSubmit, SessionStart, Stop)
```

The hook commands call `symposium`, now the primary binary (`cargo agents` still works). `grep -m1 'symposium hook' ~/.claude/settings.json` prints `"command": "symposium hook claude pre-tool-use"`. The telemetry question is asked, but nothing is recorded yet.

## 2. Skills from dependencies (AT-DISCOVERY)

```bash
symposium status
```

```text
💤 assert-struct-guidance — registry `symposium-recommendations`
💤 codspeed — registry `symposium-recommendations`
💤 dial9 — registry `symposium-recommendations`
✅ error-battery-pack — registry `symposium-recommendations`
💤 rtk — registry `symposium-recommendations` (dormant: awaiting `symposium use`)
✅ symposium — registry `symposium-recommendations`
💤 toasty-guidance — registry `symposium-recommendations`
✅ demo-app — workspace member
❓ acme-retry 0.3.1 — found via dependency `acme-retry`, awaiting consent (`symposium use acme-retry`)
```

`error-battery-pack` is active through `anyhow`. The other sleeping entries are gated on dependencies this workspace lacks. `rtk` has no gate, so it is dormant until `use`. `acme-retry` is the dependency's own plugin, waiting for consent. The registry rows are whatever symposium-dev/recommendations holds on the day you run this.

Run `symposium sync` and answer "Ask me later" (the default):

```text
Dependency `acme-retry` provides 1 skill group. Enable it?: Ask me later
ℹ️  scanning 2 workspace dependencies
installed plugin symposium for claude -> <DEMO>/work/demo-app/.claude/skills/symposium
installed plugin error-battery-pack for claude -> <DEMO>/work/demo-app/.claude/skills/error-battery-pack
✅ installed skill find-crate-source for codex → <DEMO>/work/demo-app/.agents/skills/find-crate-source
✅ installed skill rust-best-practice for codex → <DEMO>/work/demo-app/.agents/skills/rust-best-practice
✅ installed skill application-errors for codex → <DEMO>/work/demo-app/.agents/skills/application-errors
✅ installed skill error-handling-basics for codex → <DEMO>/work/demo-app/.agents/skills/error-handling-basics
✅ installed skill library-errors for codex → <DEMO>/work/demo-app/.agents/skills/library-errors
```

Delivery differs per agent:
- Claude Code gets one project plugin per plugin, under `.claude/skills/<plugin>/`.
- Codex gets per-skill copies under `.agents/skills/`.
- Every installed directory carries a `.gitignore` with `*`.

To check in the agents:
- Claude only loads project plugins in a trusted folder started at the workspace root, so `/skills` lists them only in that case.
- Codex can be checked without a model call: `codex debug prompt-input hi | grep -o -- '- [a-z-]*: '`.

## 3. Consent for a dependency's plugin (AT-DISCOVERY, AT-APPROVAL)

### 3a. SessionStart context

A hook never prompts. The pending plugin reaches the agent as context instead:

```bash
printf '{"session_id":"demo","cwd":"%s","hook_event_name":"SessionStart","source":"startup"}' "$PWD" \
  | symposium hook claude session-start | python3 -m json.tool
```

```json
{
    "hookSpecificOutput": {
        "hookEventName": "SessionStart",
        "additionalContext": "This project has crate-aware tools available via `symposium`. Run `symposium --help` to list them before working with the Rust code. Only use tools under the 'Commands for agents' section unless the user explicitly asks you to run one from 'Commands for humans'.\n\nThese dependencies ship agent plugins that are not enabled yet: acme-retry. They stay off until the user consents. Tell the user to run `symposium sync` (which asks about each one) or `symposium use <name>`. Do not enable them yourself.",
        "reloadSkills": true
    }
}
```

### 3b. Enable at the prompt

Run `symposium sync` again and pick "Enable":

```text
Dependency `acme-retry` provides 1 skill group. Enable it?:
> Ask me later
  Enable
  No — don't ask again
...
ℹ️  enabled dependency plugins: acme-retry
installed plugin acme-retry for claude -> <DEMO>/work/demo-app/.claude/skills/acme-retry
✅ installed skill retry-policies for codex → <DEMO>/work/demo-app/.agents/skills/retry-policies
```

Each answer is written to the global `[plugins]` table in `~/.symposium/config.toml`:
- "Enable" goes to `auto-enable = ["acme-retry"]`, which `status` shows as `` `[plugins] auto-enable` ``.
- "No" goes to `disable`.
- "Ask me later" writes nothing.

In 0.7.0 no command undoes either answer; [#535](https://github.com/symposium-dev/symposium/issues/535) makes `use --remove` do it.

### 3c. A dependency added mid-session

In a Claude session, ask it to add `acme-metrics` as a path dependency. You can also replay what the agent would do:

```bash
cargo add acme-metrics --path ../acme-metrics
P=$(printf '{"session_id":"demo","cwd":"%s","hook_event_name":"PostToolUse","tool_name":"Bash","tool_input":{"command":"cargo add acme-metrics --path ../acme-metrics"},"tool_response":{"stdout":""}}' "$PWD")
echo "$P" | symposium hook claude post-tool-use | python3 -m json.tool
echo "$P" | symposium hook claude post-tool-use; echo
```

The `Cargo.lock` change triggers a re-sync. The first call names the new plugin, and the second prints `{}`, because each package and version is surfaced once:

```json
{
    "hookSpecificOutput": {
        "hookEventName": "PostToolUse",
        "additionalContext": "These dependencies ship agent plugins that are not enabled yet: acme-metrics. They stay off until the user consents. Tell the user to run `symposium sync` (which asks about each one) or `symposium use <name>`. Do not enable them yourself."
    }
}
```

### 3d. `use`

```bash
symposium use acme-metrics
symposium status | tail -2
```

```text
✅ enabled acme-metrics for this workspace
installed plugin acme-metrics for claude -> <DEMO>/work/demo-app/.claude/skills/acme-metrics
✅ installed skill metric-naming for codex → <DEMO>/work/demo-app/.agents/skills/metric-naming
✅ acme-metrics 1.2.0 — `[plugins] use`
✅ acme-retry 0.3.1 — `[plugins] auto-enable`
```

A running Claude session needs `/reload-plugins` to see it. A `use` run from another terminal does not reach that session at all.

## 4. Team hook and team subcommand (AT-TEAM, AT-HOOK)

`demo-app/Symposium.toml` holds `no-publish`, a symposium-format `PreToolUse` hook whose script exits 2 on `cargo publish` without `--dry-run`, and `demo-check`, an agent-audience subcommand. The workspace is a trust root, so nothing asks for consent.

```bash
printf '{"session_id":"demo","cwd":"%s","hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"cargo publish"}}' "$PWD" | symposium hook claude pre-tool-use; echo
printf '{"session_id":"demo","cwd":"%s","hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"cargo publish --dry-run"}}' "$PWD" | symposium hook claude pre-tool-use; echo
printf '{"session_id":"demo","turn_id":"t1","cwd":"%s","hook_event_name":"PreToolUse","model":"m","tool_name":"Bash","tool_use_id":"u1","tool_input":{"command":"cargo publish"}}' "$PWD" | symposium hook codex pre-tool-use; echo
printf '{"timestamp":1,"cwd":"%s","toolName":"bash","toolArgs":"{\\"command\\":\\"cargo publish\\"}"}' "$PWD" | symposium hook copilot pre-tool-use; echo
```

The four outputs are, in order: Claude, Claude with `--dry-run`, Codex, Copilot.

```text
{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"deny","permissionDecisionReason":"demo-app team rule: never run cargo publish from an agent session; use cargo publish --dry-run"}}
{}
{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"deny","permissionDecisionReason":"demo-app team rule: never run cargo publish from an agent session; use cargo publish --dry-run"}}
{"permissionDecision":"deny","permissionDecisionReason":"demo-app team rule: never run cargo publish from an agent session; use cargo publish --dry-run"}
```

The script receives one canonical event, and each agent gets the decision in its own wire format. On first start, Codex asks you to review the hooks.

```bash
symposium --help
symposium demo-check
```

```text
Commands for humans:
init         Set up user-wide configuration
...
use          Enable a plugin by name and sync it into the workspace

Commands for agents:
crate-info   Find crate sources
demo-check   Run the demo-app team checks (fmt, tests)
```

`demo-check` exists only inside the workspace; compare `symposium --help` run from `~`. Portable hook binaries (`hook::hook_main`) are still a draft, [#486](https://github.com/symposium-dev/symposium/issues/486).

## 5. Plugin authoring

```bash
cd "$DEMO/work/plugins"
symposium plugin validate rust-review
symposium plugin import claude-changelog --out imported
find imported -type f | sort
```

```text
  ⚠️  rust-review (agent plugin): plugin `rust-review` references no dependency; it stays dormant until enabled with `symposium use rust-review`
  ✅ review-unsafe (skill)
⚠️  name `Claude-Changelog` normalized to `claude-changelog`
Imported claude-changelog into imported/claude-changelog
  ⚠️  claude-changelog (plugin): plugin `claude-changelog` references no dependency; it stays dormant until enabled with `symposium use claude-changelog`
  ⚠️  claude-changelog (plugin): `Stop` hooks have only the `claude` format, so they fire on no other agent; add a `format = "symposium"` hook for the others
  ✅ changelog (skill)
imported/claude-changelog/Symposium.toml
imported/claude-changelog/plugin.json
imported/claude-changelog/skills/changelog/SKILL.md
```

In the import:
- The command became a skill.
- The hook kept Claude's format, so it runs only on Claude Code, with `CLAUDE_PLUGIN_ROOT` set.
- `validate` names what does not port.

```bash
cd "$DEMO/work/demo-app"
symposium plugin try ../plugins/rust-review
```

```text
trying rust-review in every workspace; edits reach your agents on the next sync
installed plugin rust-review for claude -> ~/.claude/skills/rust-review
installed plugin rust-review for codex -> ~/.codex/plugins/cache/symposium/rust-review
ℹ️  Claude Code has no listing command; start a new session here and run `/skills`
ℹ️  Codex CLI lists review-unsafe
```

`try` links the folder into the `user-plugins` registry. Because the plugin is dormant, `try` also records a global `use` entry. It then asks each agent what it actually loaded. Undo it with `symposium plugin try --remove rust-review`.

## Bonus

### B1. Out-of-process package manager (AT-ECOSYSTEM)

A `[[package-manager]]` named `cargo` replaces the in-process cargo PM. Append it to `~/.symposium/config.toml` only once: a second copy makes the config invalid, and symposium then falls back to its defaults. The wrapper below logs every spawn.

```toml
[[package-manager]]
name = "cargo"
command = "sh"
args = ["-c", "echo \"$(date +%T) spawned symposium-pm-cargo\" >> \"$HOME/pm-spawns.log\"; exec symposium-pm-cargo"]
```

```bash
cd "$DEMO/work/demo-app"
symposium status | tail -3
cat ~/pm-spawns.log
```

```text
✅ demo-app — workspace member
✅ acme-metrics 1.2.0 — `[plugins] use`
✅ acme-retry 0.3.1 — `[plugins] auto-enable`
11:07:29 spawned symposium-pm-cargo
```

The status output is the same as with the in-process PM, and the log shows one spawn per command. With `use` or `auto-enable` packages, a hook event spawns the PM too. There is no second real ecosystem yet.

### B2. Audits (AT-APPROVAL, AT-POLICY)

```bash
symposium plugin digest ../acme-retry
```

```text
sha256:e98011210ebf04e22fe2cc2b0a6ebbc31163c1c2337b0e796745496a6a697af6
```

Nothing beyond the digest can be shown. `symposium-dev/audits` is private and has no entries, so with `[defaults] symposium-audits = true` the fetch fails and no verdict appears anywhere. In 0.7.0, `plugin sync symposium-audits` reports that as `No git source found for provider` and exits 0; [#533](https://github.com/symposium-dev/symposium/issues/533) reports the fetch error instead.

### B3. Agent evaluations (AT-EVAL)

From the symposium checkout in `$DEMO/src`, `cargo xtask agent-eval --check-tasks` prints `ok formatting`, `ok store-durability` and `checked 2 tasks`. Real trials run under `srt` and need Linux, so none were run for this.

## 6. just-right-rust for every workspace (AT-INSTALL, AT-INSTALL-04 is [#451](https://github.com/symposium-dev/symposium/issues/451))

Run this one last: it installs a plugin for every workspace. It uses just-right-rust `main` at e3e2da7, which merged [symposium-dev/just-right-rust#3](https://github.com/symposium-dev/just-right-rust/pull/3): a rules entry skill, six guides, and two script hooks. crates.io still has 0.1.1, whose hooks stall 0.7.0, so do not take that one (see the end of this topic). Instead, `main` is served from a path registry, since a registry plugin wins over the crate of the same name. The registry root cannot itself be an entry, which is why the clone is linked from a separate folder.

```bash
git clone -q https://github.com/symposium-dev/just-right-rust "$DEMO/just-right-rust"
mkdir -p "$DEMO/registry" && ln -s "$DEMO/just-right-rust" "$DEMO/registry/just-right-rust"
echo "$DEMO/registry"
```

Append the following to `~/.symposium/config.toml`. Replace the `path` value with the absolute path the `echo` above printed:

```toml
[[registry]]
name = "jrr-main"
path = "/Users/you/symposium-demo/registry"
```

Then run:

```bash
symposium status | grep just-right
cd ~
symposium use just-right-rust --global
ls ~/.claude/skills/just-right-rust/skills
codex debug prompt-input hi | grep -o 'just-right-rust:[a-z-]*' | sort -u
```

```text
✅ just-right-rust — registry `jrr-main`
✅ enabled just-right-rust for every workspace
installed plugin just-right-rust for claude -> ~/.claude/skills/just-right-rust
installed plugin just-right-rust for codex -> ~/.codex/plugins/cache/symposium/just-right-rust
async-rust
run-rust-tests
rust-comments
rust-domain-types
rust-error-discipline
rust-keep-it-simple
rust-ownership
rust-rules
rust-test-quality
unsafe-rust-review
just-right-rust:async-rust
...
just-right-rust:unsafe-rust-review
```

`use --global` takes about 3.5 s, because the `unsafe-rust-review` group is a `source.git` group fetched from google/rust-skills. The manifest has no `version`, so the compiled plugin is `0.0.0`, and Codex caches it under `.../just-right-rust/0.0.0`.

The hooks need a git repository:
- At SessionStart, the hook records the working tree and returns the rules as context.
- At Stop, it checks only the Rust lines added since then.

```bash
cd "$DEMO/work/demo-app"
git init -q && git add Cargo.toml Cargo.lock Symposium.toml src scripts && git -c user.name=demo -c user.email=demo@localhost commit -qm base
S=$(printf '{"session_id":"jrr","cwd":"%s","hook_event_name":"SessionStart","source":"startup"}' "$PWD")
echo "$S" | symposium hook claude session-start | python3 -c 'import json,sys; print(len(json.load(sys.stdin)["hookSpecificOutput"]["additionalContext"].splitlines()), "lines of context")'
```

Now play the session's edit by appending this to `src/main.rs`:

```rust

fn parse(s: &str) -> Result<u16, String> {
    let n: i64 = s.parse().unwrap();
    Ok(n as u16)
}
```

Then fire Stop twice:

```bash
P=$(printf '{"session_id":"jrr","cwd":"%s","hook_event_name":"Stop","stop_hook_active":false}' "$PWD")
echo "$P" | symposium hook claude stop; echo "exit $?"
echo "$P" | symposium hook claude stop; echo "exit $?"
```

```text
88 lines of context
just-right-rust check: findings in the Rust lines this session added.
untyped error: src/main.rs:9: fn parse(s: &str) -> Result<u16, String> {
panic on failure: src/main.rs:10: let n: i64 = s.parse().unwrap();
unchecked cast (fine only when it widens): src/main.rs:11: Ok(n as u16)
Fix each one in the lines you added, changing existing code just enough where that is what it takes. That the existing code does the same is not a reason to keep one. Keep a finding only when the rule does not apply to that line, and say why in one line. Then finish.exit 2
{}exit 0
```

What these outputs show:
- **Stop:** the findings go to stderr with exit 2, which blocks Claude's stop and hands them to the model. Each finding is reported once, so the second Stop prints `{}`.
- **SessionStart:** it takes about 1.4 s, including the registry freshness check, and it adds the rules in any folder, workspace or not.
- **PreToolUse:** it answers in about 0.04 s.

With 0.1.1 from crates.io, which is what `symposium use just-right-rust --global` installs without the registry, the hooks need a helper binary. Symposium `cargo install`s that binary inside each hook event and never finishes within the 8 s hook budget. Every event then waits 8 to 9 s and leaves a ~200 MB `cargo-install*` folder in `$TMPDIR`. Two changes fix it:
- Publishing `main` as 0.1.2.
- On the symposium side, [#512](https://github.com/symposium-dev/symposium/issues/512), which installs hook helpers at `use` and `sync` instead of per event.

## Bugs this pass found, with fixes open

All of them reproduce on main:
- [#532](https://github.com/symposium-dev/symposium/issues/532): switching `hook-scope` left the old scope registered.
  - The old scope's hooks stayed, so Codex and Copilot ran every hook twice; Claude Code dedups identical commands.
  - After a switch from global to project, the user-level MCP entries stayed, so the servers kept running in every project.
- [#533](https://github.com/symposium-dev/symposium/issues/533): output fixes.
  - `-q` did not silence report output.
  - `plugin sync` hid failed fetches and exited 0.
  - The unavailable-PM error repeated itself.
  - Plugin install lines did not match skill install lines.
  - `plugin validate` on a crate listed each skill twice and dropped `source.git` groups.
- [#534](https://github.com/symposium-dev/symposium/issues/534): Codex 0.159.3 only watches skill roots that exist at session start, so skills copied into a new `.agents/skills/` mid-session never load. Codex now gets a "start a new session" notice.
- [#535](https://github.com/symposium-dev/symposium/issues/535): `use X --remove` now also clears X from `auto-enable` and `disable`, the only way to undo a consent answer without editing the config.

## Open gaps

1. just-right-rust 0.1.1 from crates.io stalls every hook (topic 6). Publishing `main` fixes it, and so does #512 on the symposium side.
2. Global packages ignore predicates: every condition counts as met (topic 6). just-right-rust uses `depends-on = ["*"]`, so this demo cannot show the difference.
3. `use` from another terminal does not tell a running session to reload (topic 3).
4. The audits list is private and empty (B2).
5. Eval trials are Linux only (B3).
6. Telemetry asks for consent but records nothing.

## Reset

If you logged Claude in inside the sandbox, run `claude auth logout` there first: the login may live in your keychain. Then remove the sandbox, which holds everything the demo wrote:

```bash
rm -rf "${DEMO:?}"
```
