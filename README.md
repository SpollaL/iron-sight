# iron-sight

A terminal CSV viewer with vim-style navigation, built with Rust and ratatui.

## Requirements

- Rust 1.75 or higher

## Build & Install

```
# Run directly
cargo run -- <path-to-file.csv>

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

### Other

| Key | Action |
|-----|--------|
| `_` | Autofit current column width |
| `q` | Quit |
