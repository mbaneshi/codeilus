# Schematic Explorer

The Schematic Explorer is a unified, interactive visualization of your codebase's structure and relationships. It combines a file tree view with a community graph view on a single page.

## Getting There

Navigate to **Explore > Schematic Explorer** or go directly to `/explore/schematic`.

## Modes

### Tree Mode

The default view. Shows your codebase as an expandable directory tree.

- **Click a directory** to expand/collapse its children (loaded lazily from the server)
- **Click a file** to open the detail panel with source code, symbols, and AI explanations
- **Community color stripes** on the left edge of each node show which module it belongs to

### Graph Mode

Click the **Graph** toggle to switch. Shows communities (modules) as a grid.

- Each community card shows its name, symbol count, and color
- **Click a community** in the sidebar to drill into its symbols
- Symbols are laid out with edges showing call/import relationships
- **Breadcrumb navigation** lets you go back to the community overview

## Interactions

| Action | What happens |
|---|---|
| **Hover** | Tooltip with node details, connected edges highlight, unrelated nodes dim |
| **Click** | Expand directory / open detail panel for files and symbols |
| **Double-click** | Deep dive: file → tree view, symbol → source popup, community → learning chapter |
| **Right-click** | Context menu: View source, Ask AI, Add annotation, Focus here, Hide |
| **Scroll wheel** | Zoom in/out |
| **Drag** | Pan the canvas |

## Keyboard Shortcuts

| Key | Action |
|---|---|
| `1` | Switch to Tree mode |
| `2` | Switch to Graph mode |
| `F` | Fit all nodes in viewport |
| `?` | Show keyboard shortcut help |
| `Esc` | Close detail panel / context menu |

## Detail Panel

Click any file or symbol to open the detail panel on the right side with 5 tabs:

- **Overview** — AI-generated explanation, language badge, SLOC, community info
- **Source** — Syntax-highlighted source code (lazy-loaded)
- **Relations** — Callers and callees (click to navigate to that node on the canvas)
- **Learn** — Link to the relevant learning chapter with progress bar
- **Notes** — Add, flag, and delete annotations on the node

## Legend

Click **Legend** in the toolbar to see:

- Node types (directory, file, symbol, community)
- Edge colors (calls = indigo, imports = teal, extends = amber, implements = pink)
- Community color swatches
- Interaction reference

## Minimap

A small overview in the bottom-right corner shows your viewport position. Click or drag on it to pan the main canvas.

## Search

Use the search box in the toolbar to highlight nodes matching your query. Matching nodes get an accent-colored border.
