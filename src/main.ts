import { invoke } from "@tauri-apps/api/core";
import "./style.css";

type Snippet = {
  trigger: string;
  replace: string;
};

type GitState = {
  dirty: boolean;
  branch: string;
  ahead: number;
};

const app = document.querySelector<HTMLDivElement>("#app")!;

app.innerHTML = `
  <main class="shell">
    <header class="hero">
      <div>
        <div class="eyebrow">ESPANSO MANAGER</div>
        <h1>Snippetlerini hızlı ve temiz yönet.</h1>
        <p>Espanso <code>base.yml</code> dosyanı doğrudan yönetir. Ayrı veritabanı yok.</p>
      </div>
      <div class="hero-actions">
        <button id="git-save" class="button ghost">Git durumu…</button>
        <button id="add" class="button primary">+ Yeni snippet</button>
      </div>
    </header>

    <section class="toolbar card">
      <input id="search" class="search" placeholder="Trigger veya içerikte ara…" autocomplete="off" />
      <div id="count" class="count">0 snippet</div>
    </section>

    <section id="list" class="list"></section>

    <dialog id="editor">
      <form id="form" method="dialog" class="editor-card">
        <div class="editor-head">
          <div>
            <div class="eyebrow">SNIPPET</div>
            <h2 id="editor-title">Yeni snippet</h2>
          </div>
          <button type="button" id="close-editor" class="icon-button" aria-label="Kapat">×</button>
        </div>

        <label>
          <span>Trigger</span>
          <input id="trigger" placeholder=":ornek" required />
        </label>

        <label>
          <span>Çıktı</span>
          <textarea id="replace" rows="7" placeholder="Yazılacak metin" required></textarea>
        </label>

        <div class="editor-actions">
          <button type="button" id="cancel" class="button ghost">Vazgeç</button>
          <button type="submit" class="button primary">Kaydet</button>
        </div>
      </form>
    </dialog>

    <div id="toast" class="toast" role="status"></div>
  </main>
`;

const list = document.querySelector<HTMLDivElement>("#list")!;
const search = document.querySelector<HTMLInputElement>("#search")!;
const count = document.querySelector<HTMLDivElement>("#count")!;
const addButton = document.querySelector<HTMLButtonElement>("#add")!;
const gitButton = document.querySelector<HTMLButtonElement>("#git-save")!;
const dialog = document.querySelector<HTMLDialogElement>("#editor")!;
const form = document.querySelector<HTMLFormElement>("#form")!;
const triggerInput = document.querySelector<HTMLInputElement>("#trigger")!;
const replaceInput = document.querySelector<HTMLTextAreaElement>("#replace")!;
const editorTitle = document.querySelector<HTMLHeadingElement>("#editor-title")!;
const toast = document.querySelector<HTMLDivElement>("#toast")!;

let snippets: Snippet[] = [];
let editingIndex: number | null = null;

function escapeHtml(value: string): string {
  const div = document.createElement("div");
  div.textContent = value;
  return div.innerHTML;
}

function showToast(message: string, isError = false) {
  toast.textContent = message;
  toast.classList.toggle("error", isError);
  toast.classList.add("show");
  window.setTimeout(() => toast.classList.remove("show"), 2600);
}

function render() {
  const query = search.value.toLocaleLowerCase("tr");
  const filtered = snippets.filter((snippet) =>
    `${snippet.trigger} ${snippet.replace}`.toLocaleLowerCase("tr").includes(query)
  );

  count.textContent = `${filtered.length} snippet`;

  if (!filtered.length) {
    list.innerHTML = `
      <div class="empty card">
        <div class="empty-icon">⌁</div>
        <strong>Eşleşen snippet yok.</strong>
        <span>Yeni bir kayıt ekleyebilir veya aramayı değiştirebilirsin.</span>
      </div>
    `;
    return;
  }

  list.innerHTML = filtered
    .map((snippet) => {
      const index = snippets.indexOf(snippet);
      return `
        <article class="snippet-card card">
          <div class="trigger">${escapeHtml(snippet.trigger)}</div>
          <div class="replacement">${escapeHtml(snippet.replace)}</div>
          <div class="row-actions">
            <button class="button small ghost" data-edit="${index}">Düzenle</button>
            <button class="button small danger" data-delete="${index}">Sil</button>
          </div>
        </article>
      `;
    })
    .join("");
}

async function refreshGitState() {
  try {
    const state = await invoke<GitState>("git_state");
    gitButton.disabled = false;
    gitButton.classList.toggle("dirty", state.dirty);
    gitButton.classList.toggle("clean", !state.dirty);
    gitButton.textContent = state.dirty ? "● GitHub’a Kaydet" : "✓ GitHub güncel";
  } catch (error) {
    gitButton.disabled = false;
    gitButton.textContent = "Git durumu alınamadı";
    gitButton.classList.add("danger");
    console.error(error);
  }
}

async function loadSnippets() {
  snippets = await invoke<Snippet[]>("list_snippets");
  render();
  await refreshGitState();
}

function openNew() {
  editingIndex = null;
  editorTitle.textContent = "Yeni snippet";
  triggerInput.value = ":";
  replaceInput.value = "";
  dialog.showModal();
  window.setTimeout(() => triggerInput.focus(), 0);
}

function openEdit(index: number) {
  editingIndex = index;
  editorTitle.textContent = "Snippet düzenle";
  triggerInput.value = snippets[index].trigger;
  replaceInput.value = snippets[index].replace;
  dialog.showModal();
  window.setTimeout(() => triggerInput.focus(), 0);
}

async function saveSnippet() {
  const trigger = triggerInput.value.trim();
  const replace = replaceInput.value;

  if (!trigger || !replace) {
    showToast("Trigger ve çıktı gerekli.", true);
    return;
  }

  try {
    await invoke("save_snippet", {
      index: editingIndex,
      snippet: { trigger, replace }
    });
    dialog.close();
    await loadSnippets();
    showToast(editingIndex === null ? "Snippet eklendi." : "Snippet güncellendi.");
  } catch (error) {
    showToast(String(error), true);
  }
}

async function deleteSnippet(index: number) {
  const snippet = snippets[index];
  if (!confirm(`${snippet.trigger} silinsin mi?`)) return;

  try {
    await invoke("delete_snippet", { index });
    await loadSnippets();
    showToast("Snippet silindi.");
  } catch (error) {
    showToast(String(error), true);
  }
}

async function saveToGitHub() {
  gitButton.disabled = true;
  gitButton.textContent = "Kaydediliyor…";
  try {
    const message = await invoke<string>("git_save");
    await refreshGitState();
    showToast(message);
  } catch (error) {
    await refreshGitState();
    showToast(String(error), true);
  }
}

search.addEventListener("input", render);
addButton.addEventListener("click", openNew);
gitButton.addEventListener("click", saveToGitHub);
document.querySelector("#cancel")?.addEventListener("click", () => dialog.close());
document.querySelector("#close-editor")?.addEventListener("click", () => dialog.close());

list.addEventListener("click", (event) => {
  const target = event.target as HTMLElement;
  const edit = target.closest<HTMLButtonElement>("[data-edit]");
  const del = target.closest<HTMLButtonElement>("[data-delete]");

  if (edit) openEdit(Number(edit.dataset.edit));
  if (del) void deleteSnippet(Number(del.dataset.delete));
});

form.addEventListener("submit", (event) => {
  event.preventDefault();
  void saveSnippet();
});

window.addEventListener("keydown", (event) => {
  if (event.ctrlKey && event.key.toLowerCase() === "n") {
    event.preventDefault();
    openNew();
  }

  if (event.ctrlKey && event.key.toLowerCase() === "s" && dialog.open) {
    event.preventDefault();
    void saveSnippet();
  }

  if (event.key === "Escape" && dialog.open) {
    dialog.close();
  }
});

void loadSnippets();
