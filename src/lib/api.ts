import { invoke } from "@tauri-apps/api/core";
import type {
  AIProposal,
  AppLog,
  AppSettings,
  CitationStatus,
  CitationTask,
  ClaimRecord,
  ExportJob,
  ExportValidationWarning,
  LiteratureItem,
  ManuscriptMode,
  ManuscriptSection,
  ProjectConfig,
  ProjectCreateInput,
  ProjectImportInput,
  ReferenceItem
} from "../types";
import {
  appLog,
  convertCitationsForMode,
  createInitialSections,
  createProjectConfig,
  makeId,
  mergeSections,
  nowIso,
  parseBibtexEntries,
  safeFolderName,
  scanWordCitationTasks,
  searchLiteratureMock,
  validateExportDraft
} from "./domain";

declare global {
  interface Window {
    __TAURI_INTERNALS__?: unknown;
  }
}

const isTauri = () => typeof window !== "undefined" && Boolean(window.__TAURI_INTERNALS__);

const storageKey = "paperforge.mvp.state";

interface BrowserState {
  projects: ProjectConfig[];
  sectionsByProject: Record<string, ManuscriptSection[]>;
  referencesByProject: Record<string, ReferenceItem[]>;
  bibtexByProject: Record<string, string>;
  citationTasksByProject: Record<string, CitationTask[]>;
  literatureByProject: Record<string, LiteratureItem[]>;
  claimsByProject: Record<string, ClaimRecord[]>;
  proposalsByProject: Record<string, AIProposal[]>;
  logs: AppLog[];
  settings: AppSettings;
}

export const defaultSettings: AppSettings = {
  workspaceRoot: "workspace",
  defaultManuscriptMode: "word",
  llmProvider: {
    baseUrl: "https://api.openai.com/v1",
    apiKey: "",
    model: "gpt-4.1-mini"
  },
  defaultCitationStyle: "apa",
  defaultExportMode: "word",
  themeMode: "dark"
};

const emptyState = (): BrowserState => ({
  projects: [],
  sectionsByProject: {},
  referencesByProject: {},
  bibtexByProject: {},
  citationTasksByProject: {},
  literatureByProject: {},
  claimsByProject: {},
  proposalsByProject: {},
  logs: [],
  settings: defaultSettings
});

function loadState(): BrowserState {
  const raw = localStorage.getItem(storageKey);
  if (!raw) return emptyState();
  try {
    const parsed = JSON.parse(raw) as Partial<BrowserState>;
    return {
      ...emptyState(),
      ...parsed,
      settings: {
        ...defaultSettings,
        ...(parsed.settings ?? {}),
        llmProvider: {
          ...defaultSettings.llmProvider,
          ...(parsed.settings?.llmProvider ?? {})
        }
      }
    } as BrowserState;
  } catch {
    return emptyState();
  }
}

function saveState(state: BrowserState) {
  localStorage.setItem(storageKey, JSON.stringify(state));
}

async function tauriOrBrowser<T>(command: string, args: Record<string, unknown>, fallback: () => T | Promise<T>) {
  if (isTauri()) return invoke<T>(command, args);
  return fallback();
}

export const api = {
  readSettings() {
    return tauriOrBrowser<AppSettings>("read_settings", {}, () => loadState().settings);
  },

  saveSettings(settings: AppSettings) {
    return tauriOrBrowser<AppSettings>("save_settings", { settings }, () => {
      const state = loadState();
      state.settings = settings;
      saveState(state);
      return settings;
    });
  },

  listProjects() {
    return tauriOrBrowser<ProjectConfig[]>("list_projects", {}, () => loadState().projects);
  },

  readAppLogs() {
    return tauriOrBrowser<AppLog[]>("read_app_logs", {}, () => loadState().logs ?? []);
  },

  appendAppLog(log: AppLog) {
    return tauriOrBrowser<AppLog[]>("append_app_log", { log }, () => {
      const state = loadState();
      state.logs = [log, ...(state.logs ?? [])].slice(0, 80);
      saveState(state);
      return state.logs;
    });
  },

  createProject(input: ProjectCreateInput) {
    return tauriOrBrowser<ProjectConfig>("create_project", { input }, () => {
      const state = loadState();
      const rootPath = `${input.workspaceRoot || state.settings.workspaceRoot}/${safeFolderName(input.title)}`;
      const project = createProjectConfig(input, rootPath);
      state.projects = [project, ...state.projects];
      state.sectionsByProject[project.id] = createInitialSections();
      state.referencesByProject[project.id] = [];
      state.bibtexByProject[project.id] = "";
      state.citationTasksByProject[project.id] = [];
      state.literatureByProject[project.id] = [];
      state.claimsByProject[project.id] = [];
      state.proposalsByProject[project.id] = [];
      saveState(state);
      return project;
    });
  },

  importProject(input: ProjectImportInput) {
    return tauriOrBrowser<ProjectConfig>("import_project_folder", { rootPath: input.rootPath }, () => {
      const state = loadState();
      const folderName = input.rootPath.trim().split(/[\\/]/).filter(Boolean).pop() ?? "Imported_Project";
      const project = createProjectConfig(
        {
          title: folderName.replace(/_/g, " "),
          author: "",
          targetJournal: "",
          manuscriptMode: state.settings.defaultManuscriptMode,
          workspaceRoot: input.rootPath
        },
        input.rootPath.trim()
      );
      state.projects = [project, ...state.projects.filter((item) => item.rootPath !== project.rootPath)];
      state.sectionsByProject[project.id] = createInitialSections();
      state.referencesByProject[project.id] = [];
      state.bibtexByProject[project.id] = "";
      state.citationTasksByProject[project.id] = [];
      state.literatureByProject[project.id] = [];
      state.claimsByProject[project.id] = [];
      state.proposalsByProject[project.id] = [];
      saveState(state);
      return project;
    });
  },

  deleteProject(projectId: string) {
    return tauriOrBrowser<boolean>("delete_project", { projectId, deleteFiles: false }, () => {
      const state = loadState();
      state.projects = state.projects.filter((project) => project.id !== projectId);
      delete state.sectionsByProject[projectId];
      delete state.referencesByProject[projectId];
      delete state.bibtexByProject[projectId];
      delete state.citationTasksByProject[projectId];
      delete state.literatureByProject[projectId];
      delete state.claimsByProject[projectId];
      delete state.proposalsByProject[projectId];
      saveState(state);
      return true;
    });
  },

  exportProjectManifest(projectId: string) {
    return tauriOrBrowser<string>("export_project_manifest", { projectId }, () => {
      const state = loadState();
      const project = state.projects.find((item) => item.id === projectId);
      if (!project) throw new Error("Project not found");
      return JSON.stringify(
        {
          exportedAt: nowIso(),
          project,
          sections: state.sectionsByProject[projectId] ?? [],
          references: state.referencesByProject[projectId] ?? [],
          citationTasks: state.citationTasksByProject[projectId] ?? [],
          literature: state.literatureByProject[projectId] ?? [],
          claims: state.claimsByProject[projectId] ?? [],
          note: "PaperForge MVP manifest export. User manuscripts stay local."
        },
        null,
        2
      );
    });
  },

  openProject(projectId: string) {
    return tauriOrBrowser<ProjectConfig>("open_project", { projectId }, () => {
      const project = loadState().projects.find((item) => item.id === projectId);
      if (!project) throw new Error("Project not found");
      return project;
    });
  },

  readProjectConfig(projectId: string) {
    return tauriOrBrowser<ProjectConfig>("read_project_config", { projectId }, () => {
      const project = loadState().projects.find((item) => item.id === projectId);
      if (!project) throw new Error("Project not found");
      return project;
    });
  },

  updateProjectConfig(project: ProjectConfig) {
    return tauriOrBrowser<ProjectConfig>("update_project_config", { project }, () => {
      const state = loadState();
      state.projects = state.projects.map((item) => (item.id === project.id ? project : item));
      saveState(state);
      return project;
    });
  },

  ensureProjectStructure(projectId: string) {
    return tauriOrBrowser<boolean>("ensure_project_structure", { projectId }, () => Boolean(projectId));
  },

  readSections(projectId: string) {
    return tauriOrBrowser<ManuscriptSection[]>("list_sections", { projectId }, () => {
      const state = loadState();
      if (!state.sectionsByProject[projectId]) state.sectionsByProject[projectId] = createInitialSections();
      saveState(state);
      return state.sectionsByProject[projectId];
    });
  },

  readSection(projectId: string, filename: string) {
    return tauriOrBrowser<ManuscriptSection>("read_section", { projectId, filename }, () => {
      const section = loadState().sectionsByProject[projectId]?.find((item) => item.filename === filename);
      if (!section) throw new Error("Section not found");
      return section;
    });
  },

  saveSection(projectId: string, section: ManuscriptSection) {
    return tauriOrBrowser<ManuscriptSection>("save_section", { projectId, section }, () => {
      const state = loadState();
      const sections = state.sectionsByProject[projectId] ?? [];
      state.sectionsByProject[projectId] = sections.map((item) =>
        item.id === section.id ? { ...section, updatedAt: nowIso() } : item
      );
      saveState(state);
      return section;
    });
  },

  listProjectTree(projectId: string) {
    return tauriOrBrowser("list_project_tree", { projectId }, () => []);
  },

  parseBibtex(bibtex: string) {
    return tauriOrBrowser<ReferenceItem[]>("parse_bibtex", { bibtex }, () => parseBibtexEntries(bibtex));
  },

  saveBibtex(projectId: string, bibtex: string) {
    return tauriOrBrowser<ReferenceItem[]>("save_bibtex", { projectId, bibtex }, () => {
      const state = loadState();
      const references = parseBibtexEntries(bibtex);
      state.bibtexByProject[projectId] = bibtex;
      state.referencesByProject[projectId] = references;
      saveState(state);
      return references;
    });
  },

  listReferences(projectId: string) {
    return tauriOrBrowser<ReferenceItem[]>("list_references", { projectId }, () => {
      return loadState().referencesByProject[projectId] ?? [];
    });
  },

  scanCitationTasks(projectId: string) {
    return tauriOrBrowser<CitationTask[]>("scan_citation_tasks", { projectId }, () => {
      const state = loadState();
      const tasks = scanWordCitationTasks(
        state.sectionsByProject[projectId] ?? [],
        state.referencesByProject[projectId] ?? [],
        state.citationTasksByProject[projectId] ?? []
      );
      state.citationTasksByProject[projectId] = tasks;
      saveState(state);
      return tasks;
    });
  },

  updateCitationTaskStatus(projectId: string, taskId: string, status: CitationStatus) {
    return tauriOrBrowser<CitationTask[]>("update_citation_task_status", { projectId, taskId, status }, () => {
      const state = loadState();
      state.citationTasksByProject[projectId] = (state.citationTasksByProject[projectId] ?? []).map((task) =>
        task.id === taskId ? { ...task, status } : task
      );
      saveState(state);
      return state.citationTasksByProject[projectId];
    });
  },

  addLiteratureItem(projectId: string, item: Omit<LiteratureItem, "id" | "embeddingStatus">) {
    return tauriOrBrowser<LiteratureItem>("add_literature_item", { projectId, item }, () => {
      const state = loadState();
      const literature: LiteratureItem = { ...item, id: makeId("lit"), embeddingStatus: "not_indexed" };
      state.literatureByProject[projectId] = [literature, ...(state.literatureByProject[projectId] ?? [])];
      saveState(state);
      return literature;
    });
  },

  listLiterature(projectId: string) {
    return tauriOrBrowser<LiteratureItem[]>("list_literature", { projectId }, () => loadState().literatureByProject[projectId] ?? []);
  },

  searchLiteratureMock(projectId: string, query: string) {
    return tauriOrBrowser<LiteratureItem[]>("search_literature_mock", { projectId, query }, () =>
      searchLiteratureMock(loadState().literatureByProject[projectId] ?? [], query)
    );
  },

  listClaims(projectId: string) {
    return tauriOrBrowser<ClaimRecord[]>("list_claims", { projectId }, () => loadState().claimsByProject[projectId] ?? []);
  },

  saveClaims(projectId: string, claims: ClaimRecord[]) {
    return tauriOrBrowser<ClaimRecord[]>("save_claims", { projectId, claims }, () => {
      const state = loadState();
      state.claimsByProject[projectId] = claims;
      saveState(state);
      return claims;
    });
  },

  addClaim(projectId: string, claim: ClaimRecord) {
    return tauriOrBrowser<ClaimRecord>("add_claim", { projectId, claim }, () => {
      const state = loadState();
      state.claimsByProject[projectId] = [claim, ...(state.claimsByProject[projectId] ?? [])];
      saveState(state);
      return claim;
    });
  },

  updateClaimStatus(projectId: string, claimId: string, status: ClaimRecord["status"]) {
    return tauriOrBrowser<ClaimRecord[]>("update_claim_status", { projectId, claimId, status }, () => {
      const state = loadState();
      state.claimsByProject[projectId] = (state.claimsByProject[projectId] ?? []).map((claim) =>
        claim.id === claimId ? { ...claim, status } : claim
      );
      saveState(state);
      return state.claimsByProject[projectId];
    });
  },

  generateAiProposal(projectId: string, sectionId: string, instruction: string, selectedText: string, settings: AppSettings) {
    return tauriOrBrowser<AIProposal>("generate_ai_proposal", { projectId, sectionId, instruction, selectedText, settings }, () => {
      const state = loadState();
      const mockPrefix = settings.llmProvider.apiKey ? "Provider abstraction ready; mock proposal:" : "MOCK: no API key configured.";
      const proposal: AIProposal = {
        id: makeId("proposal"),
        sectionId,
        instruction,
        originalText: selectedText,
        proposedText: `${mockPrefix}\n\n${selectedText || "This paragraph"} can be revised with clearer research gap, cautious claims, and citation hooks such as [CITE: Zhang2023].`,
        citationKeys: ["Zhang2023"],
        createdAt: nowIso(),
        status: "pending"
      };
      state.proposalsByProject[projectId] = [proposal, ...(state.proposalsByProject[projectId] ?? [])];
      saveState(state);
      return proposal;
    });
  },

  applyAiProposal(projectId: string, proposal: AIProposal, section: ManuscriptSection) {
    return tauriOrBrowser<ManuscriptSection>("apply_ai_proposal", { projectId, proposal, section }, () => {
      const state = loadState();
      const sections = state.sectionsByProject[projectId] ?? [];
      const nextSection = {
        ...section,
        content: proposal.originalText
          ? section.content.replace(proposal.originalText, proposal.proposedText)
          : `${section.content.trim()}\n\n${proposal.proposedText}\n`,
        updatedAt: nowIso()
      };
      state.sectionsByProject[projectId] = sections.map((item) => (item.id === section.id ? nextSection : item));
      state.proposalsByProject[projectId] = (state.proposalsByProject[projectId] ?? []).map((item) =>
        item.id === proposal.id ? { ...item, status: "applied" } : item
      );
      saveState(state);
      return nextSection;
    });
  },

  async exportProject(projectId: string, mode: ManuscriptMode, sections: ManuscriptSection[]) {
    const command = mode === "word" ? "export_word_draft" : mode === "latex" ? "export_latex" : "export_markdown_pandoc";
    return tauriOrBrowser<ExportJob>(command, { projectId }, () => ({
      id: makeId("export"),
      projectId,
      mode,
      status: "success",
      outputPath: `outputs/${mode === "latex" ? "main.tex" : mode === "word" ? "combined_word_draft.md" : "combined.md"}`,
      logs: [
        "MOCK export completed in browser fallback.",
        mode === "word"
          ? "Word draft keeps [CITE: key] placeholders for Zotero Word plugin."
          : mode === "latex"
            ? "LaTeX export uses \\cite{key} and references.bib."
            : "Markdown/Pandoc export uses [@key] syntax.",
        convertCitationsForMode(mergeSections(sections), mode).slice(0, 80)
      ],
      createdAt: nowIso()
    }));
  },

  validateExport(mode: ManuscriptMode, sections: ManuscriptSection[], references: ReferenceItem[]): ExportValidationWarning[] {
    return validateExportDraft(mode, sections, references);
  },

  openOutputFolder(path: string) {
    return tauriOrBrowser<boolean>("open_path", { path }, () => Boolean(path));
  },

  appLog
};
