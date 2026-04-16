import type { Highlight, Note, CreateHighlightRequest, CreateNoteRequest } from './types';

export async function fetchHighlights(path: string): Promise<Highlight[]> {
  const r = await fetch(`/api/highlights?path=${encodeURIComponent(path)}`);
  const data = await r.json();
  return data.highlights || [];
}

export async function createHighlight(req: CreateHighlightRequest): Promise<Highlight> {
  const r = await fetch('/api/highlights', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(req),
  });
  return r.json();
}

export async function deleteHighlight(id: number): Promise<void> {
  await fetch(`/api/highlights/${id}`, { method: 'DELETE' });
}

export async function fetchNotes(path: string): Promise<Note[]> {
  const r = await fetch(`/api/notes?path=${encodeURIComponent(path)}`);
  const data = await r.json();
  return data.notes || [];
}

export async function createNote(req: CreateNoteRequest): Promise<Note> {
  const r = await fetch('/api/notes', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(req),
  });
  return r.json();
}

export async function deleteNote(id: number): Promise<void> {
  await fetch(`/api/notes/${id}`, { method: 'DELETE' });
}
