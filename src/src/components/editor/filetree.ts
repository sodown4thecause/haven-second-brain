// src/components/editor/filetree.ts - typed file-tree contract.
//
// Spine stub. Reads canonical allowlist roots from the same constants
// haven-git uses (crates/haven-git/src/lib.rs OwnedAllowlist::default_vault)
// so the file tree UX cannot drift from the writer's view. No filesystem
// reads here; the real implementation lives in the Tauri follow-up PR
// (ADR-008). Until then the stub returns the canonical roots and a
// marker that says "stub" so callers can tell.

export type FileTreeRootKind = 'notes' | 'journal' | 'memory' | 'skills' | 'inputs';

export interface FileTreeRoot {
  readonly kind: FileTreeRootKind;
  readonly relPath: string;
}

export const FILE_TREE_ROOTS: ReadonlyArray<FileTreeRoot> = [
  { kind: 'notes',   relPath: 'notes' },
  { kind: 'journal', relPath: 'journal' },
  { kind: 'memory',  relPath: 'memory' },
  { kind: 'skills',  relPath: 'skills' },
  { kind: 'inputs',  relPath: 'inputs' },
];

export interface FileTreeView {
  roots(): ReadonlyArray<FileTreeRoot>;
  /** Returns the canonical relPath or null if the file is not under any root. */
  locate(absPath: string): string | null;
}

export class FileTreeStub implements FileTreeView {
  roots(): ReadonlyArray<FileTreeRoot> {
    return FILE_TREE_ROOTS;
  }
  locate(absPath: string): string | null {
    const normalized = absPath.replace(/\\/g, '/');
    for (const root of FILE_TREE_ROOTS) {
      if (normalized.endsWith('/' + root.relPath) || normalized === root.relPath) {
        return root.relPath + '/';
      }
      const prefix = root.relPath + '/';
      if (normalized.includes('/' + prefix)) {
        const idx = normalized.indexOf('/' + prefix);
        return normalized.slice(idx + 1);
      }
    }
    return null;
  }
}
