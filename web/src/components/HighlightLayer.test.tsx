import { render } from '@testing-library/preact';

import HighlightLayer from './HighlightLayer';

describe('HighlightLayer', () => {
  it('renders markdown blocks directly in the selectable document container', () => {
    const { container } = render(
      <HighlightLayer
        currentPath="notes/example.md"
        docHtml="<h1>Reader title</h1><p>First paragraph.</p>"
        highlights={[]}
        onHighlightsChange={() => {}}
      />,
    );

    const docContainer = container.firstElementChild as HTMLElement | null;

    expect(docContainer).not.toBeNull();
    expect(Array.from(docContainer?.children ?? []).map((el) => el.tagName)).toEqual(['H1', 'P']);
  });

  it('wraps rendered tables in horizontal scroll containers', () => {
    const { container } = render(
      <HighlightLayer
        currentPath="notes/example.md"
        docHtml={'<table><thead><tr><th>Family</th><th>Planning</th></tr></thead><tbody><tr><td>Dreamer</td><td>Imagined rollouts</td></tr></tbody></table>'}
        highlights={[]}
        onHighlightsChange={() => {}}
      />,
    );

    const scrollWrap = container.querySelector('.table-scroll');
    expect(scrollWrap).toBeInTheDocument();
    expect(scrollWrap?.querySelector('table')).toBeInTheDocument();
  });
});
