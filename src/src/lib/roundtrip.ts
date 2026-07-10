// src/lib/roundtrip.ts - a minimal Markdown round-trip helper used by tests.
//
// We do not depend on the editor here; the `parseFrontmatter` helper pulls
// off the YAML block the same way `crates/okf` does so tests can verify that
// the frontend and the Rust backend agree on the contract.

export interface FrontmatterPair {
  readonly raw: string;
  readonly body: string;
}

export function parseFrontmatter(input: string): FrontmatterPair {
  if (!input.startsWith('---')) {
    return { raw: '', body: input };
  }
  const rest = input.slice(3).replace(/^(\r?\n)/, '');
  const fenceIdx = rest.indexOf('\n---');
  if (fenceIdx === -1) {
    return { raw: rest, body: '' };
  }
  const raw = rest.slice(0, fenceIdx);
  const body = rest.slice(fenceIdx + 4).replace(/^(\r?\n)/, '');
  return { raw, body };
}

export function joinFrontmatter(pair: FrontmatterPair): string {
  if (!pair.raw) {
    return pair.body;
  }
  return `---\n${pair.raw}\n---\n${pair.body}`;
}
