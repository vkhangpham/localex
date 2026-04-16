# Localex

Local-first reading shell for Markdown workspaces.

Current repo state:
- `docs/PRD.md` contains product requirements
- Rust backend scaffold under `crates/localex-cli/`
- lightweight Preact reading prototype under `web/`

## Why Localex

Localex aims at one thing: make local Markdown folders pleasant to read.

Not editor-first.
Not graph-first.
Not cloud-first.

## Repo layout

- `docs/PRD.md` — product requirements
- `crates/localex-cli/` — Rust CLI + local API shell
- `web/` — reading UI prototype and tests

## Backend commands

```bash
cargo test -p localex-cli
cargo run -p localex-cli -- .
```

Available scaffold routes:
- `GET /`
- `GET /api/health`
- `GET /api/reader/defaults`

## Frontend commands

```bash
cd web
npm install
npm test
npm run build
npm run dev
```

## Current frontend prototype

Prototype already demonstrates:
- warm reading-first layout
- words-per-line control
- line-height control
- font-size control
- font-family control
- one/two-column toggle
- browser zoom shortcut override to text zoom
- focus overlay for image, code block, and table

## Next implementation steps

1. wire frontend bundle into Rust server
2. render real Markdown content instead of sample article
3. add internal link resolution and hover previews
4. add ref jump/back stack
5. add SQLite-backed highlights, notes, themes, and preferences in `~/.localex`
