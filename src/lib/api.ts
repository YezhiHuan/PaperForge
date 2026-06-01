import { invoke } from "@tauri-apps/api/core";
import type {
  AIProposal,
  AgentLogEntry,
  AgentMode,
  AgentRun,
  AgentSkill,
  AppLog,
  AppSettings,
  CitationStatus,
  CitationTask,
  ClaimRecord,
  ExportJob,
  ExportValidationWarning,
  FileTreeNode,
  LiteratureItem,
  ManuscriptMode,
  ManuscriptSection,
  ProjectConfig,
  ProjectCreateInput,
  ProjectImportInput,
  ProjectActivity,
  ReferenceItem,
  SectionCreateInput,
  SectionRenameInput,
  TextFilePayload,
  WorkspaceConfig,
  AgentChatMessage,
  AgentChatResponse,
  AgentChatToolTrace,
} from "../types";
import {
  appLog,
  builtInAgentSkills,
  convertCitationsForMode,
  createAgentChange,
  createAgentLogEntry,
  createProjectConfig,
  makeId,
  makeSectionFilename,
  nowIso,
  parseBibtexEntries,
  projectActivity,
  safeFolderName,
  scanWordCitationTasks,
  selectAgentSkill,
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
  agentRunsByProject: Record<string, AgentRun[]>;
  agentLogsByProject: Record<string, AgentLogEntry[]>;
  activitiesByProject: Record<string, ProjectActivity[]>;
  logs: AppLog[];
  settings: AppSettings;
}

export const defaultSettings: AppSettings = {
  workspaceRoot: "workspace",
  defaultManuscriptMode: "word",
  llmProvider: {
    provider: "openai-compatible",
    baseUrl: "https://api.openai.com/v1",
    apiKey: "",
    model: "gpt-4.1-mini",
    temperature: 0.3,
    maxTokens: 2000
  },
  defaultCitationStyle: "apa",
  defaultExportMode: "markdown",
  themeMode: "light",
  language: "en"
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
  agentRunsByProject: {},
  agentLogsByProject: {},
  activitiesByProject: {},
  logs: [],
  settings: defaultSettings
});

function normalizeProject(project: ProjectConfig): ProjectConfig {
  const title = project.title?.trim() || "Untitled Paper";
  const author = project.author?.trim() ?? "";
  const targetJournal = project.targetJournal?.trim() || "Unspecified Journal";
  return {
    ...project,
    version: project.version ?? "2.1.1",
    title,
    author,
    authors: project.authors ?? (author ? author.split(",").map((item) => item.trim()).filter(Boolean) : []),
    targetJournal,
    journal: targetJournal === "Unspecified Journal" ? "" : targetJournal,
    language: project.language ?? "en",
    citationStyle: project.citationStyle?.trim() || defaultSettings.defaultCitationStyle,
    exportMode: project.exportMode ?? project.manuscriptMode ?? defaultSettings.defaultExportMode,
    manuscript: project.manuscript ?? {
      sectionNaming: "numbered",
      sections: []
    },
    sections: project.manuscript?.sections ?? project.sections ?? []
  };
}

function syncProjectManifest(project: ProjectConfig, sections: ManuscriptSection[]) {
  return {
    ...normalizeProject(project),
    updatedAt: nowIso(),
    manuscript: {
      sectionNaming: project.manuscript?.sectionNaming ?? "numbered",
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

  initWorkspace(rootPath: string) {
    return tauriOrBrowser<WorkspaceConfig>("init_workspace", { rootPath }, () => {
      const state = loadState();
      state.settings = { ...state.settings, workspaceRoot: rootPath || "workspace", themeMode: state.settings.themeMode || "light" };
      saveState(state);
      return {
        version: "2.1.1",
        workspaceName: "workspace",
        createdAt: nowIso(),
        updatedAt: nowIso(),
        papersDir: "papers",
        defaultLanguage: "en"
      };
    });
  },

  listProjects() {
    return tauriOrBrowser<ProjectConfig[]>("list_projects", {}, () => loadState().projects.map(normalizeProject));
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
      const normalizedInput = {
        ...input,
        title: input.title?.trim() || "Untitled Paper",
        author: input.author?.trim() || "",
        targetJournal: input.targetJournal?.trim() || "Unspecified Journal",
        manuscriptMode: input.manuscriptMode || state.settings.defaultManuscriptMode,
        citationStyle: input.citationStyle || state.settings.defaultCitationStyle || "apa",
        exportMode: input.exportMode || state.settings.defaultExportMode || "markdown"
      };
      const rootPath = `${input.workspaceRoot || state.settings.workspaceRoot}/papers/${safeFolderName(normalizedInput.title)}`;
      const project = normalizeProject(createProjectConfig(normalizedInput, rootPath));
      state.projects = [project, ...state.projects];
      state.sectionsByProject[project.id] = project.manuscript.sections.map((section) => ({
        id: section.id,
        filename: section.path.split("/").pop() ?? `${section.id}.md`,
        title: section.title,
        order: section.order,
        content: `## ${section.title}\n\n`,
        path: section.path,
        status: section.status,
        createdAt: section.createdAt,
        updatedAt: section.updatedAt
      }));
      state.referencesByProject[project.id] = [];
      state.bibtexByProject[project.id] = "";
      state.citationTasksByProject[project.id] = [];
      state.literatureByProject[project.id] = [];
      state.claimsByProject[project.id] = [];
      state.proposalsByProject[project.id] = [];
      state.agentRunsByProject[project.id] = [];
      state.agentLogsByProject[project.id] = [];
      state.activitiesByProject[project.id] = [];
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
          targetJournal: "Unspecified Journal",
          manuscriptMode: state.settings.defaultManuscriptMode,
          citationStyle: state.settings.defaultCitationStyle,
          exportMode: state.settings.defaultExportMode,
          workspaceRoot: input.rootPath,
          sectionNaming: "numbered",
          sectionNames: []
        },
        input.rootPath.trim()
      );
      const normalizedProject = normalizeProject(project);
      state.projects = [normalizedProject, ...state.projects.filter((item) => item.rootPath !== project.rootPath)];
      state.sectionsByProject[project.id] = project.manuscript.sections.map((section) => ({
        id: section.id,
        filename: section.path.split("/").pop() ?? `${section.id}.md`,
        title: section.title,
        order: section.order,
        content: `## ${section.title}\n\n`,
        path: section.path,
        status: section.status,
        createdAt: section.createdAt,
        updatedAt: section.updatedAt
      }));
      state.referencesByProject[project.id] = [];
      state.bibtexByProject[project.id] = "";
      state.citationTasksByProject[project.id] = [];
      state.literatureByProject[project.id] = [];
      state.claimsByProject[project.id] = [];
      state.proposalsByProject[project.id] = [];
      state.agentRunsByProject[project.id] = [];
      state.agentLogsByProject[project.id] = [];
      state.activitiesByProject[project.id] = [];
      saveState(state);
      return normalizedProject;
    });
  },

  deleteProject(projectId: string) {
    return tauriOrBrowser<boolean>("delete_project", { projectId, deleteFiles: true }, () => {
      throw new Error("Paper folder deletion requires the PaperForge desktop app.");
    });
  },

  exportProjectManifest(projectId: string) {
    return tauriOrBrowser<string>("export_project_manifest", { projectId }, () => {
      const state = loadState();
      const project = state.projects.find((item) => item.id === projectId);
      if (!project) throw new Error("Project not found");
      return JSON.stringify(syncProjectManifest(project, state.sectionsByProject[projectId] ?? []), null, 2);
    });
  },

  openProject(projectId: string) {
    return tauriOrBrowser<ProjectConfig>("open_project", { projectId }, () => {
      const project = loadState().projects.find((item) => item.id === projectId);
      if (!project) throw new Error("Project not found");
      return normalizeProject(project);
    });
  },

  readProjectConfig(projectId: string) {
    return tauriOrBrowser<ProjectConfig>("read_project_config", { projectId }, () => {
      const project = loadState().projects.find((item) => item.id === projectId);
      if (!project) throw new Error("Project not found");
      return normalizeProject(project);
    });
  },

  updateProjectConfig(project: ProjectConfig) {
    return tauriOrBrowser<ProjectConfig>("update_project_config", { project }, () => {
      const state = loadState();
      const normalized = normalizeProject({ ...project, updatedAt: nowIso() });
      state.projects = state.projects.map((item) => (item.id === project.id ? normalized : item));
      saveState(state);
      return normalized;
    });
  },

  updateProjectMetadata(projectId: string, partial: Partial<Pick<ProjectConfig, "title" | "author" | "authors" | "targetJournal" | "manuscriptMode" | "citationStyle" | "exportMode">>) {
    return tauriOrBrowser<ProjectConfig>("update_project_metadata", { projectId, partial }, () => {
      const state = loadState();
      const project = state.projects.find((item) => item.id === projectId);
      if (!project) throw new Error("Project not found");
      const next = normalizeProject({
        ...project,
        ...partial,
        title: partial.title !== undefined ? partial.title.trim() || "Untitled Paper" : project.title,
        targetJournal: partial.targetJournal !== undefined ? partial.targetJournal.trim() || "Unspecified Journal" : project.targetJournal,
        updatedAt: nowIso()
      });
      state.projects = state.projects.map((item) => (item.id === projectId ? next : item));
      saveState(state);
      return next;
    });
  },

  ensureProjectStructure(projectId: string) {
    return tauriOrBrowser<boolean>("ensure_project_structure", { projectId }, () => Boolean(projectId));
  },

  readSections(projectId: string) {
    return tauriOrBrowser<ManuscriptSection[]>("list_sections", { projectId }, () => {
      const state = loadState();
      if (!state.sectionsByProject[projectId]) state.sectionsByProject[projectId] = [];
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
      const nextSections = sections.map((item) =>
        item.id === section.id ? { ...section, updatedAt: nowIso() } : item
      );
      state.sectionsByProject[projectId] = nextSections;
      state.projects = state.projects.map((project) => project.id === projectId ? syncProjectManifest(project, nextSections) : project);
      saveState(state);
      return section;
    });
  },

  createSection(projectId: string, input: SectionCreateInput) {
    return tauriOrBrowser<ManuscriptSection>("create_section", { projectId, input }, () => {
      const state = loadState();
      const project = normalizeProject(state.projects.find((item) => item.id === projectId)!);
      const sections = state.sectionsByProject[projectId] ?? [];
      const order = sections.length + 1;
      const filenames = new Set(sections.map((section) => section.filename));
      const filename = makeSectionFilename(input.title, order, project.manuscript.sectionNaming, filenames);
      const timestamp = nowIso();
      const section: ManuscriptSection = {
        id: makeId("section"),
        filename,
        title: input.title.trim(),
        order,
        content: `## ${input.title.trim()}\n\n`,
        path: `manuscript/sections/${filename}`,
        status: "draft",
        createdAt: timestamp,
        updatedAt: timestamp
      };
      const nextSections = [...sections, section];
      state.sectionsByProject[projectId] = nextSections;
      state.projects = state.projects.map((item) => item.id === projectId ? syncProjectManifest(project, nextSections) : item);
      state.activitiesByProject[projectId] = [
        projectActivity("section.created", `Created section: ${section.title}`, section.id),
        ...(state.activitiesByProject[projectId] ?? [])
      ];
      saveState(state);
      return section;
    });
  },

  renameSection(projectId: string, input: SectionRenameInput) {
    return tauriOrBrowser<ManuscriptSection>("rename_section", { projectId, input }, () => {
      const state = loadState();
      const sections = state.sectionsByProject[projectId] ?? [];
      const timestamp = nowIso();
      const nextSections = sections.map((section) =>
        section.id === input.sectionId ? { ...section, title: input.title.trim(), updatedAt: timestamp } : section
      );
      const section = nextSections.find((item) => item.id === input.sectionId);
      if (!section) throw new Error("Section not found");
      state.sectionsByProject[projectId] = nextSections;
      const project = normalizeProject(state.projects.find((item) => item.id === projectId)!);
      state.projects = state.projects.map((item) => item.id === projectId ? syncProjectManifest(project, nextSections) : item);
      state.activitiesByProject[projectId] = [
        projectActivity("section.renamed", `Renamed section: ${section.title}. File path kept: ${section.path}`, section.id),
        ...(state.activitiesByProject[projectId] ?? [])
      ];
      saveState(state);
      return section;
    });
  },

  listProjectTree(projectId: string) {
    return tauriOrBrowser("list_project_tree", { projectId }, () => []);
  },

  listProjectFiles(projectId: string) {
    return tauriOrBrowser<FileTreeNode[]>("list_project_files", { projectId }, () => {
      const state = loadState();
      const project = state.projects.find((item) => item.id === projectId);
      const sections = state.sectionsByProject[projectId] ?? [];
      const sectionNodes: FileTreeNode[] = sections.map((section) => ({
        name: section.filename,
        path: section.path,
        relativePath: section.path,
        kind: "file",
        extension: "md"
      }));
      return [
        {
          name: "manuscript",
          path: "manuscript",
          relativePath: "manuscript",
          kind: "directory",
          children: [
            {
              name: "sections",
              path: "manuscript/sections",
              relativePath: "manuscript/sections",
              kind: "directory",
              children: sectionNodes
            }
          ]
        },
        { name: "references", path: "references", relativePath: "references", kind: "directory", children: [] },
        { name: "attachments", path: "attachments", relativePath: "attachments", kind: "directory", children: [] },
        { name: "exports", path: "exports", relativePath: "exports", kind: "directory", children: [] },
        {
          name: "paperforge.json",
          path: "paperforge.json",
          relativePath: "paperforge.json",
          kind: "file",
          extension: "json"
        },
        ...(project ? [] : [])
      ];
    });
  },

  readTextFile(projectId: string, path: string) {
    return tauriOrBrowser<TextFilePayload>("read_text_file", { projectId, path }, () => {
      const state = loadState();
      const section = (state.sectionsByProject[projectId] ?? []).find((item) => item.path === path);
      if (!section) throw new Error("Browser fallback can read manuscript sections only.");
      return { path, content: section.content };
    });
  },

  writeTextFile(projectId: string, path: string, content: string) {
    return tauriOrBrowser<TextFilePayload>("write_text_file", { projectId, path, content }, () => {
      const state = loadState();
      const sections = state.sectionsByProject[projectId] ?? [];
      state.sectionsByProject[projectId] = sections.map((section) => section.path === path ? { ...section, content, updatedAt: nowIso() } : section);
      saveState(state);
      return { path, content };
    });
  },

  deleteTextFile(projectId: string, path: string) {
    return tauriOrBrowser<string>("delete_text_file", { projectId, path }, () => {
      throw new Error("Deleting non-section files requires the PaperForge desktop app.");
    });
  },

  agentChat(projectId: string, messages: AgentChatMessage[], settings: AppSettings) {
    return tauriOrBrowser<AgentChatResponse>("agent_chat_with_tools", { request: { projectId, settings, messages } }, () => {
      throw new Error("Agent tool chat requires the PaperForge desktop app. Run `npm run tauri:dev` to test tool calling.");
    });
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

  listAgentSkills(projectId: string) {
    return tauriOrBrowser<AgentSkill[]>("list_agent_skills", { projectId }, () => builtInAgentSkills);
  },

  runAgent(projectId: string, mode: AgentMode, skillId: string, request: string, sectionId?: string) {
    return tauriOrBrowser<AgentRun>("run_agent", { projectId, mode, skillId, request, sectionId }, () => {
      const state = loadState();
      const sections = state.sectionsByProject[projectId] ?? [];
      const skill = selectAgentSkill(mode, skillId, request);
      const activeSection = sections.find((section) => section.id === sectionId) ?? sections[0];
      const timestamp = nowIso();
      const filesRead: string[] = [];
      const changes = [];
      let report = "";
      const toolResults = skill.allowedTools.map((tool) => ({ tool, ok: true, message: `${tool} available in browser fallback.` }));
      if (mode === "ask" || mode === "edit") {
        throw new Error("Project Agent LLM requires the PaperForge desktop app.");
      }

      if (activeSection) {
        filesRead.push(activeSection.path, "attachments/figures");
        const figureMatch = request.match(/attachments\/figures\/[^\s)]+/i);
        const figurePath = figureMatch?.[0] ?? "attachments/figures/figure-placeholder.png";
        const proposed = `${activeSection.content.trim()}\n\n![Figure caption](${figurePath})\n`;
        changes.push(createAgentChange(activeSection.path, activeSection.content, proposed));
        report = "Insert Figure prepared a Markdown image reference. Only attachments/figures paths are accepted by the desktop safe filesystem.";
      } else {
        report = "No active section found. Create a section before using Operate mode.";
      }

      const run: AgentRun = {
        id: makeId("agent_run"),
        projectId,
        mode,
        skillId: skill.id,
        request,
        status: changes.length ? "planned" : "completed",
        plan: {
          summary: `${skill.name}: ${request || "No request text provided."}`,
          steps: ["Read active section", "Prepare safe diff", "Wait for Apply or Reject"],
          filesToRead: filesRead,
          filesToChange: changes.map((change) => change.path)
        },
        filesRead,
        filesChanged: [],
        report,
        changes,
        toolResults,
        createdAt: timestamp,
        updatedAt: timestamp
      };
      state.agentRunsByProject[projectId] = [run, ...(state.agentRunsByProject[projectId] ?? [])];
      state.agentLogsByProject[projectId] = [createAgentLogEntry(run, true), ...(state.agentLogsByProject[projectId] ?? [])].slice(0, 80);
      saveState(state);
      return run;
    });
  },

  applyAgentChange(projectId: string, runId: string, changeId: string) {
    return tauriOrBrowser<AgentRun>("apply_agent_change", { projectId, runId, changeId }, () => {
      const state = loadState();
      const runs = state.agentRunsByProject[projectId] ?? [];
      const run = runs.find((item) => item.id === runId);
      if (!run) throw new Error("Agent run not found");
      const change = run.changes.find((item) => item.id === changeId);
      if (!change) throw new Error("Agent change not found");
      const sections = state.sectionsByProject[projectId] ?? [];
      const timestamp = nowIso();
      state.sectionsByProject[projectId] = sections.map((section) =>
        section.path === change.path ? { ...section, content: change.proposedContent, updatedAt: timestamp } : section
      );
      const project = normalizeProject(state.projects.find((item) => item.id === projectId)!);
      state.projects = state.projects.map((item) => (item.id === projectId ? syncProjectManifest(project, state.sectionsByProject[projectId]) : item));
      const updatedRun: AgentRun = {
        ...run,
        status: "applied",
        filesChanged: [...new Set([...run.filesChanged, change.path])],
        changes: run.changes.map((item) => item.id === changeId ? { ...item, status: "applied" } : item),
        updatedAt: timestamp
      };
      state.agentRunsByProject[projectId] = runs.map((item) => item.id === runId ? updatedRun : item);
      state.agentLogsByProject[projectId] = [createAgentLogEntry(updatedRun, true), ...(state.agentLogsByProject[projectId] ?? [])].slice(0, 80);
      saveState(state);
      return updatedRun;
    });
  },

  rejectAgentRun(projectId: string, runId: string) {
    return tauriOrBrowser<AgentRun>("reject_agent_run", { projectId, runId }, () => {
      const state = loadState();
      const runs = state.agentRunsByProject[projectId] ?? [];
      const run = runs.find((item) => item.id === runId);
      if (!run) throw new Error("Agent run not found");
      const updatedRun: AgentRun = {
        ...run,
        status: "rejected",
        changes: run.changes.map((change) => ({ ...change, status: "rejected" })),
        updatedAt: nowIso()
      };
      state.agentRunsByProject[projectId] = runs.map((item) => item.id === runId ? updatedRun : item);
      state.agentLogsByProject[projectId] = [createAgentLogEntry(updatedRun, true), ...(state.agentLogsByProject[projectId] ?? [])].slice(0, 80);
      saveState(state);
      return updatedRun;
    });
  },

  readAgentLog(projectId: string) {
    return tauriOrBrowser<AgentLogEntry[]>("read_agent_log", { projectId }, () => loadState().agentLogsByProject[projectId] ?? []);
  },

  generateAiProposal(projectId: string, sectionId: string, instruction: string, selectedText: string, settings: AppSettings) {
    return tauriOrBrowser<AIProposal>("generate_ai_proposal", { projectId, sectionId, instruction, selectedText, settings }, () => {
      throw new Error("AI proposal generation requires the PaperForge desktop app.");
    });
  },

  testAiConnection(settings: AppSettings) {
    return tauriOrBrowser<string>("test_ai_connection", { settings }, () => {
      if (!settings.llmProvider.apiKey.trim()) throw new Error("API key is required. Configure it in Settings.");
      if (!settings.llmProvider.model.trim()) throw new Error("Model is required. Choose or enter a model in Settings.");
      throw new Error("AI connection test requires the PaperForge desktop app.");
    });
  },

  fetchAiModels(settings: AppSettings) {
    return tauriOrBrowser<string[]>("fetch_ai_models", { settings }, () => {
      if (!settings.llmProvider.apiKey.trim()) throw new Error("API key is required. Configure it in Settings.");
      throw new Error("Model fetching requires the PaperForge desktop app.");
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
    const command = mode === "markdown" ? "export_markdown_package" : mode === "word" ? "export_word_draft" : "export_latex";
    return tauriOrBrowser<ExportJob>(command, { projectId }, () => {
      if (mode !== "markdown") {
        return {
          id: makeId("export"),
          projectId,
          mode,
          status: "failed",
          outputPath: "",
          logs: [
            "Desktop mode with Pandoc is required for Word and LaTeX export."
          ],
          createdAt: nowIso()
        };
      }
      const state = loadState();
      const project = normalizeProject(state.projects.find((item) => item.id === projectId)!);
      const stamp = nowIso().replace(/[-:]/g, "").replace(/\..+/, "").replace("T", "-");
      const outputPath = `${project.rootPath}/exports/markdown/paperforge-export-${stamp}`;
      const markdown = sections.length
        ? sections
            .slice()
            .sort((a, b) => a.order - b.order)
            .map((section) => `# ${section.title}\n\n${convertCitationsForMode(section.content.trim(), "markdown")}`)
            .join("\n\n")
        : "<!-- Empty manuscript exported by PaperForge -->\n";
      return {
      id: makeId("export"),
      projectId,
      mode,
      status: "success",
        outputPath,
      logs: [
          "Browser fallback preview only. Desktop mode writes the package folder.",
          "manifest.json, paper.md, sections/, references/, attachments/, claims/, export-report.json",
          markdown.slice(0, 80)
      ],
      createdAt: nowIso()
      };
    });
  },

  exportProjectFolder(projectId: string) {
    return tauriOrBrowser<ExportJob>("export_project_folder", { projectId }, () => {
      const state = loadState();
      const project = normalizeProject(state.projects.find((item) => item.id === projectId)!);
      return {
        id: makeId("export"),
        projectId,
        mode: "markdown",
        status: "success",
        outputPath: `${project.rootPath}/exports/project-folder/browser-preview`,
        logs: ["Browser fallback preview only. Desktop mode writes a project folder snapshot."],
        createdAt: nowIso()
      };
    });
  },

  validateExport(mode: ManuscriptMode, sections: ManuscriptSection[], references: ReferenceItem[]): ExportValidationWarning[] {
    return validateExportDraft(mode, sections, references);
  },

  openOutputFolder(path: string) {
    return tauriOrBrowser<boolean>("open_path", { path }, () => Boolean(path));
  },

  appLog
};
