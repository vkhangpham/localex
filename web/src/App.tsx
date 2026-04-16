import type { JSX } from 'preact';
import { useCallback, useEffect, useMemo, useRef, useState } from 'preact/hooks';
import BacklinksPanel from './components/BacklinksPanel';
import HighlightLayer from './components/HighlightLayer';
import NotePanel from './components/NotePanel';
import type { Highlight, Note } from './lib/types';
import * as api from './lib/api';
import { renderMath } from './lib/katex-render';

// ── Types ──

type LayoutMode = 'one' | 'two';
type FontKey = 'inter' | 'charter' | 'plex';

type ReaderSettings = {
  targetWordsPerLine: number;
  lineHeight: number;
  fontSizePx: number;
  fontFamily: FontKey;
  layoutMode: LayoutMode;
};

type FileEntry = {
  path: string;
  name: string;
  is_dir: boolean;
  children: FileEntry[];
};

type Heading = {
  level: number;
  id: string;
  text: string;
};

type RenderedDoc = {
  html: string;
  headings: Heading[];
};

type LinkPreview = {
  html: string;
  x: number;
  y: number;
};

type JumpEntry = {
  path: string;
  scrollY: number;
};

type Backlink = {
  source_path: string;
  link_text: string;
  excerpt: string;
};

// ── Constants ──

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

function clamp(value: number, min: number, max: number) {
  return Math.min(max, Math.max(min, value));
}

function isInternalLink(href: string): boolean {
  if (href.startsWith('#')) return true;
  if (href.endsWith('.md') || href.endsWith('.markdown')) return true;
  return false;
}

// ── App ──

export default function App() {
  const [settings, setSettings] = useState<ReaderSettings>(DEFAULT_SETTINGS);
  const [fileTree, setFileTree] = useState<FileEntry[]>([]);
  const [currentPath, setCurrentPath] = useState<string | null>(null);
  const [doc, setDoc] = useState<RenderedDoc | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [focusedHtml, setFocusedHtml] = useState<string | null>(null);
  const [jumpStack, setJumpStack] = useState<JumpEntry[]>([]);
  const [theme, setTheme] = useState<string>('light');
  const [backlinks, setBacklinks] = useState<Backlink[]>([]);
  const [highlights, setHighlights] = useState<Highlight[]>([]);
  const [notes, setNotes] = useState<Note[]>([]);
  const [preview, setPreview] = useState<LinkPreview | null>(null);
  const hoverTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const currentPathRef = useRef<string | null>(null);
  const pendingScrollRef = useRef<number | null>(null);

  // Fetch file tree on mount
  useEffect(() => {
    fetch('/api/files')
      .then((r) => r.json())
      .then(setFileTree)
      .catch((e) => setError(e.message));
  }, []);

  // Live reload via SSE with reconnection
  useEffect(() => {
    let es: EventSource | null = null;
    let reconnectTimer: ReturnType<typeof setTimeout> | null = null;
    let lastRefresh = 0;

    function connect() {
      es = new EventSource('/api/events');

      es.onmessage = (e) => {
        const now = Date.now();
        if (now - lastRefresh < 300) return;
        lastRefresh = now;

        try {
          const data = JSON.parse(e.data);
          const changedPaths: string[] = data.paths || [];

          if (data.kind === 'create' || data.kind === 'remove') {
            fetch('/api/files').then((r) => r.json()).then(setFileTree).catch(() => {});
          }

          // Handle deleted current file
          if (data.kind === 'remove' && currentPathRef.current && changedPaths.includes(currentPathRef.current)) {
            setDoc(null);
            setCurrentPath(null);
            setError('File was deleted');
          }

          if (data.kind === 'modify' && currentPathRef.current) {
            if (changedPaths.some((p) => p === currentPathRef.current)) {
              fetch(`/api/render?path=${encodeURIComponent(currentPathRef.current)}`)
                .then((r) => r.json())
                .then((d: RenderedDoc) => { if (currentPathRef.current === changedPaths[0]) setDoc(d); })
                .catch(() => {});
              fetch(`/api/backlinks?path=${encodeURIComponent(currentPathRef.current)}`)
                .then((r) => r.json())
                .then((d) => setBacklinks(d.backlinks || []))
                .catch(() => {});
            }
          }
        } catch { /* ignore parse errors */ }
      };

      es.onerror = () => {
        es?.close();
        es = null;
        reconnectTimer = setTimeout(connect, 3000);
      };
    }

    connect();

    return () => {
      es?.close();
      if (reconnectTimer) clearTimeout(reconnectTimer);
    };
  }, []);

  // Load saved theme on mount
  useEffect(() => {
    fetch('/api/preferences/theme')
      .then((r) => r.json())
      .then((data) => {
        if (data.value) {
          applyTheme(data.value);
          setTheme(data.value);
        }
      })
      .catch(() => {});
  }, []);

  const applyTheme = (name: string) => {
    document.documentElement.setAttribute('data-theme', name);
  };

  const changeTheme = (name: string) => {
    applyTheme(name);
    setTheme(name);
    fetch('/api/preferences', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ key: 'theme', value: name }),
    }).catch(() => {});
  };

  // Fetch rendered document + backlinks when path changes
  useEffect(() => {
    if (!currentPath) return;
    currentPathRef.current = currentPath;
    setLoading(true);
    setError(null);

    const controller = new AbortController();
    const signal = controller.signal;

    fetch(`/api/render?path=${encodeURIComponent(currentPath)}`, { signal })
      .then((r) => {
        if (!r.ok) throw new Error(`HTTP ${r.status}`);
        return r.json();
      })
      .then((data: RenderedDoc) => {
        if (!signal.aborted) {
          setDoc(data);
          setLoading(false);
        }
      })
      .catch((e) => {
        if (!signal.aborted) {
          setError(e.message);
          setLoading(false);
        }
      });
    fetch(`/api/backlinks?path=${encodeURIComponent(currentPath)}`, { signal })
      .then((r) => r.json())
      .then((data) => { if (!signal.aborted) setBacklinks(data.backlinks || []); })
      .catch(() => { if (!signal.aborted) setBacklinks([]); });
    api.fetchHighlights(currentPath).then((h) => { if (!signal.aborted) setHighlights(h); }).catch(() => { if (!signal.aborted) setHighlights([]); });
    api.fetchNotes(currentPath).then((n) => { if (!signal.aborted) setNotes(n); }).catch(() => { if (!signal.aborted) setNotes([]); });

    return () => controller.abort();
  }, [currentPath]);

  // Scroll restoration after cross-file navigation
  useEffect(() => {
    if (doc && pendingScrollRef.current !== null) {
      const y = pendingScrollRef.current;
      pendingScrollRef.current = null;
      // Use requestAnimationFrame to wait for DOM render
      requestAnimationFrame(() => {
        window.scrollTo({ top: y, behavior: 'instant' as ScrollBehavior });
      });
    }
  }, [doc]);

  // Render math (KaTeX) after document HTML updates
  useEffect(() => {
    if (!doc) return;
    const reader = document.querySelector('.reader-rendered');
    if (reader) renderMath(reader as HTMLElement);
  }, [doc]);

  // Auto-select first file
  useEffect(() => {
    if (fileTree.length > 0 && !currentPath) {
      const first = findFirstFile(fileTree);
      if (first) setCurrentPath(first);
    }
  }, [fileTree, currentPath]);

  // Zoom shortcut override
  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if (!(event.metaKey || event.ctrlKey)) return;
      const isIn = event.key === '+' || event.key === '=';
      const isOut = event.key === '-' || event.key === '_';
      const isReset = event.key === '0';
      if (!(isIn || isOut || isReset)) return;
      event.preventDefault();
      setSettings((cur) => ({
        ...cur,
        fontSizePx: isReset
          ? DEFAULT_SETTINGS.fontSizePx
          : clamp(cur.fontSizePx + (isIn ? 1 : -1), 14, 28),
      }));
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, []);

  // Focus mode: click images/tables/code/pre inside rendered content
  // Footnote jump-back: intercept footnote-ref and footnote-backref clicks
  // Internal link navigation: cross-file .md links
  useEffect(() => {
    const reader = document.querySelector('.reader-rendered');
    if (!reader) return;

    const onClick = (e: Event) => {
      const target = e.target as HTMLElement;
      const anchor = target.closest('a');
      if (!anchor) {
        // Non-link click: check for focusable block
        const block = target.closest('img, pre, table');
        if (!block) return;
        e.preventDefault();
        setFocusedHtml((block as HTMLElement).outerHTML);
        return;
      }

      // Footnote reference click (in-text → footnote)
      const fnRef = anchor.closest('.footnote-ref');
      if (fnRef) {
        e.preventDefault();
        const href = anchor.getAttribute('href');
        if (!href) return;
        setJumpStack((stack) => [...stack, { path: currentPathRef.current || '', scrollY: window.scrollY }]);
        const targetEl = document.querySelector(href);
        targetEl?.scrollIntoView({ behavior: 'smooth' });
        return;
      }

      // Footnote back-reference click (footnote → in-text)
      const fnBackRef = anchor.closest('.footnote-backref');
      if (fnBackRef) {
        e.preventDefault();
        setJumpStack((stack) => {
          if (stack.length === 0) return stack;
          const prev = stack[stack.length - 1];
          if (prev.path === currentPathRef.current) {
            window.scrollTo({ top: prev.scrollY, behavior: 'smooth' });
          }
          return stack.slice(0, -1);
        });
        return;
      }

      // Internal cross-file .md link navigation
      const href = anchor.getAttribute('href');
      if (href && isInternalLink(href) && !href.startsWith('#')) {
        e.preventDefault();
        setJumpStack((stack) => [...stack, { path: currentPathRef.current || '', scrollY: window.scrollY }]);
        setCurrentPath(href);
        return;
      }

      // Regular link: check for focusable block parent
      const block = target.closest('img, pre, table');
      if (block) {
        e.preventDefault();
        setFocusedHtml((block as HTMLElement).outerHTML);
      }
    };

    reader.addEventListener('click', onClick);
    return () => reader.removeEventListener('click', onClick);
  }, [doc]);

  // Back button for jump stack (footnotes + cross-file nav)
  const goBack = useCallback(() => {
    setJumpStack((stack) => {
      if (stack.length === 0) return stack;
      const prev = stack[stack.length - 1];
      if (prev.path !== currentPathRef.current) {
        pendingScrollRef.current = prev.scrollY;
        setCurrentPath(prev.path);
      } else {
        window.scrollTo({ top: prev.scrollY, behavior: 'smooth' });
      }
      return stack.slice(0, -1);
    });
  }, []);

  // Hover preview for internal links
  useEffect(() => {
    const reader = document.querySelector('.reader-rendered');
    if (!reader) return;

    const extractAnchorPreview = (href: string): string => {
      const id = href.slice(1);
      const el = document.getElementById(id);
      if (!el) return '';
      const tag = el.tagName.toLowerCase();
      let html = el.outerHTML;
      const next = el.nextElementSibling;
      if (next && (next.tagName === 'P' || next.tagName === 'BLOCKQUOTE')) {
        html += next.outerHTML;
      }
      return html;
    };

    const fetchFilePreview = async (href: string): Promise<string> => {
      try {
        const res = await fetch(`/api/render?path=${encodeURIComponent(href)}`);
        if (!res.ok) return '';
        const data: RenderedDoc = await res.json();
        const tmp = document.createElement('div');
        tmp.innerHTML = data.html;
        const h1 = tmp.querySelector('h1');
        const firstP = tmp.querySelector('p');
        let html = '';
        if (h1) html += h1.outerHTML;
        if (firstP) html += firstP.outerHTML;
        return html || '';
      } catch {
        return '';
      }
    };

    const onMouseEnter = (e: Event) => {
      const target = e.target as HTMLElement;
      const anchor = target.closest('a');
      if (!anchor) return;
      const href = anchor.getAttribute('href');
      if (!href || !isInternalLink(href)) return;

      const rect = anchor.getBoundingClientRect();
      const x = rect.left;
      const y = rect.bottom + 8;

      hoverTimer.current = setTimeout(() => {
        if (href.startsWith('#')) {
          const html = extractAnchorPreview(href);
          if (html) setPreview({ html, x, y });
        } else {
          fetchFilePreview(href).then((html) => {
            if (html) setPreview({ html, x, y });
          });
        }
      }, 300);
    };

    const onMouseLeave = () => {
      if (hoverTimer.current) {
        clearTimeout(hoverTimer.current);
        hoverTimer.current = null;
      }
      setPreview(null);
    };

    reader.addEventListener('mouseover', onMouseEnter);
    reader.addEventListener('mouseout', onMouseLeave);
    return () => {
      reader.removeEventListener('mouseover', onMouseEnter);
      reader.removeEventListener('mouseout', onMouseLeave);
      if (hoverTimer.current) clearTimeout(hoverTimer.current);
    };
  }, [doc]);

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
    setSettings((cur) => ({ ...cur, [key]: value }));
  };

  return (
    <>
      <div class="app-shell">
        <aside class="sidebar sidebar-left">
          <div class="sidebar-card">
            <p class="sidebar-label">Workspace</p>
            <h2>Files</h2>
            <ul class="nav-list">
              {fileTree.map((entry) => (
                <FileTreeItem
                  entry={entry}
                  depth={0}
                  currentPath={currentPath}
                  onSelect={setCurrentPath}
                />
              ))}
            </ul>
            <BacklinksPanel backlinks={backlinks} onSelect={setCurrentPath} />
          </div>
        </aside>

        <main class="main-pane">
          <div class="top-strip">
            <span class="eyebrow">Calm, local, reading-first.</span>
            <span class="hint">Ctrl/Cmd +/- text zoom. Click images/tables/code to focus.</span>
          </div>

          <section class="reader-frame">
            <article class="reader-surface" style={readerStyle}>
              <div class="reader-content reader-rendered">
                {loading && <p>Loading...</p>}
                {error && <p class="error-text">{error}</p>}
                {!loading && !error && !doc && (
                  <p>Select a file from the sidebar to start reading.</p>
                )}
                {doc && !loading && (
                  <HighlightLayer
                    currentPath={currentPath || ''}
                    docHtml={doc.html}
                    highlights={highlights}
                    onHighlightsChange={setHighlights}
                  />
                )}
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
                onInput={(e) => updateSetting('targetWordsPerLine', Number((e.currentTarget as HTMLInputElement).value))}
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
                onInput={(e) => updateSetting('lineHeight', Number((e.currentTarget as HTMLInputElement).value))}
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
                onChange={(e) => updateSetting('fontFamily', (e.currentTarget as HTMLSelectElement).value as FontKey)}
                value={settings.fontFamily}
              >
                {Object.entries(FONT_LABELS).map(([key, label]) => (
                  <option key={key} value={key}>{label}</option>
                ))}
              </select>
            </label>

            <div class="control-group">
              <span>Theme</span>
              <div class="toggle-row">
                <button aria-pressed={theme === 'light'} onClick={() => changeTheme('light')} type="button">Light</button>
                <button aria-pressed={theme === 'dark'} onClick={() => changeTheme('dark')} type="button">Dark</button>
                <button aria-pressed={theme === 'sepia'} onClick={() => changeTheme('sepia')} type="button">Sepia</button>
              </div>
            </div>

            <div class="control-group">
              <span>Columns</span>
              <div class="toggle-row">
                <button aria-pressed={settings.layoutMode === 'one'} onClick={() => updateSetting('layoutMode', 'one')} type="button">1 column</button>
                <button aria-pressed={settings.layoutMode === 'two'} onClick={() => updateSetting('layoutMode', 'two')} type="button">2 columns</button>
              </div>
            </div>

            <div class="control-group">
              <span>Zoom text</span>
              <div class="toggle-row">
                <button onClick={() => updateSetting('fontSizePx', clamp(settings.fontSizePx - 1, 14, 28))} type="button">A-</button>
                <button onClick={() => updateSetting('fontSizePx', DEFAULT_SETTINGS.fontSizePx)} type="button">Reset</button>
                <button onClick={() => updateSetting('fontSizePx', clamp(settings.fontSizePx + 1, 14, 28))} type="button">A+</button>
              </div>
              <strong>Font size: {settings.fontSizePx}px</strong>
            </div>

            {doc && doc.headings.length > 0 && (
              <div class="control-group toc-group">
                <span>Table of contents</span>
                <ul class="toc-list">
                  {doc.headings.map((h) => (
                    <li class={`toc-item toc-h${h.level}`}>
                      <a href={`#${h.id}`} onClick={(e) => { e.preventDefault(); document.getElementById(h.id)?.scrollIntoView({ behavior: 'smooth' }); }}>
                        {h.text}
                      </a>
                    </li>
                  ))}
                </ul>
              </div>
            )}

            <NotePanel
              notes={notes}
              highlights={highlights}
              currentPath={currentPath || ''}
              onNotesChange={setNotes}
            />
          </div>
        </aside>
      </div>

      {focusedHtml && (
        <div aria-modal="true" class="focus-overlay" onClick={() => setFocusedHtml(null)} role="dialog">
          <div class="focus-card" onClick={(e) => e.stopPropagation()}>
            <button class="focus-close" onClick={() => setFocusedHtml(null)} type="button">Close</button>
            <div class="overlay-block" dangerouslySetInnerHTML={{ __html: focusedHtml }} />
          </div>
        </div>
      )}

      {jumpStack.length > 0 && (
        <button class="jump-back-btn" onClick={goBack} type="button">
          ← Back
        </button>
      )}

      {preview && (
        <div
          class="hover-preview"
          style={{ left: `${preview.x}px`, top: `${preview.y}px` }}
          dangerouslySetInnerHTML={{ __html: preview.html }}
        />
      )}
    </>
  );
}

// ── File tree ──

function FileTreeItem({ entry, depth, currentPath, onSelect }: {
  entry: FileEntry;
  depth: number;
  currentPath: string | null;
  onSelect: (path: string) => void;
}) {
  const [expanded, setExpanded] = useState(depth < 1);

  if (entry.is_dir) {
    return (
      <li>
        <button
          class="nav-item dir-item"
          onClick={() => setExpanded(!expanded)}
          style={{ paddingLeft: `${14 + depth * 12}px` }}
          type="button"
        >
          {expanded ? '▾' : '▸'} {entry.name}
        </button>
        {expanded && entry.children.length > 0 && (
          <ul class="nav-list">
            {entry.children.map((child) => (
              <FileTreeItem entry={child} depth={depth + 1} currentPath={currentPath} onSelect={onSelect} />
            ))}
          </ul>
        )}
      </li>
    );
  }

  return (
    <li>
      <button
        class={`nav-item${currentPath === entry.path ? ' active' : ''}`}
        onClick={() => onSelect(entry.path)}
        style={{ paddingLeft: `${14 + depth * 12}px` }}
        type="button"
      >
        {entry.name}
      </button>
    </li>
  );
}

function findFirstFile(entries: FileEntry[]): string | null {
  for (const entry of entries) {
    if (!entry.is_dir) return entry.path;
    const found = findFirstFile(entry.children);
    if (found) return found;
  }
  return null;
}
