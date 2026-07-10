// src/components/editor/search.ts - typed global search panel contract.
//
// Spine stub. Wraps VaultIpc.search and re-projects the underlying
// paths-only response into typed SearchHit records. The IPC contract on
// disk (src/src/lib/ipc.ts:46-48) returns paths; this stub is the seam
// where Phase 2 adds snippets and scores.

import type { IndexSearchResponse, VaultIpc } from '../../lib/ipc.js';

export interface SearchHit {
  readonly path: string;
  readonly snippet: string;
  readonly score: number;
}

export interface SearchPanelView {
  query(q: string): Promise<ReadonlyArray<SearchHit>>;
}

export class SearchPanelStub implements SearchPanelView {
  constructor(private readonly ipc: VaultIpc) {}

  async query(q: string): Promise<ReadonlyArray<SearchHit>> {
    if (!q.trim()) return [];
    const response: IndexSearchResponse = await this.ipc.search({ query: q, limit: 32 });
    return response.paths.map((path, index) => ({
      path,
      snippet: '',
      score: Number((1 / (index + 1)).toFixed(3)),
    }));
  }
}
