// scripts/okf-lint.mjs - the workspace-wide OKF linter wrapper.
//
// This script checks Markdown files under `notes/` against the OKF v0.1
// strict-write rules (`type`, `okf_version`). Violations break the build. We
// intentionally keep the parser simple here: heavy lifting lives in the
// Rust crate; this script is only the pre-commit sniff check.

import { readFileSync } from 'node:fs';
import { globSync } from 'node:fs';
import { resolve } from 'node:path';

const vaultRoot = process.argv[2] ?? resolve('notes');

const files = globSync(`${vaultRoot}/**/*.md`);

const errors = [];

for (const file of files) {
  const raw = readFileSync(file, 'utf8');
  if (!raw.startsWith('---')) {
    errors.push(`${file}: missing frontmatter`);
    continue;
  }
  const rest = raw.slice(3).replace(/^(\r?\n)/, '');
  const fenceIdx = rest.indexOf('\n---');
  if (fenceIdx === -1) {
    errors.push(`${file}: frontmatter opened but never closed`);
    continue;
  }
  const yaml = rest.slice(0, fenceIdx);
  if (!/^okf_version:\s*v0\.1\s*$/m.test(yaml)) {
    errors.push(`${file}: okf_version missing or not v0.1`);
  }
  if (!/^type:\s*\S/m.test(yaml)) {
    errors.push(`${file}: type missing or empty`);
  }
}

if (errors.length > 0) {
  for (const err of errors) {
    console.error(err);
  }
  process.exit(1);
}
console.log(`OKF lint clean: ${files.length} files`);
