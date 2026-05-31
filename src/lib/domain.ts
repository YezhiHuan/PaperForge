import type {
  AppLog,
  CitationBackend,
  CitationTask,
  ClaimRecord,
  ExportValidationWarning,
  LiteratureItem,
  ManuscriptMode,
  ManuscriptSection,
  ProjectActivity,
  ProjectConfig,
  ProjectCreateInput,
  ReferenceItem,
  SectionNamingMode,
  SectionTemplateId
} from "../types";

export const sectionTemplateOptions: Array<{ id: SectionTemplateId; label: string; sections: string[] }> = [
  { id: "empty", label: "Empty manuscript", sections: [] },
  {
    id: "standard",
    label: "Standard research paper",
    sections: ["Abstract", "Introduction", "Methods", "Results", "Discussion", "Conclusion", "References"]
  },
  {
    id: "engineeringSimulation",
    label: "Engineering simulation paper",
    sections: [
      "Abstract",
      "Introduction",
      "Geometry and Physical Model",
      "Numerical Method",
      "Mesh Independence Study",
      "Results and Discussion",
      "Optimization Analysis",
      "Conclusion"
    ]
  },
  {
    id: "review",
    label: "Review paper",
    sections: ["Abstract", "Introduction", "Background", "Literature Review", "Discussion", "Future Perspectives", "Conclusion"]
  }
];

export const projectFolders = [
  "manuscript/sections",
  "references/papers",
  "references/bib",
  "references/notes",
  "attachments/figures",
  "attachments/tables",
  "attachments/raw-data",
  "attachments/supplementary",
  "exports/markdown",
  "exports/json",
  "exports/word",
  "exports/latex",
  ".paperforge"
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

export function slugifyTitle(title: string) {
  return title
    .trim()
    .toLowerCase()
    .normalize("NFKD")
    .replace(/[\u0300-\u036f]/g, "")
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "");
}

export function makeSectionFilename(title: string, index: number, naming: SectionNamingMode, existingFilenames: Set<string>) {
  const fallback = naming === "numbered" ? "section" : `section-${String(index).padStart(3, "0")}`;
  const slug = slugifyTitle(title) || fallback;
  const prefix = naming === "numbered" ? `${String(index).padStart(2, "0")}_` : "";
  const base = `${prefix}${slug}`;
  let filename = `${base}.md`;
  let suffix = 2;
  while (existingFilenames.has(filename)) {
    filename = `${base}_${suffix}.md`;
    suffix += 1;
  }
  existingFilenames.add(filename);
  return filename;
}

export function createProjectConfig(input: ProjectCreateInput, rootPath: string): ProjectConfig {
  const timestamp = nowIso();
  const sections = createInitialSections(input.sectionNames, input.sectionNaming);
  return {
    id: makeId("project"),
    version: "1.0.0",
    title: input.title.trim() || "Untitled Paper",
    author: input.author.trim(),
    authors: input.author.trim() ? input.author.split(",").map((item) => item.trim()).filter(Boolean) : [],
    targetJournal: input.targetJournal.trim() || "Unspecified Journal",
    journal: input.targetJournal.trim(),
    language: "en",
    citationStyle: input.citationStyle?.trim() || "apa",
    exportMode: input.exportMode ?? input.manuscriptMode,
    manuscriptMode: input.manuscriptMode,
    rootPath,
    createdAt: timestamp,
    updatedAt: timestamp,
    citationBackend: citationBackendForMode(input.manuscriptMode),
    manuscript: {
      sectionNaming: input.sectionNaming,
      sections: sections.map((section) => ({
        id: section.id,
        title: section.title,
        path: section.path,
        order: section.order,
        status: section.status,
        createdAt: section.createdAt,
        updatedAt: section.updatedAt
      }))
    },
    sections: sections.map((section) => ({
      id: section.id,
      title: section.title,
      path: section.path,
      order: section.order,
      status: section.status,
      createdAt: section.createdAt,
      updatedAt: section.updatedAt
    }))
  };
}

export function createInitialSections(sectionNames: string[] = [], naming: SectionNamingMode = "numbered"): ManuscriptSection[] {
  const timestamp = nowIso();
  const filenames = new Set<string>();
  return sectionNames
    .map((title) => title.trim())
    .filter(Boolean)
    .map((title, index) => {
      const order = index + 1;
      const filename = makeSectionFilename(title, order, naming, filenames);
      return {
        id: makeId("section"),
        filename,
        title,
        order,
        content: `## ${title}\n\n`,
        path: `manuscript/sections/${filename}`,
        status: "draft" as const,
        createdAt: timestamp,
        updatedAt: timestamp
      };
    });
}

export function formatCitation(mode: ManuscriptMode, citekey: string) {
  const clean = citekey.trim();
  if (!clean) return "";
  if (mode === "word") return `[CITE: ${clean}]`;
  if (mode === "latex") return `\\cite{${clean}}`;
  return `[@${clean}]`;
}

export function convertCitationsForMode(markdown: string, mode: ManuscriptMode) {
  if (mode === "word") {
    return markdown
      .replace(/\\cite\{([^}]+)\}/g, (_match, citekey: string) => `[CITE: ${citekey.trim()}]`)
      .replace(/\[@([A-Za-z0-9_:.+-]+)\]/g, (_match, citekey: string) => `[CITE: ${citekey.trim()}]`);
  }
  if (mode === "latex") {
    return markdown
      .replace(/\[CITE:\s*([A-Za-z0-9_:.+-]+)\s*\]/g, (_match, citekey: string) => `\\cite{${citekey.trim()}}`)
      .replace(/\[@([A-Za-z0-9_:.+-]+)\]/g, (_match, citekey: string) => `\\cite{${citekey.trim()}}`);
  }
  return markdown
    .replace(/\[CITE:\s*([A-Za-z0-9_:.+-]+)\s*\]/g, (_match, citekey: string) => `[@${citekey.trim()}]`)
    .replace(/\\cite\{([^}]+)\}/g, (_match, citekey: string) => `[@${citekey.trim()}]`);
}

export function validateExportDraft(mode: ManuscriptMode, sections: ManuscriptSection[], references: ReferenceItem[]): ExportValidationWarning[] {
  const draft = mergeSections(sections);
  const referenceKeys = new Set(references.map((reference) => reference.citekey));
  const warnings: ExportValidationWarning[] = [];
  const add = (severity: ExportValidationWarning["severity"], message: string) => {
    warnings.push({ id: makeId("export_warning"), severity, message });
  };

  if (!draft.trim()) {
    add("error", "Draft has no manuscript content.");
  }
  if (references.length === 0) {
    add("warning", "No references saved. Citation metadata may be missing.");
  }
  if (mode === "word") {
    const placeholders = [...draft.matchAll(/\[CITE:\s*([A-Za-z0-9_:.+-]+)\s*\]/g)];
    const missing = placeholders.map((match) => match[1]).filter((citekey) => !referenceKeys.has(citekey));
    if (placeholders.length > 0) add("info", `${placeholders.length} Word citation placeholder(s) will stay for Zotero Word plugin.`);
    if (missing.length > 0) add("warning", `Missing reference metadata for: ${[...new Set(missing)].join(", ")}.`);
  }
  if (mode === "latex" && /\[CITE:|\[@/.test(draft)) {
    add("info", "Word/Pandoc citation markers will be converted to LaTeX \\cite{}.");
  }
  if (mode === "markdown" && /\\cite\{|\[CITE:/.test(draft)) {
    add("info", "Word/LaTeX citation markers will be converted to Pandoc [@key].");
  }

  return warnings;
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

export function projectActivity(type: ProjectActivity["type"], message: string, sectionId?: string): ProjectActivity {
  return {
    id: makeId("activity"),
    type,
    message,
    sectionId,
    createdAt: nowIso()
  };
}
