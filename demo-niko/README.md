# Symposium demo: 0.7.0

A self-guided walk through what Symposium does today, organized by the user journeys on the project board. Each topic says what to run and what you should see. Every expected output below was captured with symposium 0.7.0 on macOS on 2026-11-23, using the fixtures in this folder.

Time: about 40 minutes for the core topics (1 to 6), 15 more for the bonus ones.

## How each topic is laid out

- **Journey**: the board lane and admission tests it touches.
- **Why it matters**: one line.
- **Steps**: commands to copy, run from the demo shell.
- **Expected**: what the command prints, trimmed to the lines that matter, and what to look for.
- **In the agent**: what to check in the agent's TUI.
- **Status**: what works and what does not yet.
- **Reset**: how to undo the topic, when later topics would see it.
- **Open question**: the decision the topic surfaces, when there is one.

`<DEMO>` in the outputs stands for your `~/symposium-demo`. `~` in Symposium's own output is the sandbox home, `<DEMO>/home`.

## 0. Setup (5 min)

### What you need

- A Rust toolchain installed with rustup, git, and network access (registries are fetched from GitHub).
- Claude Code and, optionally, Codex CLI. Copilot CLI, Antigravity CLI and Goose work too, but the steps below show Claude and Codex.
- `cargo-binstall` makes the install take seconds. Without it, `cargo install` builds symposium in a few minutes.

### Enter the sandbox

Symposium writes to your agents' configuration (`~/.claude/settings.json`, `~/.codex/hooks.json`, skill folders). To keep your real setup untouched, the demo runs with `HOME` pointing at a throwaway folder:

```bash
cd <this folder>
. ./env.sh
```

It prints `demo shell: HOME=/Users/<you>/symposium-demo/home`. Rustup, the cargo registry cache, asdf and mise keep using your real folders, so toolchains and agent binaries still work. Source `env.sh` again in every terminal you use for the demo.

### Install symposium 0.7.0 into the sandbox

```bash
cargo binstall symposium --version 0.7.0 --root "$DEMO" -y
```

Without cargo-binstall:

```bash
cargo install symposium --version 0.7.0 --locked --root "$DEMO"
```

Then `symposium --version` prints `symposium 0.7.0`. Three binaries land in `$DEMO/bin`: `symposium`, `cargo-agents` and `symposium-pm-cargo`.

### Copy the fixtures

```bash
mkdir -p "$DEMO/work" && cp -R fixtures/. "$DEMO/work/"
```

| Fixture | What it is |
|---|---|
| `demo-app/` | A binary crate depending on `anyhow` and on `acme-retry`. Its `Symposium.toml` holds a team hook and a team command. |
| `acme-retry/` | A dependency that ships a skill (`skills/retry-policies/`). |
| `acme-metrics/` | A second such dependency, added during a session in topic 3. |
| `plugins/rust-review/` | An Agent Plugins package: `plugin.json` and one skill. |
| `plugins/claude-changelog/` | A Claude Code plugin: one command and one `Stop` hook. |

### Log your agents in, inside the sandbox

The sandbox home has no agent logins yet:

- Claude Code: `claude auth login`. Or run `claude setup-token` in a normal terminal and `export CLAUDE_CODE_OAUTH_TOKEN=<token>` in the demo shell.
- Codex CLI: `codex login`, or copy your login: `mkdir -p ~/.codex && cp "$SYMPOSIUM_DEMO_REAL_HOME/.codex/auth.json" ~/.codex/`.

`claude auth status` should report `"loggedIn": true`.

## 1. Set up Symposium

**Journey**: Set up Symposium (AT-SETUP-01, #424).

**Why it matters**: first contact. One command configures every agent you have.

**Steps**:

```bash
cd "$DEMO/work/demo-app"
symposium init
```

Press Enter at each question to take the defaults.

**Expected**:

```text
Setting up symposium for your user account.

ℹ️  Detected: Claude Code, Codex CLI
Which agents do you use? (space to select, enter to confirm):
> [ ] Antigravity CLI
  [x] Claude Code
  [x] Codex CLI
  [ ] GitHub Copilot
  [ ] Goose
  [ ] Kiro
  [ ] OpenCode
...
Install hooks and agent configuration: Globally (recommended)
Automatic updates: Auto-update (recommended)
Enable anonymous usage telemetry? It is stored locally under ~/.symposium/telemetry/ and never uploaded automatically — you can review it with `symposium telemetry show` and share it yourself no
✅ ~/.symposium/config.toml: wrote user config (agents: Claude Code, Codex CLI)
✅ ~/.claude/settings.json: added hooks (PreToolUse, PostToolUse, UserPromptSubmit, SessionStart, Stop)
✅ ~/.codex/hooks.json: added hooks (PreToolUse, PostToolUse, UserPromptSubmit, SessionStart, Stop)
```

Look for: the agents you logged into come preselected, detected by their config folder. The hook written into each agent's config calls `symposium`, which 0.7.0 made the primary command (`cargo agents` still works):

```bash
grep -m1 'symposium hook' ~/.claude/settings.json
```

```text
            "command": "symposium hook claude pre-tool-use"
```

**Status**: works. The telemetry question is real, but nothing is recorded yet.

## 2. Skills arrive with your dependencies

**Journey**: Discover capabilities while coding (AT-DISCOVERY).

**Why it matters**: the core promise. Depending on `anyhow` brings error-handling guidance with no configuration.

**Steps**:

```bash
symposium status
```

**Expected**:

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

Look for: `error-battery-pack` is active because the workspace depends on `anyhow`. `dial9` and the other sleeping entries wait for a dependency this workspace lacks. `rtk` names no dependency, so it waits for `symposium use`. `acme-retry` is a dependency's own plugin and waits for consent (topic 3). The registry entries are whatever symposium-dev/recommendations holds the day you run it.

Now install. Sync asks about `acme-retry`; press Enter ("Ask me later") for now:

```bash
symposium sync
```

**Expected**:

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

Look for: Claude Code gets each plugin as one project plugin under `.claude/skills/<plugin>/`; Codex gets per-skill copies under `.agents/skills/`. Everything Symposium installs carries a `.gitignore` containing `*`, so the project's `git status` stays clean.

**In the agent**:

- Run `claude` in `<DEMO>/work/demo-app` and trust the folder (Claude Code loads project plugins only in a trusted folder, started at the workspace root). `/skills` lists `application-errors`, `rust-best-practice` and the rest. Ask "how should I handle errors in this crate?".
- Run `codex` in the same folder and ask the same. To check without a model call: `codex debug prompt-input hi | grep -o -- '- [a-z-]*: '`.

**Status**: works. Checked while preparing this: the CLI and the Codex listing. The Claude TUI step was not run.

## 3. A dependency's own plugin needs consent

**Journey**: Discover capabilities while coding, Approve plugins (AT-DISCOVERY, AT-APPROVAL).

**Why it matters**: depending on a crate means compiling its code, not letting its author inject agent context. A dependency's plugin stays off until the user says yes, and the agent is told so it can ask.

### 3a. What the agent hears when a session starts

This is the JSON Claude Code sends at session start, piped by hand:

```bash
printf '{"session_id":"demo","cwd":"%s","hook_event_name":"SessionStart","source":"startup"}' "$PWD" \
  | symposium hook claude session-start | python3 -m json.tool
```

**Expected**:

```json
{
    "hookSpecificOutput": {
        "hookEventName": "SessionStart",
        "additionalContext": "This project has crate-aware tools available via `symposium`. Run `symposium --help` to list them before working with the Rust code. Only use tools under the 'Commands for agents' section unless the user explicitly asks you to run one from 'Commands for humans'.\n\nThese dependencies ship agent plugins that are not enabled yet: acme-retry. They stay off until the user consents. Tell the user to run `symposium sync` (which asks about each one) or `symposium use <name>`. Do not enable them yourself.",
        "reloadSkills": true
    }
}
```

**In the agent**: start `claude` in demo-app and ask "is there anything I should set up for this project?". It should name `acme-retry` and point you at `symposium sync`.

### 3b. Say yes

```bash
symposium sync
```

Pick "Enable" (arrow down, then Enter).

**Expected**:

```text
Dependency `acme-retry` provides 1 skill group. Enable it?:
> Ask me later
  Enable
  No — don't ask again
...
Dependency `acme-retry` provides 1 skill group. Enable it?: Enable
ℹ️  enabled dependency plugins: acme-retry
ℹ️  scanning 2 workspace dependencies
installed plugin acme-retry for claude -> <DEMO>/work/demo-app/.claude/skills/acme-retry
✅ installed skill retry-policies for codex → <DEMO>/work/demo-app/.agents/skills/retry-policies
```

```bash
symposium status | tail -1
grep -A1 '^\[plugins\]' ~/.symposium/config.toml
```

```text
✅ acme-retry 0.3.1 — `[plugins] auto-enable`
[plugins]
auto-enable = ["acme-retry"]
```

Look for: the answer is recorded in the user config. "Ask me later", the default, records nothing, so pressing Enter never declines anything. "No" goes to `[plugins] disable`.

### 3c. A dependency added in the middle of a session

In a Claude session in demo-app, ask: "add acme-metrics as a path dependency from ../acme-metrics". After the agent runs `cargo add`, Symposium sees `Cargo.lock` change, re-syncs, and tells the agent, once.

Or by hand:

```bash
cargo add acme-metrics --path ../acme-metrics
P=$(printf '{"session_id":"demo","cwd":"%s","hook_event_name":"PostToolUse","tool_name":"Bash","tool_input":{"command":"cargo add acme-metrics --path ../acme-metrics"},"tool_response":{"stdout":""}}' "$PWD")
echo "$P" | symposium hook claude post-tool-use | python3 -m json.tool
echo "$P" | symposium hook claude post-tool-use; echo
```

**Expected**: the first call names the new plugin,

```json
{
    "hookSpecificOutput": {
        "hookEventName": "PostToolUse",
        "additionalContext": "These dependencies ship agent plugins that are not enabled yet: acme-metrics. They stay off until the user consents. Tell the user to run `symposium sync` (which asks about each one) or `symposium use <name>`. Do not enable them yourself."
    }
}
```

and the second prints `{}`: each new plugin is named once. If the agent already got the notice, the first call prints `{}` too.

### 3d. Enable by name

```bash
symposium use acme-metrics
symposium status | tail -2
```

**Expected**:

```text
✅ enabled acme-metrics for this workspace
ℹ️  scanning 3 workspace dependencies
installed plugin acme-metrics for claude -> <DEMO>/work/demo-app/.claude/skills/acme-metrics
✅ installed skill metric-naming for codex → <DEMO>/work/demo-app/.agents/skills/metric-naming
✅ acme-metrics 1.2.0 — `[plugins] use`
✅ acme-retry 0.3.1 — `[plugins] auto-enable`
```

**In the agent**: a running Claude session needs `/reload-plugins` to see `acme-metrics`.

**Status**: works. Gap: a `symposium use` run from another terminal does not tell the running session to reload.

## 4. Team rules in the repository: one hook for every agent, and a team command

**Journey**: Get the team setup, Publish portable hooks (AT-TEAM, AT-HOOK).

**Why it matters**: a repository carries its conventions in its own `Symposium.toml`, written once in Symposium's hook format and enforced on every agent. The workspace is a trust root, so nobody is asked for consent.

The fixture `demo-app/Symposium.toml`:

```toml
[[hooks]]
name = "no-publish"
event = "PreToolUse"
matcher = "^(Bash|bash|run_command|execute_bash)$"
command = { script = "scripts/no-publish.sh" }

[subcommand.demo-check]
description = "Run the demo-app team checks (fmt, tests)"
audience = "agents"
command = { script = "scripts/demo-check.sh" }
```

`scripts/no-publish.sh` exits 2 with a reason when the command contains `cargo publish` without `--dry-run`.

### 4a. The hook, as three agents see it

```bash
printf '{"session_id":"demo","cwd":"%s","hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"cargo publish"}}' "$PWD" | symposium hook claude pre-tool-use; echo
printf '{"session_id":"demo","cwd":"%s","hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"cargo publish --dry-run"}}' "$PWD" | symposium hook claude pre-tool-use; echo
printf '{"session_id":"demo","turn_id":"t1","cwd":"%s","hook_event_name":"PreToolUse","model":"m","tool_name":"Bash","tool_use_id":"u1","tool_input":{"command":"cargo publish"}}' "$PWD" | symposium hook codex pre-tool-use; echo
printf '{"timestamp":1,"cwd":"%s","toolName":"bash","toolArgs":"{\\"command\\":\\"cargo publish\\"}"}' "$PWD" | symposium hook copilot pre-tool-use; echo
```

**Expected** (Claude, Claude with `--dry-run`, Codex, Copilot):

```text
{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"deny","permissionDecisionReason":"demo-app team rule: never run cargo publish from an agent session; use cargo publish --dry-run"}}
{}
{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"deny","permissionDecisionReason":"demo-app team rule: never run cargo publish from an agent session; use cargo publish --dry-run"}}
{"permissionDecision":"deny","permissionDecisionReason":"demo-app team rule: never run cargo publish from an agent session; use cargo publish --dry-run"}
```

Look for: one shell script. Symposium hands it each agent's event in one canonical shape and returns the decision in each agent's own shape.

**In the agent**: ask Claude "publish this crate to crates.io". The Bash call is denied and Claude reports the team rule. Codex does the same; the first time it starts, it asks you to review the hooks, so trust them.

### 4b. The team command

```bash
symposium --help
symposium demo-check
```

**Expected** (`--help`, trimmed):

```text
Commands for humans:
init         Set up user-wide configuration
...
use          Enable a plugin by name and sync it into the workspace

Commands for agents:
crate-info   Find crate sources
demo-check   Run the demo-app team checks (fmt, tests)
```

and `demo-check` runs the script: `demo-check: cargo fmt --check`, the test run, then `demo-check: all good`.

Look for: help splits commands for humans and for agents. `demo-check` comes from the repository and exists only inside it: compare with `symposium --help` run from `~`. The session-start context in 3a already tells the agent to run `symposium --help`.

**Status**: works. Portable hook binaries (`hook::hook_main`) are a draft, #486.

## 5. Writing and trying a plugin

**Journey**: plugin authoring.

**Why it matters**: a plugin author can start from the Agent Plugins format or from an existing Claude Code plugin, see what will not port, and use the plugin in real agents before publishing it.

### 5a. Validate an Agent Plugins package

```bash
cd "$DEMO/work/plugins"
symposium plugin validate rust-review
```

**Expected**:

```text
  ⚠️  rust-review (agent plugin): plugin `rust-review` references no dependency; it stays dormant until enabled with `symposium use rust-review`
  ✅ review-unsafe (skill)
```

### 5b. Import a Claude Code plugin

```bash
symposium plugin import claude-changelog --out imported
find imported -type f | sort
```

**Expected**:

```text
⚠️  name `Claude-Changelog` normalized to `claude-changelog`
Imported claude-changelog into imported/claude-changelog
  ⚠️  claude-changelog (plugin): plugin `claude-changelog` references no dependency; it stays dormant until enabled with `symposium use claude-changelog`
  ⚠️  claude-changelog (plugin): `Stop` hooks have only the `claude` format, so they fire on no other agent; add a `format = "symposium"` hook for the others
  ✅ changelog (skill)
imported/claude-changelog/Symposium.toml
imported/claude-changelog/plugin.json
imported/claude-changelog/skills/changelog/SKILL.md
```

Look for: the command became a skill, the hook kept Claude's format (it runs only in Claude Code, with `CLAUDE_PLUGIN_ROOT` set), and validation says exactly what will not port.

### 5c. Try a plugin in your agents

```bash
cd "$DEMO/work/demo-app"
symposium plugin try ../plugins/rust-review
```

**Expected**:

```text
trying rust-review in every workspace; edits reach your agents on the next sync
ℹ️  scanning 3 workspace dependencies
installed plugin rust-review for claude -> ~/.claude/skills/rust-review
installed plugin rust-review for codex -> ~/.codex/plugins/cache/symposium/rust-review
ℹ️  Claude Code has no listing command; start a new session here and run `/skills`
ℹ️  Codex CLI lists review-unsafe
```

Look for: `try` links your folder, so an edit reaches the agents on the next sync, and it asks each agent what it actually loaded.

**In the agent**: a new Claude session lists `review-unsafe` under `/skills`.

**Reset**:

```bash
symposium plugin try --remove rust-review
```

```text
stopped trying rust-review
ℹ️  scanning 3 workspace dependencies
➖ removed ~/.symposium/installed/rust-review
➖ removed ~/.claude/skills/rust-review
➖ removed ~/.codex/plugins/cache/symposium/rust-review
```

**Status**: works.

## 6. A ready-made setup for every workspace (do this one last)

**Journey**: Install a ready-made setup (AT-INSTALL, AT-INSTALL-04 is #451).

**Why it matters**: one command installs a curated package, here `just-right-rust`, for the user in every agent that keeps plugins at user level, instead of per project.

**Steps**:

```bash
cd ~
symposium use just-right-rust --global
```

**Expected** (about 10 seconds):

```text
✅ enabled just-right-rust for every workspace
installed plugin just-right-rust for claude -> ~/.claude/skills/just-right-rust
installed plugin just-right-rust for codex -> ~/.codex/plugins/cache/symposium/just-right-rust
```

```bash
ls ~/.claude/skills/just-right-rust/skills
grep -A1 just-right ~/.codex/config.toml
```

```text
async-rust
unsafe-rust-review
[plugins."just-right-rust@symposium"]
enabled = true
```

**In the agent**: a new session in any folder lists both skills. For Codex without a model call, `codex debug prompt-input hi | grep -o 'just-right-rust:[a-z-]*'` prints `just-right-rust:async-rust` and `just-right-rust:unsafe-rust-review`.

**Status**: the install works; living with it does not yet. just-right-rust 0.1.1 declares hooks whose helper binary Symposium tries to `cargo install` inside each hook event. The install never finishes within the 8 second hook budget, so **every hook event in a Rust workspace waits 8 to 9 seconds**, and each one leaves a `cargo-install*` folder of about 200 MB in `$TMPDIR`. To see it:

```bash
cd "$DEMO/work/demo-app"
time (printf '{"session_id":"demo","cwd":"%s","hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"ls"}}' "$PWD" | symposium hook claude pre-tool-use)
```

It prints `{}` after about 9 seconds (instantly without the plugin). Two fixes are waiting: symposium #512 installs hook helpers at `use` and `sync` instead of per event, and just-right-rust 0.1.2 drops the no-op hooks (symposium-dev/just-right-rust#2, which needs a crates.io publish by an owner).

**Reset** (before going back to any other topic):

```bash
symposium use just-right-rust --global --remove
```

```text
➖ removed the just-right-rust enablement for every workspace
ℹ️  scanning 3 workspace dependencies
➖ removed ~/.symposium/installed/just-right-rust
➖ removed ~/.claude/skills/just-right-rust
➖ removed ~/.codex/plugins/cache/symposium/just-right-rust
```

Then, if no other `cargo install` is running, delete the leftovers: `rm -rf "${TMPDIR:-/tmp}"/cargo-install*`.

**Open question**: a global package ignores predicates today (every condition counts as met, so it serves every workspace). On 2026-11-03 you answered that predicates should still filter (AT-INSTALL-04). Which way do we go? just-right-rust uses `depends-on = ["*"]`, so this demo cannot show the difference.

## Bonus B1. Package managers out of process

**Journey**: Ecosystems (AT-ECOSYSTEM).

**Why it matters**: the cargo package manager is its own binary speaking JSON-RPC over stdio. Another ecosystem plugs in the same way, through a `[[package-manager]]` entry.

**Steps**: append this to `~/.symposium/config.toml`. The wrapper logs each spawn, then runs the real binary:

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

**Expected**: the same status lines as before, and one `spawned symposium-pm-cargo` line per command:

```text
✅ demo-app — workspace member
✅ acme-metrics 1.2.0 — `[plugins] use`
✅ acme-retry 0.3.1 — `[plugins] auto-enable`
11:07:29 spawned symposium-pm-cargo
```

Look for: an entry named `cargo` replaces the built-in, in-process cargo package manager; any other name adds an ecosystem.

**Reset**: delete the block from `~/.symposium/config.toml`.

**Status**: works. There is no second real ecosystem yet.

## Bonus B2. Audits

**Journey**: Approve plugins, Policy (AT-APPROVAL, AT-POLICY).

**Why it matters**: a community list records which dependency plugin payloads were reviewed and which releases are revoked. A revoked release never loads, and the consent prompt and `status` show the list's verdict.

**What works today**: the payload digest an audit entry is keyed on.

```bash
symposium plugin digest ../acme-retry
```

```text
sha256:e98011210ebf04e22fe2cc2b0a6ebbc31163c1c2337b0e796745496a6a697af6
```

**What cannot be shown yet**: the list, `symposium-dev/audits`, is a private repository with no entries. With `[defaults] symposium-audits = true` the fetch fails, so the prompt and `status` show no verdict at all, and `symposium plugin sync symposium-audits` prints `No git source found for provider: symposium-audits` (the failed fetch only reaches the log).

**Open question**: when does the list go public, and who curates its first entries?

## Bonus B3. Agent evaluations

**Journey**: Evaluate (AT-EVAL).

**Why it matters**: it measures whether Symposium helps: a real agent works on a small Rust task with and without Symposium, the result is graded, and what Symposium did along the way is checked.

**Steps**, from a symposium checkout:

```bash
cargo xtask agent-eval --check-tasks
```

**Expected**:

```text
ok    formatting
ok    store-durability
checked 2 tasks
```

A real batch is `cargo xtask agent-eval formatting --client claude --model <model>`. It needs Linux (trials run under `srt`) and a Claude credential, and it keeps results under `results/`. It was not run while preparing this.

## Known gaps, in one place

1. A global just-right-rust makes every hook event wait 8 to 9 seconds and leaves about 200 MB in `$TMPDIR` per event (topic 6). Fixes: symposium #512, just-right-rust 0.1.2.
2. Global packages ignore predicates (topic 6).
3. `symposium use` from another terminal does not tell a running session to reload (topic 3).
4. The audits list is private and empty, and `plugin sync symposium-audits` reports its failed fetch as a missing source (B2).
5. Agent evaluation trials run on Linux only (B3).
6. Telemetry asks for consent and has its storage, but records nothing yet.

## Questions for you

1. Global packages and predicates: filter per workspace or not (topic 6)?
2. Can you publish just-right-rust 0.1.2, or add another owner on crates.io? You are its only owner.
3. When does the audits list go public, and who curates it (B2)?
4. Does the delivery you saw in topics 2 and 5 (one plugin per project for Claude Code, per-skill copies for Codex, user-level plugins for `--global` and `try`) match where you want plugins to go?

## Reset everything

If you logged Claude Code in inside the sandbox, run `claude auth logout` in the demo shell first, since the login may live in your keychain. Then:

```bash
./reset.sh
```

It deletes `~/symposium-demo`: the sandbox home, the demo binaries and the work copies. Nothing outside it was written.
