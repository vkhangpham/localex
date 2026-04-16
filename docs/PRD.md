# Localex PRD

Status: draft v0.1

## 1. Product summary

Localex is a local-first CLI that opens any Markdown folder in a browser and turns it into a calm, beautiful reading environment.

It is not an editor, workspace suite, or dashboard. It is a reading shell for Markdown knowledge.

Core promise:
- zero cloud
- safe local serving
- beautiful Markdown rendering
- precise navigation across links and references
- personal reading controls that change text itself, not browser chrome

## 2. Problem

Most Markdown viewers fail in one of two ways:
- too raw: renderers are technically correct but visually harsh, cramped, or ugly
- too heavy: tools become mini-IDE or productivity suite and distract from reading

Readers need something else:
- open any folder instantly
- read with excellent typography and spacing
- control measure, line height, fonts, and layout
- inspect images/tables/code without breaking flow
- navigate references and internal links without losing place
- keep notes and highlights locally

## 3. Product vision

Localex should feel like:
- reading a beautifully typeset notebook
- browsing a personal local wiki
- moving through linked notes without friction

Localex should not feel like:
- VS Code preview
- Obsidian clone
- dashboard with widgets everywhere
- graph explorer

## 4. Product principles

1. Reading first
- all decisions optimize reading comfort before feature count

2. Quiet chrome
- navigation exists but stays visually secondary to document itself

3. Local and private
- no outbound network calls, no telemetry, no cloud sync by default

4. Text-level control
- zoom changes text size, not whole browser viewport
- reader can tune measure, line height, font, and columns

5. Navigation without disorientation
- links, references, footnotes, and backlinks should preserve context and exact return path

6. Rich blocks deserve focus mode
- images, tables, code blocks, and similar non-normal text blocks can expand into temporary spotlight mode

## 5. Target users

Primary:
- people who keep notes, docs, journals, or technical writing in local Markdown folders
- researchers, engineers, writers, students, knowledge workers

Secondary:
- anyone previewing README-heavy repos or documentation trees locally

## 6. Goals

### Primary goals
- open any Markdown directory via CLI and serve it locally
- provide a beautiful reading UI with excellent defaults
- support deep personalization of reading surface
- support clean navigation across Markdown links and references
- persist highlights, notes, themes, and reader preferences in `~/.localex`

### Non-goals
- real-time collaboration
- cloud sync
- graph visualization
- keyboard-first product positioning
- full Markdown editor
- plugin ecosystem in v1

## 7. Core user stories

1. As a reader, I can run `localex /path/to/docs` and instantly get a local reading view in browser.
2. As a reader, I can adjust words per line, line height, font size, font family, and one/two-column layout until text feels right for me.
3. As a reader, when I use browser zoom shortcuts, Localex changes text size instead of scaling the entire UI.
4. As a reader, I can click an image, table, or code block to focus it in a centered overlay, then click outside to return to the exact reading flow.
5. As a reader, I can hover internal links and preview destination content before deciding to jump.
6. As a reader, I can click a footnote or reference, jump to target, then return to exact origin.
7. As a reader, I can highlight text and attach notes, all stored locally.
8. As a reader, I can switch themes and create my own custom themes.
9. As a reader, I can inspect backlinks to understand where a note is referenced.

## 8. Functional requirements

### 8.1 Local serving and safety
- CLI command: `localex [directory]`
- default host: `127.0.0.1`
- default port: product-defined local port
- only serve files under chosen root
- sanitize raw HTML by default
- no telemetry or remote calls
- notes, highlights, preferences, and themes stored under `~/.localex`

Acceptance criteria:
- app works fully offline after install
- app binds to localhost by default
- app never serves files outside chosen root without explicit opt-in

### 8.2 Markdown rendering
Must render beautifully and correctly:
- CommonMark / GFM baseline
- headings and anchors
- fenced code blocks with syntax highlighting
- inline code
- images
- tables
- blockquotes and callouts
- math
- footnotes/references
- task lists

Acceptance criteria:
- common docs and note folders render without broken layout
- code blocks are readable in light and dark themes
- tables are readable inline and in focus mode

### 8.3 Reader controls
Reader controls are first-class product features, not theme footnotes.

Controls:
- target words per line
- line height
- font size
- font family
- one-column / two-column layout
- theme selection

Behavior:
- changes apply to text surface, not browser chrome
- controls update live without full page reload
- preferences persist locally
- reader can reset to defaults

Acceptance criteria:
- controls are visible but quiet
- controls feel instantaneous
- document remains readable while controls change live

### 8.4 Measure control: words per line
User specifically controls approximate number of words per line.

Product behavior:
- expose measure as a reading control, labeled in words per line
- implementation may approximate via character-based width heuristics under the hood
- product language remains user-facing and intuitive
- defaults target calm long-form reading

Acceptance criteria:
- changing measure clearly narrows or widens reading column
- line length never expands into exhausting full-width paragraphs

### 8.5 Zoom behavior override
Browser zoom shortcuts should adjust reader font size instead of zooming full page.

Product behavior:
- intercept `Cmd/Ctrl +`, `Cmd/Ctrl -`, and `Cmd/Ctrl 0`
- increase/decrease/reset text size only
- preserve layout chrome scale and interaction fidelity
- expose equivalent on-screen controls for discoverability

Acceptance criteria:
- browser page chrome does not zoom when Localex shortcut handling is active
- text size changes instantly
- reset returns to default or user baseline font size

### 8.6 One-column / two-column reading modes
Reader can switch document surface between one-column and two-column format.

Product behavior:
- one-column is default for long-form reading
- two-column is optional for scanning dense notes or reference material
- non-normal text blocks should avoid ugly splits across columns where possible

Acceptance criteria:
- toggle is instant
- prose remains readable in both modes
- headings and rich blocks do not fragment badly

### 8.7 Rich block focus mode
Any non-normal text block should support focus mode.

Definition of focusable blocks:
- images
- tables
- code blocks
- diagrams
- embedded rich media if supported later

Interaction:
- click block to open centered focused view
- blur or dim background
- enlarge selected block without distorting aspect ratio
- click outside or close control to return block to original position
- preserve document scroll position

Acceptance criteria:
- focus mode feels immediate and reversible
- rich block is significantly easier to inspect in focus mode
- exit returns reader to exact prior reading context

### 8.8 Link hover preview
- hover internal Markdown links, anchors, and references to preview destination
- preview includes title and excerpt or target block snippet
- preview should be fast and non-intrusive

Acceptance criteria:
- preview appears quickly
- preview reduces unnecessary navigation jumps

### 8.9 Reference and footnote navigation
- clicking footnote/reference jumps to target
- Localex stores origin position
- user can go back to exact source position

Acceptance criteria:
- back action restores both location and scroll context
- repeated ref hopping remains predictable

### 8.10 Highlights and notes
- select text and save highlight
- attach note to highlight or block anchor
- persist in `~/.localex`
- restore annotations when document reopens

Acceptance criteria:
- highlights survive restart
- notes remain attached even after small nearby edits when possible

### 8.11 Themes
Built-in themes:
- light
- dark
- sepia or paper-like

Custom themes:
- stored locally in `~/.localex/themes`
- based on CSS variables or equivalent token system

Acceptance criteria:
- theme switching is instant
- custom themes require no rebuild

### 8.12 Backlinks
- index internal Markdown links
- show which docs link to current doc
- backlinks panel stays lightweight and secondary

Acceptance criteria:
- backlinks are accurate for supported link syntax
- backlinks help context without overwhelming main reader

## 9. UX requirements

### Layout
Default desktop layout:
- left rail: doc tree / backlinks toggle
- center: reader surface
- right rail: TOC / notes / reader controls

Guidelines:
- central prose width should stay narrow and calm
- generous whitespace around content
- warm neutrals over cold dashboard grays
- typography should feel closer to high-quality docs than app chrome

### Typography
- default body size around 17–19px
- generous line height around 1.65–1.8
- bold, tight heading hierarchy
- dark text softened slightly instead of pure black
- strong image and code block framing

### Responsive behavior
- on narrow screens, side rails collapse into drawers or tabs
- reader stays primary
- controls remain reachable but not dominant

## 10. Technical direction

Recommended architecture:
- Rust backend / CLI for file scanning, link indexing, serving, caching, and local persistence
- lightweight web frontend for polished typography, layouts, hover previews, focus overlays, and live controls

Storage:
- SQLite in `~/.localex`

Why this approach:
- Rust gives speed, safety, and low memory for local file/index work
- web frontend gives best path to beautiful reading experience

## 11. Performance requirements
- fast cold start on normal note folders
- incremental refresh on file change
- hover previews should feel instant after initial index
- focus mode open/close should feel immediate
- control changes should update live with no visible lag

## 12. Data and persistence

Persist under `~/.localex`:
- highlights
- notes
- reader preferences
- themes
- workspace metadata
- navigation history if useful

Likely model:
- workspace table
- document table
- highlight table
- note table
- theme table
- reader preference table

## 13. MVP scope

MVP includes:
- local CLI launch
- beautiful Markdown rendering
- syntax highlighting
- TOC
- reader controls: words/line, line height, font size, font family, one/two columns
- zoom override for text size
- focus mode for image/table/code blocks
- internal link navigation
- hover preview
- ref/footnote jump + exact back
- highlights and notes
- built-in themes + custom theme loading
- backlinks

## 14. Out of scope for MVP
- graph view
- collaboration
- sync
- plugin API
- editing-first workflows

## 15. Success criteria

Localex is successful when:
- a user prefers reading their Markdown folder in Localex over generic preview tools
- reader controls feel meaningful, not cosmetic
- references and rich block inspection feel precise and pleasant
- local-first and privacy guarantees are easy to trust

## 16. Immediate implementation priorities

1. lock reader surface design quality first
2. implement core reader controls and persistence
3. implement rich block focus mode
4. implement exact reference jump/back behavior
5. implement hover previews and backlinks
6. add notes/highlights
7. finish theme customization
