import type {
  AppLog,
  CitationBackend,
  CitationTask,
  ClaimRecord,
  LiteratureItem,
  ManuscriptMode,
  ManuscriptSection,
  ProjectConfig,
  ProjectCreateInput,
  ReferenceItem
} from "../types";

export const sectionTemplates: Array<Pick<ManuscriptSection, "filename" | "title" | "order" | "content">> = [
  {
    filename: "01_abstract.md",
    title: "Abstract",
    order: 1,
    content: "## Abstract\n\nDraft the study objective, methods, key results, and conclusion.\n"
  },
  {
    filename: "02_introduction.md",
    title: "Introduction",
    order: 2,
    content: "## Introduction\n\nFrame the research gap and cite prior work.\n"
  },
  {
    filename: "03_methods.md",
    title: "Methods",
    order: 3,
    content: "## Methods\n\nDescribe materials, setup, datasets, and analysis methods.\n"
  },
  {
    filename: "04_results.md",
    title: "Results",
    order: 4,
    content: "## Results\n\nReport findings with traceable evidence.\n"
  },
  {
    filename: "05_discussion.md",
    title: "Discussion",
    order: 5,
    content: "## Discussion\n\nInterpret results, limitations, and implications.\n"
  },
  {
    filename: "06_conclusion.md",
    title: "Conclusion",
    order: 6,
    content: "## Conclusion\n\nSummarize contribution and next work.\n"
  }
];

export const projectFolders = [
  "manuscript/sections",
  "references/csl",
  "literature/pdfs",
  "literature/notes",
  "literature/embeddings",
  "templates/latex_template",
  "figures/raw",
  "figures/processed",
  "data/raw",
  "data/processed",
  "ai/prompts",
  "ai/writing_logs",
  "outputs"
];

export function nowIso() {
  return new Date().toISOString();
}

export function makeId(prefix: string) {
  return `${prefix}_${crypto.randomUUID?.() ?? Math.random().toString(36).slice(2)}`;
}

export function citationBackendForMode(mode: ManuscriptMode): CitationBackend {
  if (mode === "word") return "zotero_word_plugin";
  if (mode === "latex") return "bibtex";
  return "pandoc";
}

export function safeFolderName(title: string) {
  const normalized = title.trim().replace(/[<>:"/\\|?*\u0000-\u001F]/g, "");
  return (normalized || "Paper_Project").replace(/\s+/g, "_");
}

export function createProjectConfig(input: ProjectCreateInput, rootPath: string): ProjectConfig {
  const timestamp = nowIso();
  return {
    id: makeId("project"),
    title: input.title.trim() || "Untitled Paper",
    author: input.author.trim(),
    targetJournal: input.targetJournal.trim(),
    manuscriptMode: input.manuscriptMode,
    rootPath,
    createdAt: timestamp,
    updatedAt: timestamp,
    citationBackend: citationBackendForMode(input.manuscriptMode)
  };
}

export function createInitialSections(): ManuscriptSection[] {
  const timestamp = nowIso();
  return sectionTemplates.map((section) => ({
    id: section.filename.replace(".md", ""),
    filename: section.filename,
    title: section.title,
    order: section.order,
    content: section.content,
    updatedAt: timestamp
  }));
}

export function formatCitation(mode: ManuscriptMode, citekey: string) {
  const clean = citekey.trim();
  if (!clean) return "";
  if (mode === "word") return `[CITE: ${clean}]`;
  if (mode === "latex") return `\\cite{${clean}}`;
  return `[@${clean}]`;
}

function getBibField(body: string, field: string) {
  const regex = new RegExp(`${field}\\s*=\\s*(\\{([^{}]*(?:\\{[^{}]*\\}[^{}]*)*)\\}|"([^"]*)")`, "i");
  const match = body.match(regex);
  return (match?.[2] ?? match?.[3] ?? "").replace(/\s+/g, " ").trim();
}

export function parseBibtexEntries(bibtex: string): ReferenceItem[] {
  const entries: ReferenceItem[] = [];
  const entryRegex = /@\w+\s*\{\s*([^,\s]+)\s*,([\s\S]*?)(?=\n@\w+\s*\{|$)/g;
  let match: RegExpExecArray | null;
  while ((match = entryRegex.exec(bibtex)) !== null) {
    const citekey = match[1].trim();
    const body = match[2];
    const author = getBibField(body, "author");
    entries.push({
      citekey,
      title: getBibField(body, "title") || "(untitled)",
      authors: author ? author.split(/\s+and\s+/i).map((item) => item.trim()) : [],
      year: getBibField(body, "year"),
      journal: getBibField(body, "journal") || getBibField(body, "booktitle"),
      doi: getBibField(body, "doi"),
      abstract: getBibField(body, "abstract")
    });
  }
  return entries;
}

export function scanWordCitationTasks(
  sections: ManuscriptSection[],
  references: ReferenceItem[],
  previous: CitationTask[] = []
): CitationTask[] {
  const byKey = new Map(references.map((ref) => [ref.citekey, ref]));
  const statusByIdentity = new Map(previous.map((task) => [`${task.sectionId}:${task.placeholder}`, task.status]));
  return sections.flatMap((section) => {
    const matches = [...section.content.matchAll(/\[CITE:\s*([A-Za-z0-9_:.+-]+)\s*\]/g)];
    return matches.map((match, index) => {
      const placeholder = match[0];
      return {
        id: `${section.id}_${match.index ?? index}_${match[1]}`,
        sectionId: section.id,
        placeholder,
        citekey: match[1],
        status: statusByIdentity.get(`${section.id}:${placeholder}`) ?? "pending",
        reference: byKey.get(match[1])
      };
    });
  });
}

export function searchLiteratureMock(items: LiteratureItem[], query: string) {
  const q = query.trim().toLowerCase();
  if (!q) return items;
  return items.filter((item) =>
    [item.filename, item.path, item.linkedCitekey, item.notes, item.abstract]
      .filter(Boolean)
      .some((value) => value!.toLowerCase().includes(q))
  );
}

export function markdownToPreview(markdown: string) {
  return markdown
    .replace(/^### (.*)$/gm, "<h3>$1</h3>")
    .replace(/^## (.*)$/gm, "<h2>$1</h2>")
    .replace(/^# (.*)$/gm, "<h1>$1</h1>")
    .replace(/\*\*(.*?)\*\*/g, "<strong>$1</strong>")
    .replace(/\n{2,}/g, "</p><p>")
    .replace(/\n/g, "<br />")
    .replace(/^/, "<p>")
    .replace(/$/, "</p>");
}

export function mergeSections(sections: ManuscriptSection[]) {
  return sections
    .slice()
    .sort((a, b) => a.order - b.order)
    .map((section) => section.content.trim())
    .join("\n\n");
}

export function createClaim(claim: string, section: string, citationKeys: string[]): ClaimRecord {
  return {
    id: makeId("claim"),
    section,
    claim,
    citationKeys,
    evidenceChunkIds: [],
    status: citationKeys.length ? "verified" : "needs_citation"
  };
}

export function appLog(level: AppLog["level"], message: string): AppLog {
  return {
    id: makeId("log"),
    level,
    message,
    createdAt: nowIso()
  };
}
