# aeph

* A minimal TUI markdown paper with task management, heavily influenced by [ephe](https://ephe.app/)

## Quick Start

- [ ] Launch with `ae` command
- [ ] Start typing your notes
- [ ] Press `Ctrl+H` for help

## Editing

Basic navigation and editing:

| Key | Action |
|-----|--------|
| Arrow keys | Move cursor |
| Home / End | Jump to line start/end |
| Page Up/Down | Scroll page |
| Tab | Indent list line |
| Shift+Tab | Unindent list line |
| Ctrl+G | Go to line number |
| Ctrl+Z | Undo |
| Ctrl+S | Save |
| Ctrl+Q | Quit |

## Task Management

Create and manage tasks with checkbox syntax:

- [ ] Incomplete task looks like this
- [x] ~~Completed task with strikethrough~~

| Key | Action |
|-----|--------|
| Ctrl+T | Toggle task status |
| Ctrl+N | Insert new task |

## Documents

Manage up to 9 documents

| Key | Action |
|-----|--------|
| Ctrl+O | Open document picker |
| 1-9 | Quick switch in picker |

## Leader Commands

Vim-style commands with `:` prefix (500ms timeout):

| Command | Action |
|---------|--------|
| `:q` | Quit |
| `:dd` | Delete current line |
| `:yy` | Yank (copy) current line |
| `:p` | Paste below cursor |
| `:1`-`:9` | Switch to document 1-9 |

## Formatting

- [ ] Format markdown with `Ctrl+F`
- [ ] Copy all content with `Ctrl+C`
- [ ] Clear document with `Ctrl+D`

### Smart Lists

Press Enter on a list item to continue:

- Bullet lists continue automatically
- Task lists create new `- [ ]` items
- Numbered lists increment

> Blockquotes also continue
> when you press Enter

### Auto-Pairing

Brackets and quotes auto-pair with cursor centering:

| Input | Result | Cursor Position |
|-------|--------|-----------------|
| `()` | `()` | `(\|)` |
| `[]` | `[]` | `[\|]` |
| `{}` | `{}` | `{\|}` |
| ` `` ` | ` `` ` | `` `\|` `` |
| `""` | `""` | `"\|"` |
| `''` | `''` | `'\|'` |
| `****` | `****` | `**\|**` |
| `~~~~` | `~~~~` | `~~\|~~` |

Type the same character again to escape from the pair and continue typing.

## Customization

### Configuration File

Create `~/.config/aeph/config.toml` to customize behavior:

```toml
# Enable/disable auto-pairing for brackets and quotes (default: true)
auto_pair = true
```

### Environment Variables

Set colors via environment variables:

```
AEPH_BG_DARK="25,25,30"
AEPH_FG_PRIMARY="160,160,160"
AEPH_ACCENT="cyan"
AEPH_ACCENT_YELLOW="yellow"
AEPH_ACCENT_GREEN="green"
```
---

