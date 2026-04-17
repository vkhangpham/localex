import { fireEvent, render, screen } from '@testing-library/preact';
import { beforeEach, afterEach, vi } from 'vitest';

import App from './App';

const fetchMock = vi.fn();

const fileTree = [
  {
    path: 'notes',
    name: 'notes',
    is_dir: true,
    children: [
      {
        path: 'notes/example.md',
        name: 'example.md',
        is_dir: false,
        children: [],
      },
    ],
  },
];

const renderedDoc = {
  html: '<h1>Example doc</h1><pre aria-label="focus code example"><code>const answer = 42;</code></pre>',
  headings: [{ level: 1, id: 'example-doc', text: 'Example doc' }],
};

function mockJson(data: unknown) {
  return Promise.resolve({
    ok: true,
    json: async () => data,
  });
}

beforeEach(() => {
  fetchMock.mockImplementation((input: string | URL) => {
    const url = String(input);

    if (url === '/api/files') return mockJson(fileTree);
    if (url === '/api/preferences/theme') return mockJson({});
    if (url.startsWith('/api/render?path=')) return mockJson(renderedDoc);
    if (url.startsWith('/api/backlinks?path=')) return mockJson({ backlinks: [] });
    if (url.startsWith('/api/highlights?path=')) return mockJson({ highlights: [] });
    if (url.startsWith('/api/notes?path=')) return mockJson({ notes: [] });
    if (url === '/api/preferences') return mockJson({ ok: true });

    throw new Error(`Unhandled fetch: ${url}`);
  });

  vi.stubGlobal('fetch', fetchMock);
});

afterEach(() => {
  fetchMock.mockReset();
});

describe('Localex reading shell', () => {
  it('renders current reader chrome and controls', async () => {
    render(<App />);

    expect(await screen.findByRole('button', { name: /example\.md/i })).toBeInTheDocument();
    expect(screen.getByRole('heading', { name: /files/i })).toBeInTheDocument();
    expect(screen.getByRole('heading', { name: /personalize reading/i })).toBeInTheDocument();
    expect(screen.getByLabelText(/target words per line/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/line height/i)).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /2 columns/i })).toHaveAttribute(
      'aria-pressed',
      'false',
    );
  });

  it('intercepts zoom shortcuts and changes font size', () => {
    render(<App />);

    expect(screen.getByText(/font size: 18px/i)).toBeInTheDocument();
    fireEvent.keyDown(window, { key: '+', ctrlKey: true });
    expect(screen.getByText(/font size: 19px/i)).toBeInTheDocument();
    fireEvent.keyDown(window, { key: '0', ctrlKey: true });
    expect(screen.getByText(/font size: 18px/i)).toBeInTheDocument();
  });

  it('opens focused overlay for rich blocks from rendered docs', async () => {
    render(<App />);

    fireEvent.click(await screen.findByLabelText(/focus code example/i));
    expect(screen.getByRole('dialog')).toBeInTheDocument();
  });
});
