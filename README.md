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

> Centrally manage reusable skills for **Claude Code** and **Codex**, then link them to any project with a single command — or interactively via TUI.

<br>

## Quick Start

```bash
npm i -g @steadymoka/skilltree   # install
skilltree init                    # set up ~/.skilltree/
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

Switch between two screens with `1` and `2`.

### Screen 1 — Skills & Tags

Organize your skills with tags. Select a skill on the left, toggle tags on the right.

```
 Skill Tree   12 skills  5 tags     1:Skills  2:Projects
╭─ Skills ─────────────────────╮╭─ Tags ───────────────────────╮
│ ▸ auth-middleware  [api,sec] ││ [✓] api                      │
│   db-migrations    [db]      ││ [ ] db                       │
│   error-handling   [api]     ││ [ ] frontend                 │
│   graphql-setup              ││ [✓] sec                      │
│   react-patterns   [frontend]││ [ ] testing                  │
│   test-helpers     [testing] ││                              │
╰──────────────────────────────╯╰──────────────────────────────╯
 Tab:focus  ↑↓:select  Space:toggle  a:new tag  q:quit
```

### Screen 2 — Projects

Link skills to projects. Select a project on the left, then toggle skills or entire tag groups on the right.

```
 Skill Tree   12 skills  5 tags     1:Skills  2:Projects
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
 Tab:focus  ↑↓:select  Space:link/unlink  Enter:fold  q:quit
```

### Keybindings

| Key | Action |
|:---:|--------|
| `1` `2` | Switch screen |
| `Tab` | Switch panel |
| `↑` `↓` `j` `k` | Navigate |
| `Space` | Toggle tag / link |
| `Enter` | Fold/unfold tag group |
| `a` | Add new tag |
| `q` | Quit |

<br>

## CLI Commands

```bash
skilltree init                        # Initialize ~/.skilltree/
skilltree link <tags...>              # Link skills by tags
skilltree link-skill <name>           # Link a single skill
skilltree unlink <name>               # Unlink a skill
skilltree unlink --all                # Unlink all skills
skilltree tree                        # Print skill tree
skilltree serve                       # Open web dashboard
```

| Flag | Description |
|------|-------------|
| `--path <dir>` | Target project path (default: cwd) |
| `--tool claude\|codex` | Target agent (default: `claude`) |

<br>

## How It Works

```
~/.skilltree/               Central skill repository
├── skills.yaml             Skill → tag mapping
├── auth-middleware/         Skill directory
│   └── SKILL.md
├── db-migrations/
└── ...

<project>/.claude/skills/   Symlinks (per-project, auto-created)
├── auth-middleware → ~/.skilltree/auth-middleware
└── error-handling → ~/.skilltree/error-handling
```

Skills live in one place (`~/.skilltree/`). Projects reference them via **symlinks** — no duplication, always up to date. Tags let you group related skills and link them in bulk.

<br>

## Development

```bash
cargo run          # Run CLI
cargo test         # Run tests

cd web && pnpm install && pnpm dev   # Web dashboard
```

## License

MIT
