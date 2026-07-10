// src/lib/roundtrip.ts - a minimal Markdown round-trip helper used by tests.
//
// We do not depend on the editor here; the `parseFrontmatter` helper pulls
// off the YAML block the same way `crates/okf` does so tests can verify that
// the frontend and the Rust backend agree on the contract.

export interface FrontmatterPair {
  readonly raw: string;
  readonly body: string;
  readonly hadFrontmatter: boolean;
}

export function parseFrontmatter(input: string): FrontmatterPair {
  if (!input.startsWith('---')) {
    return { raw: '', body: input, hadFrontmatter: false };
  }
  // Strip the opening fence plus at most one CR/LF.
  let rest = input.slice(3);
  if (rest.startsWith('\r\n')) {
    rest = rest.slice(2);
  } else if (rest.startsWith('\n')) {
    rest = rest.slice(1);
  }
  // Find a closing `---` on its own line. A leading `---` without a closing
  // fence is a body horizontal rule — treat the entire input as body.
  // `fenceAt` is the index of the START of the closing fence line, so
  // `rest.slice(0, fenceAt)` returns exactly the YAML body — with the
  // line-terminator CRLF/LF preceding `---` excluded so `raw` is clean
  // YAML text.
  let fenceAt = -1;
  let cursor = 0;
  while (cursor < rest.length) {
    const nextEnd = lineEndAt(rest, cursor);
    const lineEnd = nextEnd - lineEndingLength(rest, nextEnd - 1);
    const line = rest.slice(cursor, lineEnd);
    if (line.replace(/\r$/, '').trim() === '---') {
      fenceAt = cursor;
      break;
    }
    cursor = nextEnd;
  }
  if (fenceAt === -1) {
    return { raw: '', body: input, hadFrontmatter: false };
  }
  // Trim ONE trailing CRLF/LF from the raw bytes; that is the line
  // terminator preceding the closing fence.
  const raw = rest
    .slice(0, fenceAt)
    .replace(/\r?\n$/, '');
  let afterFence = rest.slice(fenceAt);
  if (afterFence.startsWith('---\r\n')) {
    afterFence = afterFence.slice(5);
  } else if (afterFence.startsWith('---\n')) {
    afterFence = afterFence.slice(4);
  } else if (afterFence.startsWith('---')) {
    afterFence = afterFence.slice(3);
  }
  return { raw, body: afterFence, hadFrontmatter: true };
}

// Index of the position after the next newline (LF or CRLF) starting at
// or after `cursor`. Returns `rest.length` if no newline is found.
function lineEndAt(rest: string, cursor: number): number {
  const nl = rest.indexOf('\n', cursor);
  if (nl === -1) return rest.length;
  return nl + 1;
}

// Length of a line terminator ending with the newline at `cursor` (1 for LF,
// 2 for CRLF). Returns 0 if cursor is not on a newline character.
function lineEndingLength(rest: string, cursor: number): number {
  if (rest[cursor] !== '\n') return 0;
  if (cursor > 0 && rest[cursor - 1] === '\r') return 2;
  return 1;
}

export function joinFrontmatter(pair: FrontmatterPair): string {
  if (!pair.hadFrontmatter) {
    return pair.body;
  }
  // Cover the empty-but-present fence case: `---` + raw + `---` must be
  // emitted even when `raw` is empty. Body keeps any leading blank line
  // so the round-trip reconstructs the original document byte-for-byte.
  return `---\n${pair.raw}\n---\n${pair.body}`;
}
