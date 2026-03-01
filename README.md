# iron-sight

A terminal CSV/Parquet viewer with vim-style navigation, built with Rust and ratatui. Themed with [Catppuccin Mocha](https://github.com/catppuccin/catppuccin).

## Features

- Vim-style navigation (`hjkl`, `g`/`G`, `PageUp`/`PageDown`)
- Search within a column (`/`, `n`/`N`)
- Multi-column filtering (`f`, `F`)
- Sort by any column (`s`)
- Group-by with per-column aggregations (`b`, `a`, `B`)
- Column stats popup (`S`)
- In-app help popup (`?`)
- Catppuccin Mocha color theme with zebra-striped rows and mode-aware status bar
- Supports CSV and Parquet files

## Requirements

- Rust 1.75 or higher

## Build & Install

```
# Run directly
cargo run -- <path-to-file.csv>
cargo run -- <path-to-file.parquet>

# Install
cargo install --path .
iron-sight <path-to-file.csv>
```

## Keybindings

### Navigation

| Key | Action |
|-----|--------|
| `j` / `Down` | Move down |
| `k` / `Up` | Move up |
| `h` / `Left` | Move left |
| `l` / `Right` | Move right |
| `g` / `Home` | Jump to first row |
| `G` / `End` | Jump to last row |
| `PageDown` | Scroll down 20 rows |
| `PageUp` | Scroll up 20 rows |

### Search

| Key | Action |
|-----|--------|
| `/` | Enter search mode (searches in current column) |
| `Enter` | Confirm search and jump to first match |
| `n` | Next match |
| `N` | Previous match |
| `Esc` | Exit search and clear results |

### Filter

| Key | Action |
|-----|--------|
| `f` | Enter filter mode (filters rows by current column) |
| `Enter` | Confirm filter and return to normal mode |
| `F` | Clear active filter |
| `Esc` | Exit filter and clear results |

### Sort

| Key | Action |
|-----|--------|
| `s` | Sort by current column (toggles asc/desc) |

### Group By

| Key | Action |
|-----|--------|
| `b` | Toggle group-by key for current column |
| `a` | Cycle aggregation for current column (Σ μ # ↓ ↑) |
| `B` | Execute group-by / clear and return to full view |

### Column Stats

| Key | Action |
|-----|--------|
| `S` | Toggle stats popup for current column (count, min, max, mean, median) |

### Other

| Key | Action |
|-----|--------|
| `_` | Autofit current column width |
| `?` | Toggle help popup |
| `q` | Quit |
