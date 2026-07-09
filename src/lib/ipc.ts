/**
 * Typed IPC client for Tauri commands.
 *
 * Every `invoke()` call goes through a strongly-typed wrapper here —
 * no raw string command names scattered through components.
 */
import { invoke } from "@tauri-apps/api/core";

// -- Types -------------------------------------------------------------------

export interface OpenBundleArgs {
  path: string;
}

export interface OpenBundleResult {
  status: string;
  doc_count: number;
}

export interface CreateDocumentArgs {
  path: string;
  title: string;
  content: string;
  doc_type: string;
}

export interface CreateDocumentResult {
  path: string;
  commit: string | null;
}

export interface ReadDocumentArgs {
  path: string;
}

export interface ReadDocumentResult {
  path: string;
  raw: string;
}

export interface SearchArgs {
  query: string;
}

export interface SearchResultItem {
  path: string;
  title: string;
  snippet: string;
  score: number;
}

export interface SearchResultData {
  results: SearchResultItem[];
}

// -- Commands ----------------------------------------------------------------

export function openBundle(args: OpenBundleArgs): Promise<OpenBundleResult> {
  return invoke("open_bundle", { args });
}

export function createDocument(args: CreateDocumentArgs): Promise<CreateDocumentResult> {
  return invoke("create_document", { args });
}

export function readDocument(args: ReadDocumentArgs): Promise<ReadDocumentResult> {
  return invoke("read_document", { args });
}

export function searchDocuments(args: SearchArgs): Promise<SearchResultData> {
  return invoke("search_documents", { args });
}
