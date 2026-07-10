// src/tests/roundtrip.test.ts - the frontmatter round-trip test ADR-001
// requires. The 200-note fixture lives in `fixtures/notes-200/` and is committed
// under `docs/fixtures/bundle.md`. This test only verifies the round-trip
// helper; the Rust crate-level test owns the byte-level invariants.

import test from 'node:test';
import { strict as assert } from 'node:assert';

import { joinFrontmatter, parseFrontmatter } from '../lib/roundtrip.js';

const sample = `---
okf_version: v0.1
type: note
title: Sample
---

# Hello

body
`;

void test('parseFrontmatter keeps raw yaml and body intact', () => {
  const pair = parseFrontmatter(sample);
  assert.match(pair.raw, /title: Sample/);
  assert.match(pair.body, /# Hello/);
});

void test('joinFrontmatter reassembles a parsed pair byte-for-byte', () => {
  const pair = parseFrontmatter(sample);
  const reassembled = joinFrontmatter(pair);
  assert.equal(reassembled, sample);
});

void test('parseFrontmatter handles missing frontmatter gracefully', () => {
  const pair = parseFrontmatter('plain body');
  assert.equal(pair.raw, '');
  assert.equal(pair.body, 'plain body');
});
