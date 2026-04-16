import type { JSX } from 'preact';

export type Backlink = {
  source_path: string;
  link_text: string;
  excerpt: string;
};

type Props = {
  backlinks: Backlink[];
  onSelect: (path: string) => void;
};

export default function BacklinksPanel({ backlinks, onSelect }: Props): JSX.Element | null {
  if (backlinks.length === 0) return null;

  return (
    <div class="backlinks-panel">
      <p class="sidebar-label">Linked from</p>
      <ul class="backlink-list">
        {backlinks.map((bl) => (
          <li key={bl.source_path} class="backlink-item">
            <button
              class="backlink-link"
              onClick={() => onSelect(bl.source_path)}
              type="button"
            >
              <strong>{bl.link_text}</strong>
              <span class="backlink-source">{bl.source_path}</span>
            </button>
            {bl.excerpt && (
              <p class="backlink-excerpt">{bl.excerpt}</p>
            )}
          </li>
        ))}
      </ul>
    </div>
  );
}
