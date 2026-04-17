import type { JSX } from 'preact';
import { useCallback, useEffect, useRef, useState } from 'preact/hooks';
import type { Highlight } from '../lib/types';
import * as api from '../lib/api';

type Props = {
  currentPath: string;
  docHtml: string;
  highlights: Highlight[];
  onHighlightsChange: (h: Highlight[]) => void;
};

const HIGHLIGHT_COLORS = ['yellow', 'green', 'blue', 'pink', 'orange'];

export default function HighlightLayer({ currentPath, docHtml, highlights, onHighlightsChange }: Props): JSX.Element {
  const containerRef = useRef<HTMLDivElement>(null);
  const [toolbar, setToolbar] = useState<{ x: number; y: number; text: string } | null>(null);

  // Render existing highlights on doc load
  useEffect(() => {
    if (!containerRef.current || highlights.length === 0) return;
    const container = containerRef.current;
    highlights.forEach((h) => {
      wrapHighlight(container, h);
    });
  }, [docHtml, highlights]);

  // Detect text selection
  const onMouseUp = useCallback(() => {
    const sel = window.getSelection();
    if (!sel || sel.isCollapsed || !sel.rangeCount) {
      return;
    }
    const text = sel.toString().trim();
    if (!text || text.length < 3) return;

    // Only trigger if selection is within our container
    const range = sel.getRangeAt(0);
    if (!containerRef.current?.contains(range.commonAncestorContainer)) return;

    const rect = range.getBoundingClientRect();
    setToolbar({
      x: rect.left + rect.width / 2,
      y: rect.top - 8,
      text,
    });
  }, []);

  const dismissToolbar = useCallback(() => {
    setToolbar(null);
    window.getSelection()?.removeAllRanges();
  }, []);

  const saveHighlight = useCallback(async (color: string) => {
    if (!toolbar || !currentPath) return;
    try {
      const h = await api.createHighlight({
        document_path: currentPath,
        quote_text: toolbar.text,
        color,
      });
      onHighlightsChange([...highlights, h]);
      // Immediately wrap the current selection
      if (containerRef.current) {
        wrapHighlight(containerRef.current, h);
      }
    } catch (e) {
      console.error('Failed to save highlight:', e);
    }
    dismissToolbar();
  }, [toolbar, currentPath, highlights, onHighlightsChange, dismissToolbar]);

  return (
    <>
      <div
        class="reader-doc"
        dangerouslySetInnerHTML={{ __html: docHtml }}
        onMouseUp={onMouseUp}
        ref={containerRef}
      />
      {toolbar && (
        <div class="highlight-toolbar" style={{ left: `${toolbar.x}px`, top: `${toolbar.y}px` }}>
          {HIGHLIGHT_COLORS.map((color) => (
            <button
              key={color}
              class={`highlight-color-btn highlight-color-${color}`}
              onClick={() => saveHighlight(color)}
              type="button"
              aria-label={`Highlight ${color}`}
            />
          ))}
          <button class="highlight-dismiss-btn" onClick={dismissToolbar} type="button">x</button>
        </div>
      )}
    </>
  );
}

function wrapHighlight(container: HTMLElement, h: Highlight) {
  // Find text nodes containing the quote
  const walker = document.createTreeWalker(container, NodeFilter.SHOW_TEXT);
  const textNodes: Text[] = [];
  let node: Text | null;
  while ((node = walker.nextNode() as Text | null)) {
    const idx = node.textContent?.indexOf(h.quote_text);
    if (idx !== undefined && idx !== -1) {
      textNodes.push(node);
    }
  }

  // Wrap first match
  for (const textNode of textNodes) {
    const text = textNode.textContent || '';
    const idx = text.indexOf(h.quote_text);
    if (idx === -1) continue;

    // Don't re-wrap if already inside a mark
    if (textNode.parentElement?.closest(`mark[data-highlight-id="${h.id}"]`)) continue;

    const range = document.createRange();
    range.setStart(textNode, idx);
    range.setEnd(textNode, idx + h.quote_text.length);

    const mark = document.createElement('mark');
    mark.setAttribute('data-highlight-id', String(h.id));
    mark.className = `highlight-mark highlight-${h.color}`;
    range.surroundContents(mark);
    break; // Only wrap first match
  }
}
