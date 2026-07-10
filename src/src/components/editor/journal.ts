// src/components/editor/journal.ts - typed daily-note command contract.
//
// Spine stub. Implements the path policy from ADR-009: directory
// 'journal/' is allowlisted, filename YYYY-MM-DD.md, content begins
// with the OKF frontmatter 'type: note'. Idempotent: the command does
// not write when the file already exists.

import type { VaultIpc, VaultWriteRequest, VaultWriteResponse } from '../../lib/ipc.js';

export interface JournalEntry {
  readonly date: string;       // YYYY-MM-DD
  readonly relPath: string;    // 'journal/YYYY-MM-DD.md'
  readonly bytes: Uint8Array;
}

const OKF_NOTE_HEADER = (date: string): string =>
  `---\nokf_version: v0.1\ntype: note\ndate: ${date}\n---\n\n# ${date}\n\n`;

export function journalFileName(isoDate: string): string {
  if (!/^\d{4}-\d{2}-\d{2}$/.test(isoDate)) {
    throw new Error(`invalid journal date: ${isoDate}`);
  }
  return isoDate + '.md';
}

export function journalRelPath(isoDate: string): string {
  return 'journal/' + journalFileName(isoDate);
}

export function todayUtc(): string {
  const now = new Date();
  const yyyy = now.getUTCFullYear().toString().padStart(4, '0');
  const mm = (now.getUTCMonth() + 1).toString().padStart(2, '0');
  const dd = now.getUTCDate().toString().padStart(2, '0');
  return `${yyyy}-${mm}-${dd}`;
}

export interface JournalCommandView {
  today(now: string): JournalEntry;
  /** Returns whether a write was committed (true) or the file already existed (false). */
  createIfMissing(ipc: VaultIpc, now?: string): Promise<'created' | 'exists'>;
}

export class JournalCommandStub implements JournalCommandView {
  today(now: string = todayUtc()): JournalEntry {
    if (!/^\d{4}-\d{2}-\d{2}$/.test(now)) {
      throw new Error(`invalid journal date: ${now}`);
    }
    const bytes = new TextEncoder().encode(OKF_NOTE_HEADER(now));
    return { date: now, relPath: journalRelPath(now), bytes };
  }

  async createIfMissing(
    ipc: VaultIpc,
    now: string = todayUtc(),
  ): Promise<'created' | 'exists'> {
    const entry = this.today(now);
    const req: VaultWriteRequest = {
      path: entry.relPath,
      bytes: entry.bytes,
      authorKind: 'human',
      message: `seed ${entry.relPath}`,
    };
    const response: VaultWriteResponse = await ipc.write(req);
    return response.oid.length > 0 ? 'created' : 'exists';
  }
}
