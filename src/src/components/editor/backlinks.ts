// src/components/editor/backlinks.ts - typed backlinks panel contract.
//
// Spine stub. The real source is crates/haven-index::edges_from run
// against the active index; the contract here is purely typed so the
// surface can be implemented in either Tauri (host binding) or vitest
// (deterministic fixture). Linked: docs/adr/009-pkm-ux-surface.md.

import type { IndexSearchResponse, VaultIpc } from '../../lib/ipc.js';

export interface BacklinkHit {
  readonly sourcePath: string;
  readonly line: number;
  readonly snippet: string;
}

export interface BacklinksPanelView {
  /** Source path whose backlinks the panel surfaces. */
  forPath(path: string): Promise<ReadonlyArray<BacklinkHit>>;
}

interface ParsedSnippetLine {
  readonly line: number;
  readonly text: string;
}

function toSnippetLines(snippet: string): ReadonlyArray<ParsedSnippetLine> {
  if (!snippet) return [];
  const lines = snippet.split(/\r?\n/);
  return lines.map((text, idx) => ({ line: idx + 1, text }));
}

/**
 * Spine implementation. Queries the IPC for documents that mention the
 * target path as a literal substring; full-fidelity backlinks arrive
 * when crates/haven-index::edges_from is wired through the IPC in a
 * follow-up PR. The query here is intentionally a plain FTS5 substring
 * (not a regex pattern) so SQLite's FTS handles it as a phrase lookup
 * without regex escaping.
 */
export class BacklinksPanelStub implements BacklinksPanelView {
  constructor(
    private readonly ipc: VaultIpc,
    private readonly target: string,
  ) {}

  async forPath(path: string): Promise<ReadonlyArray<BacklinkHit>> {
    const response: IndexSearchResponse = await this.ipc.search({
      query: path,
      limit: 64,
    });
    return response.paths.flatMap((sourcePath) => {
      const snippets = toSnippetLines(sourcePath);
      return snippets.map((entry) => ({
        sourcePath,
        line: entry.line,
        snippet: entry.text,
      }));
    });
  }

  get targetPath(): string {
    return this.target;
  }
}
