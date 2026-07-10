#!/usr/bin/env node
import { writeFileSync, mkdirSync } from 'node:fs';
import { resolve } from 'node:path';

const root = resolve(process.argv[2] || 'docs/fixtures/notes-200');
mkdirSync(root, { recursive: true });

const types = ['note', 'index', 'log'];
const titles = [
  'project plan', 'meeting notes', 'research note', 'decision',
  'reading list', 'tracking todo', 'question', 'review', 'journal',
  'recipe', 'quote', 'definition', 'reference', 'glossary',
  'experiment', 'hypothesis', 'observation', 'summary', 'analysis',
  'spec',
];

let total = 0;
for (let i = 0; i < 200; i++) {
  const t = types[i % types.length];
  const title = titles[i % titles.length];
  const date = new Date(2024, 0, 1 + (i % 28)).toISOString().slice(0, 10);
  const body = `# ${title} ${i}

Body line one for note ${i}.

Body line two with reference to [other](#note-${(i + 1) % 200}).`;
  const front =
    `---\n` +
    `okf_version: v0.1\n` +
    `type: ${t}\n` +
    `title: ${title} ${i}\n` +
    `timestamp: ${date}\n` +
    `tags:\n` +
    `  topic:\n` +
    `    - ${title.replace(/\s+/g, '-')}\n` +
    `---\n\n`;
  writeFileSync(`${root}/${String(i).padStart(3, '0')}-${title.replace(/\s+/g, '-')}.md`, front + body);
  total++;
}
console.log(`wrote ${total} okf fixtures under ${root}`);
