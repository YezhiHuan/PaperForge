import type {
  AgentFileChange,
  AgentLogEntry,
  AgentMode,
  AgentRun,
  AgentSkill,
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

export const builtInAgentSkills: AgentSkill[] = [
  {
    id: "ask.project-review",
    name: "Project Review",
    type: "ask",
    description: "Review project structure, manuscript sections, references, and attachments without changing files.",
    allowedTools: ["list_project_files", "list_sections", "list_figures", "check_broken_links", "write_agent_log"],
    requiresDiff: false,
    requiresConfirmation: false,
    writesFiles: false,
    riskLevel: "low"
  },
  {
    id: "ask.export-readiness",
    name: "Export Readiness",
    type: "ask",
    description: "Check whether the current manuscript is ready for Markdown, Word placeholder, or LaTeX export.",
    allowedTools: ["list_project_files", "list_sections", "check_broken_links", "write_agent_log"],
    requiresDiff: false,
    requiresConfirmation: false,
    writesFiles: false,
    riskLevel: "low"
  },
  {
    id: "edit.academic-polish",
    name: "Academic Polish",
    type: "edit",
    description: "Improve academic style while preserving technical meaning, citations, and numbers.",
    allowedTools: ["read_project_file", "patch_project_file", "write_agent_log"],
    requiresDiff: true,
    requiresConfirmation: true,
    writesFiles: true,
    riskLevel: "medium"
  },
  {
    id: "edit.translate-zh-en",
    name: "Translate ZH-EN",
    type: "edit",
    description: "Translate or bilingual-polish the active section while preserving citations and technical details.",
    allowedTools: ["read_project_file", "patch_project_file", "write_agent_log"],
    requiresDiff: true,
    requiresConfirmation: true,
    writesFiles: true,
    riskLevel: "medium"
  },
  {
    id: "operate.insert-figure",
    name: "Insert Figure",
    type: "operate",
    description: "Prepare a safe Markdown figure insertion using files under attachments/figures.",
    allowedTools: ["list_figures", "read_project_file", "patch_project_file", "write_agent_log"],
    requiresDiff: true,
    requiresConfirmation: true,
    writesFiles: true,
    riskLevel: "medium"
  }
];

export function nowIso() {
  return new Date().toISOString();
}

export function makeId(prefix: string) {
  return `${prefix}_${crypto.randomUUID?.() ?? Math.random().toString(36).slice(2)}`;
}

export function selectAgentSkill(mode: AgentMode, requestedSkillId: string | undefined, request: string) {
  if (requestedSkillId && requestedSkillId !== "auto") {
    return builtInAgentSkills.find((skill) => skill.id === requestedSkillId) ?? builtInAgentSkills.find((skill) => skill.type === mode)!;
  }
  const q = request.toLowerCase();
  if (mode === "ask" && /export|word|latex|markdown|ready|导出|投稿/.test(q)) return builtInAgentSkills.find((skill) => skill.id === "ask.export-readiness")!;
  if (mode === "edit" && /translate|translation|中文|英文|翻译|中英/.test(q)) return builtInAgentSkills.find((skill) => skill.id === "edit.translate-zh-en")!;
  if (mode === "operate") return builtInAgentSkills.find((skill) => skill.id === "operate.insert-figure")!;
  return builtInAgentSkills.find((skill) => skill.type === mode)!;
}

export function makeSimpleDiff(path: string, original: string, proposed: string) {
  if (original === proposed) return `--- ${path}\n+++ ${path}\n(no changes)`;
  const originalLines = original.split("\n");
  const proposedLines = proposed.split("\n");
  const lines = [`--- ${path}`, `+++ ${path}`];
  const max = Math.max(originalLines.length, proposedLines.length);
  for (let index = 0; index < max; index += 1) {
    const before = originalLines[index];
    const after = proposedLines[index];
    if (before === after) {
      if (before !== undefined) lines.push(` ${before}`);
    } else {
      if (before !== undefined) lines.push(`-${before}`);
      if (after !== undefined) lines.push(`+${after}`);
    }
  }
  return lines.join("\n");
}

export function createAgentLogEntry(run: AgentRun, success: boolean, error?: string): AgentLogEntry {
  return {
    id: makeId("agent_log"),
    runId: run.id,
    projectId: run.projectId,
    mode: run.mode,
    skillId: run.skillId,
    request: run.request,
    tools: run.toolResults.map((tool) => tool.tool),
    filesRead: run.filesRead,
    filesChanged: run.filesChanged,
    success,
    error,
    createdAt: nowIso()
  };
}

export function createAgentChange(path: string, originalContent: string, proposedContent: string): AgentFileChange {
  return {
    id: makeId("agent_change"),
    path,
    changeType: originalContent ? "update" : "create",
    originalContent,
    proposedContent,
    diff: makeSimpleDiff(path, originalContent, proposedContent),
    status: "pending"
  };
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
    version: "2.1.1",
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
  const escapeHtml = (value: string) =>
    value.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
  const inline = (value: string) =>
    escapeHtml(value)
      .replace(/`([^`]+)`/g, "<code>$1</code>")
      .replace(/\*\*([^*]+)\*\*/g, "<strong>$1</strong>")
      .replace(/\*([^*]+)\*/g, "<em>$1</em>")
      .replace(/\[([^\]]+)\]\((https?:\/\/[^)\s]+)\)/g, '<a href="$2" target="_blank" rel="noreferrer">$1</a>');
  const lines = markdown.split(/\r?\n/);
  const html: string[] = [];
  let inCode = false;
  let inList = false;
  let inQuote = false;
  const closeList = () => {
    if (inList) {
      html.push("</ul>");
      inList = false;
    }
  };
  const closeQuote = () => {
    if (inQuote) {
      html.push("</blockquote>");
      inQuote = false;
    }
  };
  for (const line of lines) {
    if (line.startsWith("```")) {
      closeList();
      closeQuote();
      html.push(inCode ? "</code></pre>" : "<pre><code>");
      inCode = !inCode;
      continue;
    }
    if (inCode) {
      html.push(`${escapeHtml(line)}\n`);
      continue;
    }
    if (!line.trim()) {
      closeList();
      closeQuote();
      continue;
    }
    const heading = line.match(/^(#{1,3})\s+(.*)$/);
    if (heading) {
      closeList();
      closeQuote();
      const level = heading[1].length;
      html.push(`<h${level}>${inline(heading[2])}</h${level}>`);
      continue;
    }
    const list = line.match(/^\s*[-*]\s+(.*)$/);
    if (list) {
      closeQuote();
      if (!inList) {
        html.push("<ul>");
        inList = true;
      }
      html.push(`<li>${inline(list[1])}</li>`);
      continue;
    }
    const quote = line.match(/^>\s?(.*)$/);
    if (quote) {
      closeList();
      if (!inQuote) {
        html.push("<blockquote>");
        inQuote = true;
      }
      html.push(`<p>${inline(quote[1])}</p>`);
      continue;
    }
    closeList();
    closeQuote();
    html.push(`<p>${inline(line)}</p>`);
  }
  closeList();
  closeQuote();
  if (inCode) html.push("</code></pre>");
  return html.join("\n");
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
