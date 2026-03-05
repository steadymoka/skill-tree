<p align="center">
  <h1 align="center">skilltree</h1>
  <p align="center">
    A skill manager for AI coding agents — organize, tag, and link skills across projects.
  </p>
  <p align="center">
    <a href="https://www.npmjs.com/package/@steadymoka/skilltree"><img alt="npm" src="https://img.shields.io/npm/v/@steadymoka/skilltree?color=blue&label=npm"></a>
    <a href="https://crates.io/crates/skilltree"><img alt="crates.io" src="https://img.shields.io/crates/v/skilltree?color=orange"></a>
    <a href="https://github.com/steadymoka/skilltree/blob/main/LICENSE"><img alt="license" src="https://img.shields.io/github/license/steadymoka/skilltree"></a>
  </p>
</p>

<br>

> Install, manage, and link reusable skills for **Claude Code** and **Codex** — from GitHub or locally, via CLI or TUI.

<br>

## Quick Start

```bash
npm i -g @steadymoka/skilltree   # install
skilltree init                    # set up ~/.skilltree/ (migrates from .claude/.codex/.agents)
skilltree add owner/repo          # install a skill from GitHub
skilltree link <tags...>          # link skills to your project
skilltree                         # launch TUI
```

<details>
<summary>Build from source</summary>

```bash
git clone https://github.com/steadymoka/skilltree.git
cd skilltree
cargo build --release
```

</details>

<br>

## TUI

Run `skilltree` with no arguments to launch the interactive terminal UI.

Switch between three screens with `1`, `2`, and `3`. The focused panel is highlighted while the other panel dims automatically.

### Screen 1 — Skills & Tags

Organize your skills with tags. Select a skill on the left, toggle tags on the right.

```
 Skill Tree   12 skills  5 tags    1:Skills  2:Claude  3:Codex
╭─ Skills ─────────────────────╮╭─ Tags ───────────────────────╮
│ ▸ auth-middleware  [api,sec] ││   [✓] api                    │
│   db-migrations    [db]      ││   [ ] db                     │
│   error-handling   [api]     ││   [ ] frontend               │
│   graphql-setup              ││   [✓] sec                    │
│   react-patterns   [frontend]││   [ ] testing                │
│   test-helpers     [testing] ││                              │
╰──────────────────────────────╯╰──────────────────────────────╯
 1/2/3:screen  ←→:focus  ↑↓:select  Space:toggle  a:new tag  q:quit
```

### Screen 2/3 — Claude / Codex Projects

Link skills to projects per agent. Select a project on the left, then toggle skills or entire tag groups on the right.

```
 Skill Tree   12 skills  5 tags    1:Skills  2:Claude  3:Codex
╭─ Projects ───────────────────╮╭─ Skills by Tag ──────────────╮
│ ▸ my-api         3 linked    ││ ▾ [✓] api                 2  │
│   web-app        1 linked    ││     [✓] auth-middleware       │
│   mobile-app     0 linked    ││     [✓] error-handling        │
│                               ││ ▸ [ ] db                  1  │
│                               ││ ▾ [-] sec                 1  │
│                               ││     [✓] auth-middleware       │
│                               ││ ── no tag ──                 │
│                               ││     [ ] graphql-setup         │
╰──────────────────────────────╯╰──────────────────────────────╯
 1/2/3:screen  ←→:focus  ↑↓:select  Space:link/unlink  Enter:fold  q:quit
```

### Keybindings

| Key | Action |
|:---:|--------|
| `1` `2` `3` | Switch screen |
| `←` `→` | Switch panel focus |
| `↑` `↓` `j` `k` | Navigate |
| `Space` | Toggle tag / link |
| `Enter` | Fold/unfold tag group |
| `a` | Add new tag |
| `q` | Quit |

<br>

## CLI Commands

### Skill Management

```bash
skilltree add <owner/repo>            # Install a skill from GitHub
skilltree add <owner/repo> --skill <path>  # Install a specific skill within a repo
skilltree search <query>              # Search for skills on GitHub
skilltree update [skill]              # Update skill(s) to latest
skilltree remove <name>               # Remove an installed skill
skilltree info <name>                 # Show skill details
skilltree tag <skill> <tags...>       # Set tags for a skill
```

### Linking

```bash
skilltree link <tags...>              # Link skills by tags
skilltree link --skill <name>         # Link a single skill
skilltree unlink <name>               # Unlink a skill
skilltree unlink --all                # Unlink all skills
```

### Other

```bash
skilltree init                        # Initialize ~/.skilltree/
skilltree tree                        # Print skill tree (alias: list)
skilltree doctor                      # Check health & detect issues
skilltree doctor --fix                # Auto-fix all detected issues
skilltree serve                       # Open web dashboard
```

### Common Flags

| Flag | Description |
|------|-------------|
| `--path <dir>` | Target project path (default: cwd) |
| `--tool claude\|codex` | Target agent (default: `claude`) |

<br>

## How It Works

```
~/.skilltree/               Central skill repository
├── skills.yaml             Skill → tag mapping
├── .skill-lock.json        Source & version metadata
├── auth-middleware/         Skill directory (local or from GitHub)
│   └── SKILL.md
├── db-migrations/
└── ...

<project>/.claude/skills/   Symlinks (per-project, auto-created)
├── auth-middleware → ~/.skilltree/auth-middleware
└── error-handling → ~/.skilltree/error-handling
```

Skills live in one place (`~/.skilltree/`). Projects reference them via **symlinks** — no duplication, always up to date. Install skills from GitHub with `skilltree add`, or create your own locally. Tags let you group related skills and link them in bulk.

### Doctor

`skilltree doctor` checks the health of your setup:

- **Ghost entries** — skills.yaml references a skill directory that doesn't exist
- **Unregistered directories** — skill directory exists but isn't in skills.yaml
- **Lock orphans** — .skill-lock.json references a removed skill
- **Unmanaged skills** — real directories or external symlinks in `~/.claude/skills/`, `~/.codex/skills/`, or `~/.agents/skills/` that aren't managed by skilltree
- **Broken symlinks** — dead symlinks in project skill directories

Run `skilltree doctor --fix` to auto-fix all detected issues. Unmanaged skills are adopted into `~/.skilltree/` and replaced with symlinks.

<br>

## Development

```bash
cargo run          # Run CLI
cargo test         # Run tests

cd web && pnpm install && pnpm dev   # Web dashboard
```

## License

MIT
