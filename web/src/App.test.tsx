import { fireEvent, render, screen } from '@testing-library/preact';

import App from './App';

describe('Localex reading shell', () => {
  it('renders core reader controls', () => {
    render(<App />);

    expect(
      screen.getByRole('heading', { name: /localex reading shell/i }),
    ).toBeInTheDocument();
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

  it('opens focused overlay for rich blocks', () => {
    render(<App />);

    fireEvent.click(screen.getByLabelText(/focus code example/i));
    expect(screen.getByRole('dialog', { name: /focused block preview/i })).toBeInTheDocument();
  });
});
