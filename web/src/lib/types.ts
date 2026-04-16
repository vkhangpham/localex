export type Highlight = {
  id: number;
  document_path: string;
  quote_text: string;
  prefix_context: string;
  suffix_context: string;
  heading_slug: string;
  color: string;
  created_at: string;
  updated_at: string;
};

export type Note = {
  id: number;
  highlight_id: number | null;
  document_path: string;
  anchor_text: string;
  body: string;
  created_at: string;
  updated_at: string;
};

export type CreateHighlightRequest = {
  document_path: string;
  quote_text: string;
  prefix_context?: string;
  suffix_context?: string;
  heading_slug?: string;
  color?: string;
};

export type CreateNoteRequest = {
  highlight_id?: number | null;
  document_path: string;
  anchor_text?: string;
  body: string;
};
