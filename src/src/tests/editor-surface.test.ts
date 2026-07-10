// src/tests/editor-surface.test.ts - typed contract tests for the four
// ADR-009 PKM-UX surface stubs. Verifies the seam surface against a
// mock VaultIpc; the eventual Tauri implementation will replace the
// mock without touching the stub code.
//
// Linked specs: docs/adr/009-pkm-ux-surface.md,
// docs/superpowers/specs/safe-existing-vault.md.

import { strict as assert } from 'node:assert';

import { describe, it } from 'vitest';

import {
  BacklinksPanelStub,
  FileTreeStub,
  JournalCommandStub,
  SearchPanelStub,
  journalRelPath,
} from '../components/editor/index.js';
import type {
  IndexSearchRequest,
  IndexSearchResponse,
  LintRequest,
  LintResponse,
  VaultIpc,
  VaultWriteRequest,
  VaultWriteResponse,
} from '../lib/ipc.js';

class MockIpc implements VaultIpc {
  public readonly writes: VaultWriteRequest[] = [];
  public readonly searches: IndexSearchRequest[] = [];
  private readonly writeResponse: VaultWriteResponse;
  private readonly searchAnswers: Map<string, ReadonlyArray<string>>;

  constructor(
    seedWrites: VaultWriteResponse[] = [{ oid: 'oid-stub', path: '', authorKind: 'human' }],
    searchAnswers: Map<string, ReadonlyArray<string>> = new Map(),
  ) {
    this.writeResponse = seedWrites[0] ?? { oid: '', path: '', authorKind: 'human' };
    this.searchAnswers = searchAnswers;
  }

  async write(req: VaultWriteRequest): Promise<VaultWriteResponse> {
    this.writes.push(req);
    return { ...this.writeResponse, path: req.path };
  }
  async lint(req: LintRequest): Promise<LintResponse> {
    return { ok: true, diagnostics: [{ severity: 'warning', message: req.path }] };
  }
  async search(req: IndexSearchRequest): Promise<IndexSearchResponse> {
    this.searches.push(req);
    const answer = this.searchAnswers.get(req.query) ?? [];
    return { paths: answer };
  }
}

describe('FileTreeStub', () => {
  it('returns the canonical allowlist roots', () => {
    const tree = new FileTreeStub();
    const roots = tree.roots();
    const kinds = roots.map((r) => r.kind).sort();
    assert.deepEqual(kinds, ['inputs', 'journal', 'memory', 'notes', 'skills']);
    for (const r of roots) {
      assert.match(r.relPath, /^[a-z]+$/);
    }
  });

  it('locates a journal path under the journal root', () => {
    const tree = new FileTreeStub();
    assert.equal(tree.locate('/vault/journal/2026-07-11.md'), 'journal/2026-07-11.md');
    assert.equal(tree.locate('/vault/notes/idea.md'), 'notes/idea.md');
  });

  it('returns null for paths outside any root', () => {
    const tree = new FileTreeStub();
    assert.equal(tree.locate('/vault/random/place.md'), null);
  });
});

describe('SearchPanelStub', () => {
  it('returns typed SearchHit records with descending scores', async () => {
    const ipc = new MockIpc(undefined, new Map([['okf', ['okf-note.md', 'legacy-note.md']]]));
    const panel = new SearchPanelStub(ipc);
    const hits = await panel.query('okf');
    assert.equal(hits.length, 2);
    assert.equal(hits[0]!.path, 'okf-note.md');
    assert.ok(hits[0]!.score > hits[1]!.score, 'first hit outranks second');
  });

  it('returns empty array for blank query', async () => {
    const ipc = new MockIpc();
    const panel = new SearchPanelStub(ipc);
    assert.deepEqual(await panel.query('   '), []);
    assert.equal(ipc.searches.length, 0, 'blank query must not hit the IPC');
  });
});

describe('BacklinksPanelStub', () => {
  it('queries the IPC with the target path as the literal substring', async () => {
    const ipc = new MockIpc();
    const panel = new BacklinksPanelStub(ipc, 'okf-note.md');
    await panel.forPath('okf-note.md');
    assert.equal(ipc.searches.length, 1);
    assert.equal(ipc.searches[0]!.query, 'okf-note.md');
  });

  it('returns empty array when ipc.search has no hits', async () => {
    const ipc = new MockIpc(undefined, new Map());
    const panel = new BacklinksPanelStub(ipc, 'nonexistent.md');
    assert.deepEqual(await panel.forPath('nonexistent.md'), []);
  });
});

describe('JournalCommandStub', () => {
  it('produces a journal/YYYY-MM-DD.md path with OKF header bytes', () => {
    const cmd = new JournalCommandStub();
    const entry = cmd.today('2026-07-11');
    assert.equal(entry.relPath, journalRelPath('2026-07-11'));
    assert.equal(entry.relPath, 'journal/2026-07-11.md');
    const text = new TextDecoder().decode(entry.bytes);
    assert.match(text, /---\nokf_version: v0\.1\ntype: note\n/);
    assert.match(text, /date: 2026-07-11/);
  });

  it('rejects malformed dates', () => {
    const cmd = new JournalCommandStub();
    assert.throws(() => cmd.today('07-11-2026'));
    assert.throws(() => cmd.today('not-a-date'));
  });

  it('createIfMissing writes via VaultIpc.write', async () => {
    const ipc = new MockIpc();
    const cmd = new JournalCommandStub();
    const outcome = await cmd.createIfMissing(ipc, '2026-07-11');
    assert.equal(outcome, 'created');
    assert.equal(ipc.writes.length, 1);
    const req = ipc.writes[0]!;
    assert.equal(req.path, 'journal/2026-07-11.md');
    assert.equal(req.authorKind, 'human');
    assert.match(req.message, /^seed /);
  });
});
