// src/components/editor/index.ts - barrel for editor surface stubs.
//
// Re-exports the typed contracts the rest of the app wires against, so a
// consumer can `import { FileTreeStub, JournalCommandStub } from
// './components/editor'`. The barrel is intentionally explicit; no
// `export *`.

export { FileTreeStub, FILE_TREE_ROOTS } from './filetree.js';
export type { FileTreeRoot, FileTreeRootKind, FileTreeView } from './filetree.js';
export { BacklinksPanelStub } from './backlinks.js';
export type { BacklinkHit, BacklinksPanelView } from './backlinks.js';
export { SearchPanelStub } from './search.js';
export type { SearchHit, SearchPanelView } from './search.js';
export {
  JournalCommandStub,
  journalFileName,
  journalRelPath,
  todayUtc,
} from './journal.js';
export type { JournalCommandView, JournalEntry } from './journal.js';
