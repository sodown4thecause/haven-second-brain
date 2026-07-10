// src/components/bases/view.ts - typed Bases-lite saved-view contract.
//
// Spine stub. ADR-010 pins the deletion invariant: deleting a saved view
// removes only the view file; the rest of the vault is untouched.
//
// A view is a regular note under 'notes/views/' whose frontmatter
// declares view_kind, filter_yaml, and sort. The evaluator ships in a
// phase-2 PR; this stub defines the shape and the file-path policy so
// the round-trip-deletion test can assert the no-other-files-changed
// invariant today.

import type { AuthorKind, VaultIpc, VaultWriteRequest } from '../../lib/ipc.js';

export type ViewKind = 'table' | 'list';

export interface ViewDefinition {
  readonly name: string;            // filename-friendly, plain ASCII
  readonly kind: ViewKind;
  readonly filterYaml: string;       // raw YAML; parsed via crates/okf in phase 2
  readonly sort: string;
  readonly body: string;             // starter table or list body
}

export interface ViewFrontmatter {
  readonly okf_version: 'v0.1';
  readonly type: 'view';
  readonly view_kind: ViewKind;
  readonly view_filter: string;
  readonly sort: string;
}

export interface ViewFile {
  readonly relPath: string;          // 'notes/views/<name>.md'
  readonly bytes: Uint8Array;
  readonly frontmatter: ViewFrontmatter;
  readonly body: string;
}

export const VIEW_DIR = 'notes/views';

export function viewRelPath(name: string): string {
  if (!/^[a-z0-9][a-z0-9_.-]*$/.test(name)) {
    throw new Error(`invalid view name '${name}'; use lowercase letters, digits, '.', '_', '-'`);
  }
  return `${VIEW_DIR}/${name}.md`;
}

export function viewFrontmatter(def: ViewDefinition): ViewFrontmatter {
  return {
    okf_version: 'v0.1',
    type: 'view',
    view_kind: def.kind,
    view_filter: def.filterYaml,
    sort: def.sort,
  };
}

export function serializeView(def: ViewDefinition): string {
  const fm = viewFrontmatter(def);
  const yaml = [
    'okf_version: v0.1',
    'type: view',
    `view_kind: ${fm.view_kind}`,
    `view_filter: ${JSON.stringify(fm.view_filter)}`,
    `sort: ${JSON.stringify(fm.sort)}`,
  ].join('\n');
  const fence = `---\n${yaml}\n---\n`;
  return `${fence}\n${def.body}\n`;
}

export function buildViewFile(def: ViewDefinition): ViewFile {
  const relPath = viewRelPath(def.name);
  const text = serializeView(def);
  return {
    relPath,
    bytes: new TextEncoder().encode(text),
    frontmatter: viewFrontmatter(def),
    body: def.body,
  };
}

export interface BasesView {
  build(def: ViewDefinition): ViewFile;
  /** Delete the view file via VaultIpc.write with deletion semantics. */
  delete(ipc: VaultIpc, relPath: string, authorKind: AuthorKind): Promise<void>;
}

export class BasesViewStub implements BasesView {
  build(def: ViewDefinition): ViewFile {
    return buildViewFile(def);
  }

  async delete(ipc: VaultIpc, relPath: string, authorKind: AuthorKind): Promise<void> {
    // Spine: send an empty-byte write request marked as a tombstone. The
    // Tauri host implementation will translate this into a vaults.rm
    // call. Phase 2 ties the explicit delete IPC.
    const req: VaultWriteRequest = {
      path: relPath,
      bytes: new Uint8Array(),
      authorKind,
      message: `delete ${relPath}`,
    };
    await ipc.write(req);
  }
}
