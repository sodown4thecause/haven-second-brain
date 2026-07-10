// src/lib/editor.ts - lossless Markdown editor shell contract for Haven.
//
// ADR-002 (CodeMirror 6 default) says the editor is a decorative layer over
// canonical Markdown text. So this contract deliberately pushes all
// preserve-and-render work onto the caller; the editor only knows about an
// optional syntax highlight and a raw view.

export interface EditorShell {
  /** Render a document without owning it. Returns a stable handle. */
  mount(domain: EditorDomain): EditorHandle;
}

export interface EditorDomain {
  readonly path: string;
  readonly text: string;
  /** Replace the document text. Caller is responsible for round-trip. */
  readonly onChange: (next: string) => void;
}

export interface EditorHandle {
  /** Returns the current canonical text the editor exposes for save. */
  snapshot(): string;
  /** Tear down any listeners. */
  dispose(): void;
}

export const EDITOR_CAPABILITIES = {
  preserves: 'frontmatter',
  surfacesRawMarkdown: true,
  hasGraphicalSchema: false,
} as const;
