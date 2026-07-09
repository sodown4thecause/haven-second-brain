import { createDocument, openBundle, readDocument, searchDocuments } from "./lib/ipc";

const app = document.getElementById("app")!;

app.innerHTML = `
  <header class="topbar">
    <div>
      <h1>Haven</h1>
      <p>Git-native second brain</p>
    </div>
    <button id="open-bundle">Open Bundle</button>
  </header>
  <main class="shell">
    <section class="pane editor-pane">
      <label>
        Bundle path
        <input id="bundle-path" placeholder="C:\\path\\to\\knowledge-bundle" />
      </label>
      <label>
        Document path
        <input id="doc-path" value="notes/untitled.md" />
      </label>
      <label>
        Title
        <input id="doc-title" value="Untitled" />
      </label>
      <label>
        Body
        <textarea id="doc-body" rows="16" placeholder="Write Markdown here..."></textarea>
      </label>
      <div class="actions">
        <button id="read-doc">Read</button>
        <button id="save-doc">Save</button>
      </div>
      <p id="status" role="status">Open a bundle to get started.</p>
    </section>
    <section class="pane search-pane">
      <label>
        Search
        <input id="search-query" placeholder="Search notes" />
      </label>
      <button id="search-docs">Search</button>
      <ol id="results"></ol>
    </section>
  </main>
`;

const bundlePath = document.getElementById("bundle-path") as HTMLInputElement;
const docPath = document.getElementById("doc-path") as HTMLInputElement;
const docTitle = document.getElementById("doc-title") as HTMLInputElement;
const docBody = document.getElementById("doc-body") as HTMLTextAreaElement;
const status = document.getElementById("status") as HTMLParagraphElement;
const searchQuery = document.getElementById("search-query") as HTMLInputElement;
const results = document.getElementById("results") as HTMLOListElement;

function setStatus(message: string): void {
  status.textContent = message;
}

function parseMarkdown(raw: string): { title: string; body: string } {
  const match = raw.match(/^---\n([\s\S]*?)\n---\n?\n?([\s\S]*)$/);
  if (!match) {
    return { title: docTitle.value, body: raw };
  }
  const frontmatter = match[1] ?? "";
  const title = frontmatter.match(/^title:\s*(.*)$/m)?.[1]?.replace(/^"|"$/g, "") ?? docTitle.value;
  return { title, body: match[2] ?? "" };
}

document.getElementById("open-bundle")?.addEventListener("click", async () => {
  try {
    const path = bundlePath.value.trim();
    if (!path) {
      setStatus("Enter a bundle path first.");
      return;
    }
    const result = await openBundle({ path });
    setStatus(`Bundle ${result.status}.`);
  } catch (error) {
    setStatus(`Open failed: ${String(error)}`);
  }
});

document.getElementById("read-doc")?.addEventListener("click", async () => {
  try {
    const result = await readDocument({ path: docPath.value.trim() });
    const parsed = parseMarkdown(result.raw);
    docTitle.value = parsed.title;
    docBody.value = parsed.body;
    setStatus(`Read ${result.path}.`);
  } catch (error) {
    setStatus(`Read failed: ${String(error)}`);
  }
});

document.getElementById("save-doc")?.addEventListener("click", async () => {
  try {
    const result = await createDocument({
      path: docPath.value.trim(),
      title: docTitle.value.trim() || "Untitled",
      content: docBody.value,
      doc_type: "note",
    });
    setStatus(`Saved ${result.path} at ${result.commit ?? "working tree"}.`);
  } catch (error) {
    setStatus(`Save failed: ${String(error)}`);
  }
});

document.getElementById("search-docs")?.addEventListener("click", async () => {
  try {
    const query = searchQuery.value.trim();
    if (!query) {
      results.replaceChildren();
      setStatus("Enter a search query first.");
      return;
    }
    const data = await searchDocuments({ query });
    results.replaceChildren(
      ...data.results.map((item) => {
        const row = document.createElement("li");
        row.innerHTML = `<strong></strong><p></p>`;
        row.querySelector("strong")!.textContent = `${item.title} (${item.path})`;
        row.querySelector("p")!.textContent = item.snippet;
        return row;
      }),
    );
    setStatus(`Found ${data.results.length} result(s).`);
  } catch (error) {
    setStatus(`Search failed: ${String(error)}`);
  }
});
