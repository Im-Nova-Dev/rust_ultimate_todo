# rust_ultimate_todo

A hierarchical todo TUI in Rust (Ratatui). Nested tasks, priorities, status, due dates, tags, filters, undo, and portable project files.

## Run

```bash
cargo run --release
```

Or run the built binary:

```bash
./target/release/rust_tui
```

Open a specific project file:

```bash
rust_tui ~/projects/my-stuff.tdl
```

## Where your data lives

| How you launch | File |
|----------------|------|
| No argument (default) | `~/.local/share/rust_tui/default.tdl` |
| With a path | That `.tdl` file (created on first save) |

`.tdl` files are JSON — portable, editable, and safe to back up or sync.

On save, the app writes a `.bak` backup next to the project (e.g. `default.tdl.bak`). If the main file is corrupt on load, it restores from backup and keeps the broken copy as `.corrupt`.

Autosave runs about 2 seconds after you stop editing. `Ctrl+S` saves immediately.

## Keys (cheatsheet)

Press `?` in the app for the full list. Bottom bar shows common shortcuts while you work.

### Navigation

| Key | Action |
|-----|--------|
| `j` / `k` / arrows | Move selection |
| `g` / `G` | First / last visible item |
| `h` / `l` / Space | Collapse / expand |
| `b` / `u` | Parent task |
| `0` | Jump to top |
| Mouse wheel | Scroll |

### Tasks

| Key | Action |
|-----|--------|
| `a` | Add sibling after current |
| `A` | Add child under current |
| `e` / Enter | Edit (title, desc, priority, status, due, tags) |
| `d` | Delete (press twice to confirm) |
| `D` | Delete without confirm |
| `c` | Duplicate task + subtree |
| `J` / `K` | Move among siblings |
| `>` / `<` | Indent / outdent |
| `p` / `m` | Cycle priority / status |
| `x` | Toggle done / todo |

### Search & filter

| Key | Action |
|-----|--------|
| `/` | Live search |
| `f` | Cycle quick filters |
| `F` | Clear all filters |
| `s` | Cycle sort mode |

### Commands & misc

| Key | Action |
|-----|--------|
| `:` | Command mode (`:42`, `:add …`, `:open path.tdl confirm`, etc.) |
| `Ctrl+Z` | Undo |
| `Ctrl+S` | Save now |
| `q` / `Ctrl+C` | Quit (from normal or help) |

## Useful commands

```
:42                  Jump to task #42
:add Buy milk        Quick-add at root
:open ~/foo.tdl confirm   Open another project
:save-as ~/foo.tdl   Save copy elsewhere
:export              Export to markdown
:reload confirm      Reload from disk
```

## Build

```bash
cargo build --release
```

Binary lands at `target/release/rust_tui` (~1.2 MB).