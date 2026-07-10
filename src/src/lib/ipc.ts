// src/lib/ipc.ts - typed IPC contract to the Rust vault writer.
//
// This is the single channel through which the frontend asks the writer
// thread to atomically commit, lint, or update the index. Every method has a
// request/response type pair so TypeScript catches missing-field mistakes.

import type { PathLike } from 'node:fs';

export type AuthorKind = 'human' | 'agent';

export interface VaultWriteRequest {
  readonly path: string;
  readonly bytes: Uint8Array;
  readonly authorKind: AuthorKind;
  readonly message: string;
  readonly expectedHash?: string;
}

export interface VaultWriteResponse {
  readonly oid: string;
  readonly path: string;
  readonly authorKind: AuthorKind;
}

export interface LintRequest {
  readonly path: string;
  readonly bytes: Uint8Array;
  readonly mode: 'strict-write' | 'permissive-read';
}

export interface LintResponse {
  readonly ok: boolean;
  readonly diagnostics: ReadonlyArray<LintDiagnostic>;
}

export interface LintDiagnostic {
  readonly severity: 'error' | 'warning';
  readonly message: string;
}

export interface IndexSearchRequest {
  readonly query: string;
  readonly limit: number;
}

export interface IndexSearchResponse {
  readonly paths: ReadonlyArray<string>;
}

export interface VaultIpc {
  write(req: VaultWriteRequest): Promise<VaultWriteResponse>;
  lint(req: LintRequest): Promise<LintResponse>;
  search(req: IndexSearchRequest): Promise<IndexSearchResponse>;
}

export interface FileTouchTarget extends PathLike {
  readonly touch?: never;
}
