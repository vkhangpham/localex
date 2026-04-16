import type { JSX } from 'preact';
import { useEffect, useMemo, useState } from 'preact/hooks';

type LayoutMode = 'one' | 'two';
type FontKey = 'inter' | 'charter' | 'plex';
type FocusBlock = 'image' | 'code' | 'table' | null;

type ReaderSettings = {
  targetWordsPerLine: number;
  lineHeight: number;
  fontSizePx: number;
  fontFamily: FontKey;
  layoutMode: LayoutMode;
};

const DEFAULT_SETTINGS: ReaderSettings = {
  targetWordsPerLine: 12,
  lineHeight: 1.75,
  fontSizePx: 18,
  fontFamily: 'inter',
  layoutMode: 'one',
};

const FONT_STACKS: Record<FontKey, string> = {
  inter: "Inter, system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif",
  charter: "Charter, 'Iowan Old Style', 'Palatino Linotype', 'Book Antiqua', Georgia, serif",
  plex: "'IBM Plex Sans', 'Inter', system-ui, sans-serif",
};

const FONT_LABELS: Record<FontKey, string> = {
  inter: 'Inter',
  charter: 'Charter',
  plex: 'IBM Plex Sans',
};

const SAMPLE_IMAGE = `data:image/svg+xml;utf8,${encodeURIComponent(`
<svg xmlns="http://www.w3.org/2000/svg" width="1280" height="720" viewBox="0 0 1280 720">
  <defs>
    <linearGradient id="bg" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" stop-color="#f8f2e6" />
      <stop offset="100%" stop-color="#ece2cf" />
    </linearGradient>
  </defs>
  <rect width="1280" height="720" fill="url(#bg)" rx="32" />
  <rect x="88" y="72" width="1104" height="576" rx="28" fill="#fffdf8" stroke="#d8cdb9" />
  <rect x="144" y="132" width="284" height="26" rx="13" fill="#d6cab5" />
  <rect x="144" y="184" width="812" height="34" rx="17" fill="#1b1712" opacity="0.92" />
  <rect x="144" y="246" width="944" height="18" rx="9" fill="#6b6256" opacity="0.82" />
  <rect x="144" y="282" width="910" height="18" rx="9" fill="#6b6256" opacity="0.72" />
  <rect x="144" y="318" width="874" height="18" rx="9" fill="#6b6256" opacity="0.64" />
  <rect x="144" y="388" width="992" height="168" rx="24" fill="#f2ecdf" stroke="#d8cdb9" />
  <rect x="188" y="432" width="232" height="20" rx="10" fill="#6f675b" opacity="0.78" />
  <rect x="188" y="470" width="488" height="18" rx="9" fill="#6f675b" opacity="0.58" />
  <rect x="188" y="506" width="444" height="18" rx="9" fill="#6f675b" opacity="0.48" />
  <circle cx="1036" cy="250" r="74" fill="#d8cdb9" />
  <circle cx="1036" cy="250" r="44" fill="#fff8ea" />
</svg>
`)}`;

function clamp(value: number, min: number, max: number) {
  return Math.min(max, Math.max(min, value));
}

function App() {
  const [settings, setSettings] = useState<ReaderSettings>(DEFAULT_SETTINGS);
  const [focusedBlock, setFocusedBlock] = useState<FocusBlock>(null);

  useEffect(() => {
    const onZoomShortcut = (event: KeyboardEvent) => {
      if (!(event.metaKey || event.ctrlKey)) {
        return;
      }

      const isZoomIn = event.key === '+' || event.key === '=';
      const isZoomOut = event.key === '-' || event.key === '_';
      const isReset = event.key === '0';

      if (!(isZoomIn || isZoomOut || isReset)) {
        return;
      }

      event.preventDefault();

      setSettings((current) => {
        if (isReset) {
          return { ...current, fontSizePx: DEFAULT_SETTINGS.fontSizePx };
        }

        const nextFontSize = clamp(current.fontSizePx + (isZoomIn ? 1 : -1), 14, 28);
        return { ...current, fontSizePx: nextFontSize };
      });
    };

    window.addEventListener('keydown', onZoomShortcut);
    return () => window.removeEventListener('keydown', onZoomShortcut);
  }, []);

  const readerStyle = useMemo(
    () =>
      ({
        '--reader-font-size': `${settings.fontSizePx}px`,
        '--reader-line-height': String(settings.lineHeight),
        '--reader-measure': `min(100%, calc(${settings.targetWordsPerLine} * 6.8ch + 5rem))`,
        '--reader-columns': settings.layoutMode === 'two' ? '2' : '1',
        '--reader-font-family': FONT_STACKS[settings.fontFamily],
      }) as JSX.CSSProperties,
    [settings],
  );

  const updateSetting = <K extends keyof ReaderSettings>(key: K, value: ReaderSettings[K]) => {
    setSettings((current) => ({ ...current, [key]: value }));
  };

  const openFocusedBlock = (block: Exclude<FocusBlock, null>) => setFocusedBlock(block);

  return (
    <>
      <div class="app-shell">
        <aside class="sidebar sidebar-left">
          <div class="sidebar-card">
            <p class="sidebar-label">Workspace</p>
            <h2>Markdown tree</h2>
            <ul class="nav-list">
              <li class="nav-item active">README.md</li>
              <li class="nav-item">Guides/reading-principles.md</li>
              <li class="nav-item">Design/notion-notes.md</li>
              <li class="nav-item">Backlinks</li>
            </ul>
          </div>
        </aside>

        <main class="main-pane">
          <div class="top-strip">
            <span class="eyebrow">Calm, local, reading-first.</span>
            <span class="hint">Ctrl/Cmd +/- overrides browser zoom and changes font size.</span>
          </div>

          <section class="reader-frame">
            <article class="reader-surface" style={readerStyle}>
              <div class="reader-content">
                <p class="eyebrow">Prototype reader surface</p>
                <h1>Localex reading shell</h1>
                <p class="lede">
                  Built for local Markdown folders. Wide margins, warm surfaces, precise typography, and quiet chrome.
                  Read first. Navigate gently. Annotate without turning page into dashboard.
                </p>
                <p>
                  Core controls should change text itself, not browser chrome. That means measure, line height, font
                  size, font family, and one/two-column layout all belong to reader surface. User tunes page until it
                  disappears.
                </p>
                <p>
                  Links, references, and hover previews will let readers move through connected notes without losing
                  place. Back stack must feel exact. Footnotes must jump cleanly and return to exact origin.
                </p>

                <div
                  aria-label="Focus image example"
                  class="focusable-block"
                  onClick={() => openFocusedBlock('image')}
                  onKeyDown={activateWithKeyboard(() => openFocusedBlock('image'))}
                  role="button"
                  tabIndex={0}
                >
                  <img alt="Localex reading illustration" src={SAMPLE_IMAGE} />
                  <span class="focus-hint">Click image to focus</span>
                </div>

                <p>
                  Rich blocks should expand into their own stage. Images, tables, and code deserve temporary focus
                  mode: centered, enlarged, background blurred, one click out to return to flow.
                </p>

                <div
                  aria-label="Focus code example"
                  class="focusable-block"
                  onClick={() => openFocusedBlock('code')}
                  onKeyDown={activateWithKeyboard(() => openFocusedBlock('code'))}
                  role="button"
                  tabIndex={0}
                >
                  <pre>
                    <code>{`// zoom reader text, not browser chrome\nfunction zoomReader(delta) {\n  setFontSize((size) => clamp(size + delta, 14, 28));\n}`}</code>
                  </pre>
                  <span class="focus-hint">Click code to focus</span>
                </div>

                <div
                  aria-label="Focus table example"
                  class="focusable-block"
                  onClick={() => openFocusedBlock('table')}
                  onKeyDown={activateWithKeyboard(() => openFocusedBlock('table'))}
                  role="button"
                  tabIndex={0}
                >
                  <table>
                    <thead>
                      <tr>
                        <th>Setting</th>
                        <th>Default</th>
                        <th>Intent</th>
                      </tr>
                    </thead>
                    <tbody>
                      <tr>
                        <td>Words/line</td>
                        <td>12</td>
                        <td>Narrow, calmer reading measure</td>
                      </tr>
                      <tr>
                        <td>Line height</td>
                        <td>1.75</td>
                        <td>Lower fatigue on long sessions</td>
                      </tr>
                      <tr>
                        <td>Columns</td>
                        <td>1</td>
                        <td>Switchable to 2 for scan-heavy docs</td>
                      </tr>
                    </tbody>
                  </table>
                  <span class="focus-hint">Click table to focus</span>
                </div>

                <blockquote class="callout">
                  Localex should feel closer to reading beautiful notes than managing files in IDE.
                </blockquote>
              </div>
            </article>
          </section>
        </main>

        <aside class="sidebar sidebar-right">
          <div class="sidebar-card controls-card">
            <p class="sidebar-label">Reader controls</p>
            <h2>Personalize reading</h2>

            <label class="control-group">
              <span>Target words per line</span>
              <input
                aria-label="Target words per line"
                max={18}
                min={7}
                onInput={(event) =>
                  updateSetting('targetWordsPerLine', Number((event.currentTarget as HTMLInputElement).value))
                }
                type="range"
                value={settings.targetWordsPerLine}
              />
              <strong>{settings.targetWordsPerLine} words</strong>
            </label>

            <label class="control-group">
              <span>Line height</span>
              <input
                aria-label="Line height"
                max={2}
                min={1.4}
                onInput={(event) => updateSetting('lineHeight', Number((event.currentTarget as HTMLInputElement).value))}
                step={0.05}
                type="range"
                value={settings.lineHeight}
              />
              <strong>Line height: {settings.lineHeight.toFixed(2)}</strong>
            </label>

            <label class="control-group">
              <span>Font family</span>
              <select
                aria-label="Font family"
                onChange={(event) => updateSetting('fontFamily', (event.currentTarget as HTMLSelectElement).value as FontKey)}
                value={settings.fontFamily}
              >
                {Object.entries(FONT_LABELS).map(([key, label]) => (
                  <option key={key} value={key}>
                    {label}
                  </option>
                ))}
              </select>
            </label>

            <div class="control-group">
              <span>Columns</span>
              <div class="toggle-row">
                <button
                  aria-pressed={settings.layoutMode === 'one'}
                  onClick={() => updateSetting('layoutMode', 'one')}
                  type="button"
                >
                  1 column
                </button>
                <button
                  aria-pressed={settings.layoutMode === 'two'}
                  onClick={() => updateSetting('layoutMode', 'two')}
                  type="button"
                >
                  2 columns
                </button>
              </div>
            </div>

            <div class="control-group">
              <span>Zoom text</span>
              <div class="toggle-row">
                <button onClick={() => updateSetting('fontSizePx', clamp(settings.fontSizePx - 1, 14, 28))} type="button">
                  A-
                </button>
                <button onClick={() => updateSetting('fontSizePx', DEFAULT_SETTINGS.fontSizePx)} type="button">
                  Reset
                </button>
                <button onClick={() => updateSetting('fontSizePx', clamp(settings.fontSizePx + 1, 14, 28))} type="button">
                  A+
                </button>
              </div>
              <strong>Font size: {settings.fontSizePx}px</strong>
            </div>
          </div>
        </aside>
      </div>

      {focusedBlock ? (
        <div aria-label="Focused block preview" aria-modal="true" class="focus-overlay" onClick={() => setFocusedBlock(null)} role="dialog">
          <div class="focus-card" onClick={(event) => event.stopPropagation()}>
            <button class="focus-close" onClick={() => setFocusedBlock(null)} type="button">
              Close
            </button>
            {renderFocusedBlock(focusedBlock)}
          </div>
        </div>
      ) : null}
    </>
  );
}

function activateWithKeyboard(action: () => void) {
  return (event: JSX.TargetedKeyboardEvent<HTMLDivElement>) => {
    if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault();
      action();
    }
  };
}

function renderFocusedBlock(block: Exclude<FocusBlock, null>) {
  if (block === 'image') {
    return (
      <figure class="overlay-block overlay-image">
        <img alt="Zoomed Localex reading illustration" src={SAMPLE_IMAGE} />
        <figcaption>Focused image block</figcaption>
      </figure>
    );
  }

  if (block === 'code') {
    return (
      <div class="overlay-block">
        <h3>Focused code block</h3>
        <pre>
          <code>{`const handleZoomShortcut = (event) => {\n  if (!(event.metaKey || event.ctrlKey)) return;\n  if (['+', '=', '-', '_', '0'].includes(event.key)) {\n    event.preventDefault();\n    updateReaderFont(event.key);\n  }\n};`}</code>
        </pre>
      </div>
    );
  }

  return (
    <div class="overlay-block">
      <h3>Focused table block</h3>
      <table>
        <thead>
          <tr>
            <th>Control</th>
            <th>Behavior</th>
          </tr>
        </thead>
        <tbody>
          <tr>
            <td>Image focus</td>
            <td>Center block, blur background, click out to return.</td>
          </tr>
          <tr>
            <td>Code focus</td>
            <td>Expand width and preserve scroll context.</td>
          </tr>
          <tr>
            <td>Table focus</td>
            <td>Make dense data readable without page zoom.</td>
          </tr>
        </tbody>
      </table>
    </div>
  );
}

export default App;
