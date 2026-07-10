// src/tests/roundtrip.test.ts - the frontmatter round-trip test ADR-001
// requires. The 200-note fixture lives in `docs/fixtures/bundle.md`. This test
// only verifies the round-trip helper; the Rust crate-level test owns the
// byte-level invariants.

import { strict as assert } from 'node:assert';

import { joinFrontmatter, parseFrontmatter } from '../lib/roundtrip.js';
import { describe, it } from 'vitest';

const sample = `---
okf_version: v0.1
type: note
title: Sample
---

# Hello

body
`;

describe('parseFrontmatter', () => {
  it('keeps raw yaml and body intact', () => {
    const pair = parseFrontmatter(sample);
    assert.match(pair.raw, /title: Sample/);
    assert.match(pair.body, /# Hello/);
    assert.equal(pair.hadFrontmatter, true);
  });

  it('handles missing frontmatter gracefully', () => {
    const pair = parseFrontmatter('plain body');
    assert.equal(pair.raw, '');
    assert.equal(pair.body, 'plain body');
    assert.equal(pair.hadFrontmatter, false);
  });

  it('treats a leading --- without a closing fence as body', () => {
    const pair = parseFrontmatter('---\nbody\n');
    assert.equal(pair.hadFrontmatter, false);
    assert.equal(pair.body, '---\nbody\n');
  });

  it('preserves an empty-but-present frontmatter fence', () => {
    const pair = parseFrontmatter('---\n---\nbody\n');
    assert.equal(pair.hadFrontmatter, true);
    assert.equal(pair.raw, '');
    assert.equal(pair.body, 'body\n');
  });
});

describe('joinFrontmatter', () => {
  it('reassembles a parsed pair byte-for-byte', () => {
    const pair = parseFrontmatter(sample);
    const reassembled = joinFrontmatter(pair);
    assert.equal(reassembled, sample);
  });

  it('preserves an empty-but-present fence on reassembly', () => {
    const input = '---\n---\nbody text';
    const pair = parseFrontmatter(input);
    const reassembled = joinFrontmatter(pair);
    assert.equal(reassembled, '---\n\n---\nbody text');
  });
});
