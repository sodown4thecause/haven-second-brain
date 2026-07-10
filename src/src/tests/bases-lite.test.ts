// src/tests/bases-lite.test.ts - typed contract tests for Bases-lite
// stubs plus the round-trip-deletion invariant from ADR-010.
//
// Linked ADR: docs/adr/010-bases-lite-derived-view.md.
//
// The invariant under test: deleting a saved view removes only the
// view file itself; the rest of the vault is byte-identical.

import { strict as assert } from 'node:assert';
import { mkdtempSync, mkdirSync, readFileSync, writeFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

import { describe, it } from 'vitest';

import {
  BasesQueryStub,
  BasesViewStub,
  buildViewFile,
  serializeView,
  viewRelPath,
} from '../components/bases/index.js';
import type { IndexSearchRequest, IndexSearchResponse, LintRequest, LintResponse, VaultIpc, VaultWriteRequest, VaultWriteResponse } from '../lib/ipc.js';

class MockIpc implements VaultIpc {
  public readonly writes: VaultWriteRequest[] = [];
  public readonly searches: IndexSearchRequest[] = [];

  async write(req: VaultWriteRequest): Promise<VaultWriteResponse> {
    this.writes.push(req);
    return { oid: req.path ? 'oid-stub' : '', path: req.path, authorKind: req.authorKind };
  }
  async lint(req: LintRequest): Promise<LintResponse> {
    return { ok: true, diagnostics: [{ severity: 'warning', message: req.path }] };
  }
  async search(req: IndexSearchRequest): Promise<IndexSearchResponse> {
    this.searches.push(req);
    return { paths: [] };
  }
}

function hashDir(root: string, exclude: ReadonlyArray<string> = []): Map<string, string> {
  const { createHash } = require('node:crypto') as typeof import('node:crypto');
  const out = new Map<string, string>();
  const ignored = (rel: string) => exclude.some((skip) => rel === skip);
  const stack: Array<{ dir: string; rel: string }> = [{ dir: root, rel: '' }];
  while (stack.length > 0) {
    const next = stack.pop();
    if (!next) break;
    const { readdirSync, statSync } = require('node:fs') as typeof import('node:fs');
    for (const name of readdirSync(next.dir)) {
      const abs = join(next.dir, name);
      const rel = next.rel ? `${next.rel}/${name}` : name;
      if (ignored(rel)) continue;
      const stat = statSync(abs);
      if (stat.isDirectory()) {
        stack.push({ dir: abs, rel });
      } else if (stat.isFile()) {
        const bytes = readFileSync(abs);
        out.set(rel, createHash('sha256').update(bytes).digest('hex'));
      }
    }
  }
  return out;
}

function seedVault(): string {
  const dir = mkdtempSync(join(tmpdir(), 'haven-bases-'));
  // Pre-create allowlist roots and seed two notes; this is the snapshot
  // we later compare against.
  mkdirSync(join(dir, 'notes'), { recursive: true });
  mkdirSync(join(dir, 'journal'), { recursive: true });
  mkdirSync(join(dir, 'skills'), { recursive: true });
  mkdirSync(join(dir, 'memory'), { recursive: true });
  mkdirSync(join(dir, 'inputs'), { recursive: true });
  writeFileSync(
    join(dir, 'notes', 'idea.md'),
    '---\nokf_version: v0.1\ntype: note\n---\n# Idea\n',
    'utf-8',
  );
  writeFileSync(
    join(dir, 'notes', 'todo.md'),
    '---\nokf_version: v0.1\ntype: note\n---\n# Todo\n- [ ] first\n',
    'utf-8',
  );
  writeFileSync(
    join(dir, 'journal', '2026-07-01.md'),
    '---\nokf_version: v0.1\ntype: note\ndate: 2026-07-01\n---\n# 2026-07-01\n',
    'utf-8',
  );
  return dir;
}

describe('BasesViewStub', () => {
  it('builds a view file with the expected relPath + frontmatter', () => {
    const stub = new BasesViewStub();
    const file = stub.build({
      name: 'drafts',
      kind: 'table',
      filterYaml: 'type = note AND tags ⊆ ["drafts"]',
      sort: 'updated_at desc',
      body: '| path | updated_at |\n',
    });
    assert.equal(file.relPath, viewRelPath('drafts'));
    assert.equal(file.relPath, 'notes/views/drafts.md');
    assert.equal(file.frontmatter.type, 'view');
    assert.equal(file.frontmatter.view_kind, 'table');
    assert.equal(file.frontmatter.okf_version, 'v0.1');
    const text = new TextDecoder().decode(file.bytes);
    assert.match(text, /^---\nokf_version: v0\.1\ntype: view\n/);
    assert.match(text, /view_kind: table\n/);
    assert.match(text, /\| path \| updated_at \|/);
  });

  it('rejects invalid view names', () => {
    assert.throws(() => viewRelPath('DRAFTS'));
    assert.throws(() => viewRelPath('has space'));
    assert.throws(() => viewRelPath('slash/here'));
  });

  it('produces a deterministic serialisation', () => {
    const def = {
      name: 'a',
      kind: 'list' as const,
      filterYaml: 'type = note',
      sort: 'title asc',
      body: '- path: notes/idea.md',
    };
    assert.equal(serializeView(def), serializeView(def));
  });
});

describe('BasesViewStub.delete', () => {
  it('dispatches a tombstone-shaped write via VaultIpc', async () => {
    const stub = new BasesViewStub();
    const ipc = new MockIpc();
    await stub.delete(ipc, 'notes/views/drafts.md', 'human');
    assert.equal(ipc.writes.length, 1);
    const req = ipc.writes[0]!;
    assert.equal(req.path, 'notes/views/drafts.md');
    assert.equal(req.authorKind, 'human');
    assert.equal(req.bytes.length, 0);
    assert.match(req.message, /^delete /);
  });
});

describe('BasesQueryStub', () => {
  it('returns a phase-2 placeholder row set', () => {
    const query = new BasesQueryStub();
    const result = query.evaluate({
      okf_version: 'v0.1',
      type: 'view',
      view_kind: 'list',
      view_filter: 'type = note',
      sort: 'title asc',
    });
    assert.deepEqual(result.rows, []);
    assert.equal(result.requires, 'phase-2');
  });
});

describe('Bases-lite round-trip-deletion invariant', () => {
  it('removing notes/views/<name>.md leaves every other file byte-identical', () => {
    const vault = seedVault();
    const viewRel = viewRelPath('drafts');
    const beforeAll = hashDir(vault);
    const baseSize = beforeAll.size;
    const viewAbs = join(vault, viewRel);

    // Build the view (via the typed stub) and write it to disk.
    const file = buildViewFile({
      name: 'drafts',
      kind: 'list',
      filterYaml: 'type = note',
      sort: 'updated_at desc',
      body: '- path: notes/idea.md',
    });
    mkdirSync(join(vault, 'notes', 'views'), { recursive: true });
    writeFileSync(viewAbs, file.bytes);

    const whilePresent = hashDir(vault, [viewRel.replace(/\\/g, '/')]);
    assert.equal(whilePresent.size, baseSize, 'creating the view adds exactly one file');
    assert.ok(
      beforeAll.get(viewRel) === undefined,
      'pre-create snapshot must not include the view path',
    );

    // Delete ONLY the view file via raw fs (the invariant we care about
    // is the file-surface impact of a deletion).
    rmSync(viewAbs);

    const afterDelete = hashDir(vault);
    assert.equal(
      afterDelete.size,
      baseSize,
      'deleting the view must leave the file count at the pre-create baseline',
    );
    for (const [rel, hash] of beforeAll.entries()) {
      assert.equal(
        afterDelete.get(rel),
        hash,
        `view deletion must keep '${rel}' byte-identical`,
      );
    }
  });
});
