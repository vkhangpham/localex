import '@testing-library/jest-dom/vitest';
import { vi } from 'vitest';

class MockEventSource {
  url: string;
  onmessage: ((event: MessageEvent) => void) | null = null;
  onerror: ((event: Event) => void) | null = null;

  constructor(url: string | URL) {
    this.url = String(url);
  }

  close() {}

  addEventListener() {}

  removeEventListener() {}
}

vi.stubGlobal('EventSource', MockEventSource);
