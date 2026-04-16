import type { JSX } from 'preact';
import { useCallback, useState } from 'preact/hooks';
import type { Note, Highlight } from '../lib/types';
import * as api from '../lib/api';

type Props = {
  notes: Note[];
  highlights: Highlight[];
  currentPath: string;
  onNotesChange: (n: Note[]) => void;
};

export default function NotePanel({ notes, highlights, currentPath, onNotesChange }: Props): JSX.Element | null {
  const [newNoteBody, setNewNoteBody] = useState('');

  const handleAddNote = useCallback(async () => {
    if (!newNoteBody.trim() || !currentPath) return;
    try {
      const note = await api.createNote({
        document_path: currentPath,
        body: newNoteBody.trim(),
      });
      onNotesChange([...notes, note]);
      setNewNoteBody('');
    } catch (e) {
      console.error('Failed to create note:', e);
    }
  }, [newNoteBody, currentPath, notes, onNotesChange]);

  const handleDeleteNote = useCallback(async (id: number) => {
    try {
      await api.deleteNote(id);
      onNotesChange(notes.filter((n) => n.id !== id));
    } catch (e) {
      console.error('Failed to delete note:', e);
    }
  }, [notes, onNotesChange]);

  if (notes.length === 0 && !currentPath) return null;

  return (
    <div class="note-panel">
      <p class="sidebar-label">Notes</p>
      <textarea
        class="note-input"
        placeholder="Add a note..."
        value={newNoteBody}
        onInput={(e) => setNewNoteBody((e.currentTarget as HTMLTextAreaElement).value)}
        rows={2}
      />
      <button
        class="note-add-btn"
        disabled={!newNoteBody.trim()}
        onClick={handleAddNote}
        type="button"
      >
        Save
      </button>
      <ul class="note-list">
        {notes.map((note) => {
          const linkedHighlight = note.highlight_id
            ? highlights.find((h) => h.id === note.highlight_id)
            : null;
          return (
            <li key={note.id} class="note-item">
              {linkedHighlight && (
                <blockquote class="note-quote">{linkedHighlight.quote_text}</blockquote>
              )}
              <p class="note-body">{note.body}</p>
              <button
                class="note-delete-btn"
                onClick={() => handleDeleteNote(note.id)}
                type="button"
                aria-label="Delete note"
              >
                Delete
              </button>
            </li>
          );
        })}
      </ul>
    </div>
  );
}
