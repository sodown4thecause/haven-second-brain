// src/components/bases/query.ts - typed Bases-lite query contract.
//
// Spine stub. Real evaluator is phase-2 work; for now the stub returns
// a deterministic empty projection and marks itself requires(Phase 2)
// so callers can tell the contract is real even though the engine is a
// placeholder.

import type { ViewFrontmatter } from './view.js';

export interface QueryRow {
  readonly path: string;
  readonly matchedFields: Readonly<Record<string, string>>;
}

export interface QueryResult {
  readonly rows: ReadonlyArray<QueryRow>;
  readonly requires: 'phase-2';
}

export interface BasesQuery {
  evaluate(fm: ViewFrontmatter): QueryResult;
}

export class BasesQueryStub implements BasesQuery {
  evaluate(fm: ViewFrontmatter): QueryResult {
    if (!fm) {
      return { rows: [], requires: 'phase-2' };
    }
    return { rows: [], requires: 'phase-2' };
  }
}
