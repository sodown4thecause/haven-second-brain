// src/components/bases/index.ts - barrel for Bases-lite stubs.

export {
  BasesViewStub,
  buildViewFile,
  serializeView,
  viewFrontmatter,
  viewRelPath,
  VIEW_DIR,
} from './view.js';
export type {
  BasesView,
  ViewDefinition,
  ViewFile,
  ViewFrontmatter,
  ViewKind,
} from './view.js';
export { BasesQueryStub } from './query.js';
export type { BasesQuery, QueryResult, QueryRow } from './query.js';
