import { AnimatePresence, motion } from "framer-motion";
import {
  AlertTriangle,
  BookOpen,
  Brain,
  Check,
  ChevronDown,
  Clipboard,
  Copy,
  Download,
  ExternalLink,
  FileText,
  Folder,
  FolderOpen,
  Library,
  Loader2,
  Moon,
  Pencil,
  Plus,
  RefreshCw,
  Save,
  Search,
  Settings,
  Sparkles,
  Trash2,
  X
} from "lucide-react";
import { FormEvent, useEffect, useMemo, useState } from "react";
import type { ReactNode } from "react";
import { api, defaultSettings } from "./lib/api";
import { displayTitle, internalTitle, t } from "./i18n";
import type { MessageKey } from "./i18n";
import { createClaim, formatCitation, markdownToPreview, mergeSections, nowIso, sectionTemplateOptions } from "./lib/domain";
import { APP_VERSION } from "./version";
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
  ExportValidationWarning,
  ExportJob,
  FileTreeNode,
  Language,
  LlmProviderKind,
  LiteratureItem,
  ManuscriptMode,
  ManuscriptSection,
  ProjectConfig,
  ReferenceItem,
  SectionNamingMode,
  SectionTemplateId,
  SidebarMode,
  ThemeMode
} from "./types";

type ToolTab = "info" | "agent" | "references" | "citations" | "literature" | "claims" | "export";
type Translate = (key: MessageKey) => string;
type AppView = "main" | "settings";
type ActiveMarkdownFile = { path: string; name: string; content: string; originalContent: string };

const TEXT_FILE_EXTENSIONS = new Set([
  "md", "markdown", "json", "bib", "bibtex", "tex", "txt",
  "csv", "tsv", "xml", "yaml", "yml", "toml", "log",
  "cfg", "ini", "rst", "html", "css", "js", "ts", "tsx", "jsx"
]);

function isTextFile(name: string, extension?: string): boolean {
  const ext = (extension ?? (name.includes(".") ? name.split(".").pop() ?? "" : "")).toLowerCase();
  return TEXT_FILE_EXTENSIONS.has(ext);
}

function viewerModeFor(name: string, extension?: string): "markdown" | "code" {
  const ext = (extension ?? (name.includes(".") ? name.split(".").pop() ?? "" : "")).toLowerCase();
  return ext === "md" || ext === "markdown" ? "markdown" : "code";
}
type DialogState =
  | { kind: "confirm"; title: string; description: string; confirmLabel: string; danger?: boolean; resolve: (value: boolean) => void }
  | { kind: "input"; title: string; description: string; defaultValue: string; placeholder?: string; confirmLabel: string; validate?: (value: string) => string | undefined; resolve: (value: string | null) => void }
  | { kind: "message"; title: string; description: string; resolve: () => void };

const cardVariants = {
  hidden: { opacity: 0, y: 14 },
  show: (index: number) => ({ opacity: 1, y: 0, transition: { delay: index * 0.045, duration: 0.18 } })
};

const panelVariants = {
  hidden: { opacity: 0, y: 8 },
  show: { opacity: 1, y: 0, transition: { duration: 0.16 } },
  exit: { opacity: 0, y: -8, transition: { duration: 0.12 } }
};

const emptyProjectForm = {
  title: "",
  author: "",
  targetJournal: "",
  manuscriptMode: "word" as ManuscriptMode,
  workspaceRoot: "",
  sectionTemplate: "empty" as SectionTemplateId,
  sectionNaming: "numbered" as SectionNamingMode,
  sectionNames: [] as string[]
};

const emptyImportForm = {
  rootPath: ""
};

function outputFolderFromPath(path: string) {
  const normalized = path.replace(/\\/g, "/");
  const lastSlash = normalized.lastIndexOf("/");
  if (lastSlash <= 0) return path;
  return path.slice(0, lastSlash);
}

function App() {
  const [view, setView] = useState<AppView>("main");
  const [settings, setSettings] = useState<AppSettings>(defaultSettings);
  const [projects, setProjects] = useState<ProjectConfig[]>([]);
  const [activeProject, setActiveProject] = useState<ProjectConfig | null>(null);
  const [sections, setSections] = useState<ManuscriptSection[]>([]);
  const [activeSectionId, setActiveSectionId] = useState("");
  const [editorMode, setEditorMode] = useState<"edit" | "preview">("edit");
  const [references, setReferences] = useState<ReferenceItem[]>([]);
  const [citationTasks, setCitationTasks] = useState<CitationTask[]>([]);
  const [literature, setLiterature] = useState<LiteratureItem[]>([]);
  const [claims, setClaims] = useState<ClaimRecord[]>([]);
  const [logs, setLogs] = useState<AppLog[]>([]);
  const [toolTab, setToolTab] = useState<ToolTab>("agent");
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [projectForm, setProjectForm] = useState(emptyProjectForm);
  const [showImportModal, setShowImportModal] = useState(false);
  const [importForm, setImportForm] = useState(emptyImportForm);
  const [expandedFolders, setExpandedFolders] = useState<Record<string, boolean>>({
    manuscript: true,
    references: true,
    attachments: false,
    exports: false,
    paperforge: false
  });
  const [fileTree, setFileTree] = useState<FileTreeNode[]>([]);
  const [fileTreeError, setFileTreeError] = useState("");
  const [activeMarkdownFile, setActiveMarkdownFile] = useState<ActiveMarkdownFile | null>(null);
  const [dialog, setDialog] = useState<DialogState | null>(null);
  const [aiModels, setAiModels] = useState<string[]>([]);
  const [settingsStatus, setSettingsStatus] = useState("");
  const [bibtex, setBibtex] = useState("");
  const [aiInstruction, setAiInstruction] = useState("Rewrite selected paragraph");
  const [aiLoading, setAiLoading] = useState(false);
  const [proposal, setProposal] = useState<AIProposal | null>(null);
  const [agentMode, setAgentMode] = useState<AgentMode>("ask");
  const [agentSkillId, setAgentSkillId] = useState("auto");
  const [agentRequest, setAgentRequest] = useState("Review this project");
  const [agentSkills, setAgentSkills] = useState<AgentSkill[]>([]);
  const [agentRun, setAgentRun] = useState<AgentRun | null>(null);
  const [agentLogs, setAgentLogs] = useState<AgentLogEntry[]>([]);
  const [agentLoading, setAgentLoading] = useState(false);
  const [litForm, setLitForm] = useState({ filename: "", path: "", linkedCitekey: "", notes: "" });
  const [litQuery, setLitQuery] = useState("");
  const [litSearching, setLitSearching] = useState(false);
  const [exportJob, setExportJob] = useState<ExportJob | null>(null);
  const [exportRunning, setExportRunning] = useState(false);
  const [exportWarnings, setExportWarnings] = useState<ExportValidationWarning[]>([]);
  const [draftSettings, setDraftSettings] = useState<AppSettings>(defaultSettings);
  const [claimText, setClaimText] = useState("");
  const tr: Translate = (key) => t(settings.language, key);

  const activeSection = useMemo(
    () => sections.find((section) => section.id === activeSectionId) ?? sections[0],
    [activeSectionId, sections]
  );
  const activeDocument = activeMarkdownFile ?? activeSection;
  const activeDocumentTitle = activeMarkdownFile?.name ?? activeSection?.title ?? "No section";

  const addLog = (level: AppLog["level"], message: string) => {
    const log = api.appLog(level, message);
    setLogs((current) => [log, ...current].slice(0, 8));
    void api.appendAppLog(log).catch(() => undefined);
  };

  const errorMessage = (error: unknown, fallback: string) => (error instanceof Error ? error.message : fallback);

  const showMessage = (title: string, description: string) =>
    new Promise<void>((resolve) => setDialog({ kind: "message", title, description, resolve }));

  const askConfirm = (title: string, description: string, confirmLabel: string, danger = false) =>
    new Promise<boolean>((resolve) => setDialog({ kind: "confirm", title, description, confirmLabel, danger, resolve }));

  const askInput = (
    title: string,
    description: string,
    defaultValue = "",
    confirmLabel = "OK",
    placeholder = "",
    validate?: (value: string) => string | undefined
  ) =>
    new Promise<string | null>((resolve) => setDialog({ kind: "input", title, description, defaultValue, confirmLabel, placeholder, validate, resolve }));

  const openCreateModal = () => {
    setProjectForm({ ...emptyProjectForm, manuscriptMode: settings.defaultManuscriptMode });
    setShowCreateModal(true);
  };

  useEffect(() => {
    async function boot() {
      const [loadedSettings, loadedProjects, loadedLogs] = await Promise.all([api.readSettings(), api.listProjects(), api.readAppLogs()]);
      setSettings(loadedSettings);
      setDraftSettings(loadedSettings);
      setProjects(loadedProjects);
      setLogs(loadedLogs.slice(0, 8));
      if (loadedProjects[0]) {
        await selectProject(loadedProjects[0]);
      } else {
        addLog("info", "Ready. Create first paper project.");
      }
    }
    void boot();
  }, []);

  useEffect(() => {
    const applyTheme = () => {
      const resolved =
        settings.themeMode === "system"
          ? window.matchMedia?.("(prefers-color-scheme: dark)").matches
            ? "dark"
            : "light"
          : settings.themeMode;
      document.documentElement.dataset.theme = resolved;
    };
    applyTheme();
    if (settings.themeMode !== "system") return;
    const query = window.matchMedia?.("(prefers-color-scheme: dark)");
    query?.addEventListener("change", applyTheme);
    return () => query?.removeEventListener("change", applyTheme);
  }, [settings.themeMode]);

  async function selectProject(project: ProjectConfig) {
    const openedProject = await api.openProject(project.id);
    setActiveProject(openedProject);
    setSections([]);
    setActiveSectionId("");
    let loadedSections: ManuscriptSection[] = [];
    try {
      loadedSections = await api.readSections(openedProject.id);
    } catch (error) {
      addLog("warning", errorMessage(error, "Could not load manuscript sections."));
    }
    const [refsResult, tasksResult, litResult, claimsResult, skillsResult, logsResult] = await Promise.allSettled([
      api.listReferences(openedProject.id),
      api.scanCitationTasks(openedProject.id),
      api.listLiterature(openedProject.id),
      api.listClaims(openedProject.id),
      api.listAgentSkills(openedProject.id),
      api.readAgentLog(openedProject.id)
    ]);
    const warnIfRejected = (result: PromiseSettledResult<unknown>, fallback: string) => {
      if (result.status === "rejected") {
        addLog("warning", errorMessage(result.reason, fallback));
      }
    };
    warnIfRejected(refsResult, "Could not load references.");
    warnIfRejected(tasksResult, "Could not load citation tasks.");
    warnIfRejected(litResult, "Could not load literature library.");
    warnIfRejected(claimsResult, "Could not load claims.");
    warnIfRejected(skillsResult, "Could not load agent skills.");
    warnIfRejected(logsResult, "Could not load agent log.");
    let refreshedProject = openedProject;
    try {
      refreshedProject = await api.readProjectConfig(openedProject.id);
    } catch (error) {
      addLog("warning", errorMessage(error, "Could not refresh project manifest."));
    }
    setActiveProject(refreshedProject);
    setProjects((current) => current.map((item) => (item.id === refreshedProject.id ? refreshedProject : item)));
    setSections(loadedSections);
    setActiveSectionId(loadedSections[0]?.id ?? "");
    setActiveMarkdownFile(null);
    setReferences(refsResult.status === "fulfilled" ? refsResult.value : []);
    setCitationTasks(tasksResult.status === "fulfilled" ? tasksResult.value : []);
    setLiterature(litResult.status === "fulfilled" ? litResult.value : []);
    setClaims(claimsResult.status === "fulfilled" ? claimsResult.value : []);
    setAgentSkills(skillsResult.status === "fulfilled" ? skillsResult.value : []);
    setAgentLogs(logsResult.status === "fulfilled" ? logsResult.value : []);
    setAgentRun(null);
    await refreshFileTree(openedProject.id);
    addLog("success", `Opened ${refreshedProject.title}`);
  }

  async function refreshFileTree(projectId = activeProject?.id) {
    if (!projectId) return;
    try {
      setFileTreeError("");
      setFileTree(await api.listProjectFiles(projectId));
    } catch (error) {
      const message = errorMessage(error, "Could not read project files.");
      setFileTreeError(message);
      addLog("warning", message);
    }
  }

  async function createProject(event: FormEvent) {
    event.preventDefault();
    const project = await api.createProject({
      ...projectForm,
      workspaceRoot: projectForm.workspaceRoot || settings.workspaceRoot,
      citationStyle: settings.defaultCitationStyle,
      exportMode: settings.defaultExportMode,
      manuscriptMode: projectForm.manuscriptMode || settings.defaultManuscriptMode
    });
    const nextProjects = await api.listProjects();
    setProjects(nextProjects);
    setShowCreateModal(false);
    setProjectForm({ ...emptyProjectForm, manuscriptMode: settings.defaultManuscriptMode });
    await selectProject(project);
    addLog("success", "Project structure generated. No .git folder created.");
  }

  async function importProject(event: FormEvent) {
    event.preventDefault();
    const project = await api.importProject({ rootPath: importForm.rootPath });
    const nextProjects = await api.listProjects();
    setProjects(nextProjects);
    setShowImportModal(false);
    setImportForm(emptyImportForm);
    await selectProject(project);
    addLog("success", `Imported existing project folder: ${project.rootPath}`);
  }

  async function deleteProject(project: ProjectConfig) {
    const ok = await askConfirm(
      "Delete paper",
      `This permanently deletes the local paper folder:\n${project.rootPath}`,
      "Delete",
      true
    );
    if (!ok) return;
    try {
      await api.deleteProject(project.id);
      const nextProjects = await api.listProjects();
      setProjects(nextProjects);
      if (activeProject?.id === project.id) {
        setActiveProject(null);
        setSections([]);
        setReferences([]);
        setCitationTasks([]);
        setLiterature([]);
        setClaims([]);
        setAgentSkills([]);
        setAgentLogs([]);
        setAgentRun(null);
      }
      addLog("warning", `Deleted paper folder: ${project.rootPath}`);
    } catch (error) {
      const message = error instanceof Error ? error.message : "Could not delete paper folder. Check file permissions or close files opened by another program.";
      void showMessage("Delete failed", message);
      addLog("error", message);
    }
  }

  async function exportProjectManifest(project: ProjectConfig) {
    const manifest = await api.exportProjectManifest(project.id);
    const blob = new Blob([manifest], { type: "application/json" });
    const url = URL.createObjectURL(blob);
    const link = document.createElement("a");
    link.href = url;
    link.download = `${internalTitle(project.title).replace(/[^\w.-]+/g, "_")}_paperforge_manifest.json`;
    link.click();
    URL.revokeObjectURL(url);
    addLog("success", `Exported project manifest: ${project.title}`);
  }

  async function initializeWorkspace() {
    const workspace = await api.initWorkspace(settings.workspaceRoot);
    addLog("success", `Workspace ready: ${settings.workspaceRoot}/${workspace.papersDir}`);
  }

  async function updateProjectMetadata(
    project: ProjectConfig,
    partial: Partial<Pick<ProjectConfig, "title" | "author" | "authors" | "targetJournal" | "manuscriptMode" | "citationStyle" | "exportMode">>
  ) {
    const updated = await api.updateProjectMetadata(project.id, partial);
    setProjects((current) => current.map((item) => (item.id === updated.id ? updated : item)));
    if (activeProject?.id === updated.id) setActiveProject(updated);
    addLog("success", `Updated project metadata: ${displayTitle(updated.title, settings.language)}`);
  }

  async function editProjectTitle(project: ProjectConfig) {
    const title = await askInput("Paper title", "Rename paper. Folder path stays unchanged.", project.title === "Untitled Paper" ? "" : project.title, "Rename", "Paper title");
    if (title === null) return;
    await updateProjectMetadata(project, { title });
  }

  async function saveActiveSection() {
    if (!activeProject) return;
    if (activeMarkdownFile) {
      const saved = await api.writeTextFile(activeProject.id, activeMarkdownFile.path, activeMarkdownFile.content);
      setActiveMarkdownFile({ ...activeMarkdownFile, content: saved.content, originalContent: saved.content });
      await refreshFileTree(activeProject.id);
      addLog("success", `Saved Markdown file: ${saved.path}`);
      return;
    }
    if (!activeSection) return;
    const saved = await api.saveSection(activeProject.id, activeSection);
    setSections((current) => current.map((section) => (section.id === saved.id ? { ...saved, updatedAt: nowIso() } : section)));
    await refreshFileTree(activeProject.id);
    addLog("success", `Saved to workspace: ${activeSection.path}`);
  }

  async function createSectionFromPrompt() {
    if (!activeProject) return;
    const title = await askInput("Section title", "Create a Markdown section under manuscript/sections.", "", "Create", "Section title", (value) => value.trim() ? undefined : "Section title is required.");
    if (!title?.trim()) return;
    const section = await api.createSection(activeProject.id, { title });
    const loadedSections = await api.readSections(activeProject.id);
    const updatedProject = await api.readProjectConfig(activeProject.id);
    setActiveProject(updatedProject);
    setProjects((current) => current.map((project) => (project.id === updatedProject.id ? updatedProject : project)));
    setSections(loadedSections);
    setActiveSectionId(section.id);
    setActiveMarkdownFile(null);
    await refreshFileTree(activeProject.id);
    addLog("success", `Created section: ${section.title}`);
  }

  async function renameSectionFromPrompt(section: ManuscriptSection) {
    if (!activeProject) return;
    const title = await askInput("Rename section", "File path stays unchanged.", section.title, "Rename", "Section title", (value) => value.trim() ? undefined : "Section title is required.");
    if (!title?.trim() || title.trim() === section.title) return;
    const renamed = await api.renameSection(activeProject.id, { sectionId: section.id, title });
    const updatedProject = await api.readProjectConfig(activeProject.id);
    setActiveProject(updatedProject);
    setProjects((current) => current.map((project) => (project.id === updatedProject.id ? updatedProject : project)));
    setSections((current) => current.map((item) => (item.id === renamed.id ? renamed : item)));
    setActiveSectionId(renamed.id);
    addLog("info", `Renamed section: ${renamed.title}. File path kept.`);
  }

  function updateActiveSection(content: string) {
    if (activeMarkdownFile) {
      setActiveMarkdownFile({ ...activeMarkdownFile, content });
      return;
    }
    if (!activeSection) return;
    setSections((current) => current.map((section) => (section.id === activeSection.id ? { ...section, content } : section)));
  }

  function insertCitation(citekey: string) {
    if (!activeProject || !activeSection) return;
    const citation = formatCitation(activeProject.manuscriptMode, citekey);
    updateActiveSection(`${activeSection.content}${activeSection.content.endsWith(" ") ? "" : " "}${citation}`);
    addLog("info", `Inserted ${citation}`);
  }

  async function openMarkdownFile(path: string) {
    if (!activeProject) return;
    const section = sections.find((item) => item.path === path || item.filename === path.split("/").pop());
    if (section) {
      setActiveMarkdownFile(null);
      setActiveSectionId(section.id);
      return;
    }
    try {
      const file = await api.readTextFile(activeProject.id, path);
      setActiveSectionId("");
      setActiveMarkdownFile({
        path: file.path,
        name: file.path.split("/").pop() ?? file.path,
        content: file.content,
        originalContent: file.content
      });
      addLog("info", `Opened Markdown file: ${file.path}`);
    } catch (error) {
      const message = errorMessage(error, "Could not open Markdown file.");
      addLog("error", message);
      void showMessage("Open file failed", message);
    }
  }

  async function saveBibtex() {
    if (!activeProject) return;
    const parsed = await api.saveBibtex(activeProject.id, bibtex);
    setReferences(parsed);
    addLog("success", `Parsed ${parsed.length} BibTeX reference(s).`);
  }

  async function scanTasks() {
    if (!activeProject) return;
    const tasks = await api.scanCitationTasks(activeProject.id);
    setCitationTasks(tasks);
    addLog(tasks.length ? "warning" : "success", `Word citation scan: ${tasks.length} task(s).`);
  }

  async function setTaskStatus(taskId: string, status: CitationStatus) {
    if (!activeProject) return;
    const tasks = await api.updateCitationTaskStatus(activeProject.id, taskId, status);
    setCitationTasks(tasks);
    addLog("info", `Citation task marked ${status}.`);
  }

  async function addLiterature(event: FormEvent) {
    event.preventDefault();
    if (!activeProject) return;
    const item = await api.addLiteratureItem(activeProject.id, {
      filename: litForm.filename,
      path: litForm.path,
      linkedCitekey: litForm.linkedCitekey || undefined,
      notes: litForm.notes
    });
    setLiterature((current) => [item, ...current]);
    setLitForm({ filename: "", path: "", linkedCitekey: "", notes: "" });
    addLog("success", "PDF record added. Embedding status: not_indexed.");
  }

  async function runLiteratureSearch() {
    if (!activeProject) return;
    setLitSearching(true);
    window.setTimeout(async () => {
      const results = await api.searchLiteratureMock(activeProject.id, litQuery);
      setLiterature(results);
      setLitSearching(false);
      addLog("info", `Local literature search returned ${results.length} item(s).`);
    }, 420);
  }

  async function generateProposal(instruction = aiInstruction) {
    if (!activeProject || !activeDocument) {
      void showMessage("AI Assistant", "Open a Markdown section or file before asking AI.");
      return;
    }
    setAiLoading(true);
    try {
      const selectedText = "content" in activeDocument ? activeDocument.content : "";
      const generated = await api.generateAiProposal(activeProject.id, activeSection?.id ?? activeMarkdownFile?.path ?? "markdown_file", instruction, selectedText, settings);
      setProposal(generated);
      addLog("info", "AI proposal generated.");
    } catch (error) {
      const message = error instanceof Error ? error.message : "AI proposal failed.";
      addLog("error", message);
      void showMessage("AI proposal failed", message);
    } finally {
      setAiLoading(false);
    }
  }

  async function applyProposal() {
    if (!activeProject || !proposal) return;
    if (activeMarkdownFile) {
      const next = proposal.originalText
        ? activeMarkdownFile.content.replace(proposal.originalText, proposal.proposedText)
        : `${activeMarkdownFile.content.trim()}\n\n${proposal.proposedText}\n`;
      setActiveMarkdownFile({ ...activeMarkdownFile, content: next });
      setProposal({ ...proposal, status: "applied" });
      addLog("success", "AI proposal applied to open Markdown file. Save to write changes.");
      return;
    }
    if (!activeSection) return;
    const updated = await api.applyAiProposal(activeProject.id, proposal, activeSection);
    setSections((current) => current.map((section) => (section.id === updated.id ? updated : section)));
    setProposal({ ...proposal, status: "applied" });
    addLog("success", "AI proposal applied. Change recorded in app log, no Git commit.");
  }

  async function runAgent() {
    if (!activeProject) return;
    setAgentLoading(true);
    try {
      const run = await api.runAgent(activeProject.id, agentMode, agentSkillId, agentRequest, activeSection?.id);
      setAgentRun(run);
      setAgentLogs(await api.readAgentLog(activeProject.id));
      addLog(run.changes.length ? "info" : "success", `Agent ${run.status}: ${run.skillId}`);
    } catch (error) {
      const message = error instanceof Error ? error.message : "Agent run failed.";
      addLog("error", message);
      void showMessage("Agent run failed", message);
    } finally {
      setAgentLoading(false);
    }
  }

  async function applyAgentChange(changeId: string) {
    if (!activeProject || !agentRun) return;
    const run = await api.applyAgentChange(activeProject.id, agentRun.id, changeId);
    const updatedProject = await api.readProjectConfig(activeProject.id);
    setAgentRun(run);
    setSections(await api.readSections(activeProject.id));
    setActiveProject(updatedProject);
    setProjects((current) => current.map((project) => project.id === updatedProject.id ? updatedProject : project));
    setAgentLogs(await api.readAgentLog(activeProject.id));
    addLog("success", "Agent change applied with backup.");
  }

  async function rejectAgentRun() {
    if (!activeProject || !agentRun) return;
    const run = await api.rejectAgentRun(activeProject.id, agentRun.id);
    setAgentRun(run);
    setAgentLogs(await api.readAgentLog(activeProject.id));
    addLog("info", "Agent run rejected. No files changed.");
  }

  async function addClaimRecord() {
    if (!activeProject || !activeSection || !claimText.trim()) return;
    const citationKeys = [...claimText.matchAll(/\[CITE:\s*([^\]]+)\]|\[@([^\]]+)\]|\\cite\{([^}]+)\}/g)].map(
      (match) => match[1] ?? match[2] ?? match[3]
    );
    const claim = createClaim(claimText, activeSection.filename, citationKeys);
    const saved = await api.addClaim(activeProject.id, claim);
    setClaims((current) => [saved, ...current]);
    setClaimText("");
    addLog("info", `Claim added: ${saved.status}`);
  }

  async function runExport(mode: ManuscriptMode) {
    if (!activeProject) return;
    setExportWarnings(api.validateExport(mode, sections, references));
    setExportRunning(true);
    setExportJob({ id: "running", projectId: activeProject.id, mode, status: "running", outputPath: "", logs: ["Export running"], createdAt: nowIso() });
    try {
      const job = await api.exportProject(activeProject.id, mode, sections);
      setExportJob(job);
      await refreshFileTree(activeProject.id);
      addLog(job.status === "success" ? "success" : job.status === "failed" ? "error" : "info", `${mode} export ${job.status}: ${job.outputPath || job.logs[0]}`);
    } catch (error) {
      const message = errorMessage(error, `${mode} export failed.`);
      setExportJob({ id: "failed", projectId: activeProject.id, mode, status: "failed", outputPath: "", logs: [message], createdAt: nowIso() });
      addLog("error", message);
    } finally {
      setExportRunning(false);
    }
  }

  async function runProjectFolderExport() {
    if (!activeProject) return;
    setExportRunning(true);
    setExportJob({ id: "running", projectId: activeProject.id, mode: "markdown", status: "running", outputPath: "", logs: ["Project folder export running"], createdAt: nowIso() });
    const job = await api.exportProjectFolder(activeProject.id);
    setExportJob(job);
    setExportRunning(false);
    await refreshFileTree(activeProject.id);
    addLog("success", `Project folder export: ${job.outputPath}`);
  }

  async function openOutputFolder() {
    if (!exportJob?.outputPath) return;
    try {
      const opened = await api.openOutputFolder(outputFolderFromPath(exportJob.outputPath));
      addLog(opened ? "info" : "warning", opened ? "Opened output folder." : "Export completed, but opening folder failed.");
    } catch (error) {
      addLog("warning", `Export completed, but opening folder failed: ${errorMessage(error, "Unknown error")}`);
    }
  }

  async function openProjectFolder() {
    if (!activeProject) return;
    try {
      const opened = await api.openOutputFolder(activeProject.rootPath);
      addLog(opened ? "info" : "warning", opened ? "Opened project folder." : "Project folder open unavailable in browser mode.");
    } catch (error) {
      addLog("warning", errorMessage(error, "Could not open project folder."));
    }
  }

  async function applySettings(nextSettings: AppSettings) {
    setDraftSettings(nextSettings);
    setSettings(nextSettings);
    const saved = await api.saveSettings(nextSettings);
    setSettings(saved);
    setDraftSettings(saved);
  }

  async function testAiConnection() {
    try {
      setSettingsStatus("Testing AI connection...");
      setSettingsStatus(await api.testAiConnection(draftSettings));
    } catch (error) {
      setSettingsStatus(errorMessage(error, "AI connection failed."));
    }
  }

  async function fetchAiModels() {
    try {
      setSettingsStatus("Fetching models...");
      const models = await api.fetchAiModels(draftSettings);
      setAiModels(models);
      setSettingsStatus(`Fetched ${models.length} model(s).`);
    } catch (error) {
      setSettingsStatus(errorMessage(error, "Could not fetch models."));
    }
  }

  return (
    <div className="min-h-screen bg-[var(--bg)] text-[var(--text)]">
      <TopBar
        project={activeProject}
        onDashboard={() => { setView("main"); setActiveProject(null); }}
        onNew={openCreateModal}
        onSettings={() => setView("settings")}
        version={APP_VERSION}
        t={tr}
        language={settings.language}
      />

      {view === "settings" ? (
        <SettingsPage
          settings={draftSettings}
          setSettings={applySettings}
          onBack={() => setView("main")}
          onTestConnection={testAiConnection}
          onFetchModels={fetchAiModels}
          models={aiModels}
          status={settingsStatus}
          t={tr}
        />
      ) : !activeProject ? (
        <Dashboard
          projects={projects}
          onOpen={selectProject}
          onNew={openCreateModal}
          onImport={() => setShowImportModal(true)}
          onInitWorkspace={initializeWorkspace}
          onDelete={deleteProject}
          onEditTitle={editProjectTitle}
          workspaceRoot={settings.workspaceRoot}
          t={tr}
          language={settings.language}
        />
      ) : (
        <main className="workspace-grid">
          <Sidebar
            project={activeProject}
            sections={sections}
            expandedFolders={expandedFolders}
            setExpandedFolders={setExpandedFolders}
            setActiveSectionId={setActiveSectionId}
            onCreateSection={createSectionFromPrompt}
            onRenameSection={renameSectionFromPrompt}
            onSettings={() => setView("settings")}
            fileTree={fileTree}
            fileTreeError={fileTreeError}
            onRefreshFiles={() => refreshFileTree(activeProject.id)}
            onOpenTextFile={openMarkdownFile}
            activeSectionId={activeSectionId}
            activeFilePath={activeMarkdownFile?.path ?? activeSection?.path}
            mode={settings.sidebarMode ?? "writing"}
            onModeChange={(next) => applySettings({ ...settings, sidebarMode: next })}
            t={tr}
            language={settings.language}
          />

          <section className="editor-shell">
            <div className="editor-toolbar">
              <div>
                <p className="eyebrow">{tr("project.manuscript")}</p>
                <div className="title-edit-row">
                  <input
                    className="title-input"
                    value={activeProject.title === "Untitled Paper" ? "" : activeProject.title}
                    placeholder={displayTitle(activeProject.title, settings.language)}
                    onChange={(event) => setActiveProject({ ...activeProject, title: event.target.value })}
                    onBlur={(event) => updateProjectMetadata(activeProject, { title: event.target.value })}
                  />
                  <span>{activeDocumentTitle}</span>
                </div>
              </div>
              <div className="toolbar-actions">
                <button disabled={!activeDocument} className={editorMode === "edit" ? "seg active" : "seg"} onClick={() => setEditorMode("edit")}>
                  {tr("actions.edit")}
                </button>
                <button disabled={!activeDocument} className={editorMode === "preview" ? "seg active" : "seg"} onClick={() => setEditorMode("preview")}>
                  {tr("actions.preview")}
                </button>
                <button className="secondary-btn" onClick={createSectionFromPrompt}>
                  <Plus size={15} /> {tr("actions.createSection")}
                </button>
                <button className="primary-btn" disabled={!activeDocument} onClick={saveActiveSection}>
                  <Save size={15} /> {tr("actions.save")}
                </button>
              </div>
            </div>

            {activeDocument ? (() => {
              const codeView = activeMarkdownFile !== null && viewerModeFor(activeMarkdownFile.name) === "code";
              const codeLines = codeView ? activeDocument.content.split(/\r?\n/) : [];
              return (
              <AnimatePresence mode="wait">
                {editorMode === "edit" ? (
                  <motion.textarea
                    key="editor"
                    variants={panelVariants}
                    initial="hidden"
                    animate="show"
                    exit="exit"
                    className="manuscript-editor"
                    value={activeDocument.content}
                    onChange={(event) => updateActiveSection(event.target.value)}
                  />
                ) : codeView ? (
                  <motion.div
                    key="code"
                    variants={panelVariants}
                    initial="hidden"
                    animate="show"
                    exit="exit"
                    className="code-view"
                    aria-label={`Code view of ${activeDocumentTitle}`}
                  >
                    {codeLines.map((line, index) => (
                      <div className="code-line" key={index}>
                        <span className="code-ln">{index + 1}</span>
                        <span className="code-tx">{line || " "}</span>
                      </div>
                    ))}
                  </motion.div>
                ) : (
                  <motion.article
                    key="preview"
                    variants={panelVariants}
                    initial="hidden"
                    animate="show"
                    exit="exit"
                    className="preview"
                    dangerouslySetInnerHTML={{ __html: markdownToPreview(activeDocument.content) }}
                  />
                )}
              </AnimatePresence>
              );
            })() : (
              <motion.div className="manuscript-empty" variants={panelVariants} initial="hidden" animate="show">
                <FileText size={34} />
                <h2>{tr("project.emptyManuscript")}</h2>
                <p>No manuscript sections yet. Create your first section to start writing.</p>
                <button className="primary-btn" onClick={createSectionFromPrompt}>
                  <Plus size={15} /> {tr("actions.createSection")}
                </button>
              </motion.div>
            )}
          </section>

          <RightPanel
            tab={toolTab}
            setTab={setToolTab}
            project={activeProject}
            references={references}
            bibtex={bibtex}
            setBibtex={setBibtex}
            saveBibtex={saveBibtex}
            insertCitation={insertCitation}
            citationTasks={citationTasks}
            scanTasks={scanTasks}
            setTaskStatus={setTaskStatus}
            literature={literature}
            litForm={litForm}
            setLitForm={setLitForm}
            addLiterature={addLiterature}
            litQuery={litQuery}
            setLitQuery={setLitQuery}
            runLiteratureSearch={runLiteratureSearch}
            litSearching={litSearching}
            aiInstruction={aiInstruction}
            setAiInstruction={setAiInstruction}
            generateProposal={generateProposal}
            aiLoading={aiLoading}
            proposal={proposal}
            setProposal={setProposal}
            applyProposal={applyProposal}
            agentMode={agentMode}
            setAgentMode={setAgentMode}
            agentSkillId={agentSkillId}
            setAgentSkillId={setAgentSkillId}
            agentRequest={agentRequest}
            setAgentRequest={setAgentRequest}
            agentSkills={agentSkills}
            agentRun={agentRun}
            agentLogs={agentLogs}
            agentLoading={agentLoading}
            runAgent={runAgent}
            applyAgentChange={applyAgentChange}
            rejectAgentRun={rejectAgentRun}
            claims={claims}
            claimText={claimText}
            setClaimText={setClaimText}
            addClaimRecord={addClaimRecord}
            exportJob={exportJob}
            exportRunning={exportRunning}
            exportWarnings={exportWarnings}
            runExport={runExport}
            exportProjectFolder={runProjectFolderExport}
            exportManifest={() => exportProjectManifest(activeProject)}
            openOutputFolder={openOutputFolder}
            openProjectFolder={openProjectFolder}
            updateProjectMetadata={(partial) => updateProjectMetadata(activeProject, partial)}
            combinedDraft={mergeSections(sections)}
            t={tr}
            language={settings.language}
          />
        </main>
      )}

      <AnimatePresence>
        {showCreateModal && (
          <CreateProjectModal
            form={projectForm}
            setForm={setProjectForm}
            onClose={() => setShowCreateModal(false)}
            onSubmit={createProject}
            t={tr}
          />
        )}
      </AnimatePresence>

      <AnimatePresence>
        {showImportModal && (
          <ImportProjectModal
            form={importForm}
            setForm={setImportForm}
            onClose={() => setShowImportModal(false)}
            onSubmit={importProject}
            t={tr}
          />
        )}
      </AnimatePresence>

      <AnimatePresence>
        {dialog && <AppDialog dialog={dialog} onClose={() => setDialog(null)} />}
      </AnimatePresence>
    </div>
  );
}

function TopBar({
  project,
  onDashboard,
  onNew,
  onSettings,
  version,
  t,
  language
}: {
  project: ProjectConfig | null;
  onDashboard: () => void;
  onNew: () => void;
  onSettings: () => void;
  version: string;
  t: Translate;
  language: Language;
}) {
  return (
    <header className="topbar">
      <button className="brand" onClick={onDashboard}>
        <Moon size={18} />
        <span>PaperForge</span>
        <small>v{version}</small>
      </button>
      <div className="topbar-project">{project ? `${displayTitle(project.title, language)} · ${project.manuscriptMode}` : t("app.tagline")}</div>
      <button className="secondary-btn" onClick={onSettings}>
        <Settings size={15} /> {t("tools.settings")}
      </button>
      <button className="primary-btn" onClick={onNew}>
        <Plus size={15} /> {t("actions.newProject")}
      </button>
    </header>
  );
}

function Dashboard({
  projects,
  onOpen,
  onNew,
  onImport,
  onInitWorkspace,
  onDelete,
  onEditTitle,
  workspaceRoot,
  t,
  language
}: {
  projects: ProjectConfig[];
  onOpen: (project: ProjectConfig) => void;
  onNew: () => void;
  onImport: () => void;
  onInitWorkspace: () => void;
  onDelete: (project: ProjectConfig) => void;
  onEditTitle: (project: ProjectConfig) => void;
  workspaceRoot: string;
  t: Translate;
  language: Language;
}) {
  return (
    <main className="dashboard">
      <section className="dashboard-hero">
        <p className="eyebrow">{t("app.researchIde")}</p>
        <h1>{t("dashboard.headline")}</h1>
        <button className="primary-btn large" onClick={onNew}>
          <Plus size={18} /> {t("actions.createProject")}
        </button>
        <button className="secondary-btn large" onClick={onImport}>
          <FolderOpen size={18} /> {t("actions.importExisting")}
        </button>
        <button className="secondary-btn large" onClick={onInitWorkspace}>
          <Folder size={18} /> {t("actions.initWorkspace")}
        </button>
        <small className="workspace-path">{workspaceRoot}</small>
      </section>
      <motion.section className="project-grid" initial="hidden" animate="show">
        {projects.length === 0 && <div className="empty-state">{t("dashboard.empty")}</div>}
        {projects.map((project, index) => (
          <motion.article
            className="project-card"
            key={project.id}
            variants={cardVariants}
            custom={index}
          >
            <button className="project-open" onClick={() => onOpen(project)}>
              <span className="mode-chip">{project.manuscriptMode}</span>
              <h2>{displayTitle(project.title, language)}</h2>
              <p>{project.authors?.length ? project.authors.join(", ") : t("app.noAuthors")}</p>
              <p>{project.targetJournal === "Unspecified Journal" ? t("app.noJournal") : project.targetJournal}</p>
              <small>{project.rootPath}</small>
            </button>
            <div className="project-actions">
              <button className="icon-action" onClick={() => onEditTitle(project)} title={t("actions.updateTitle")} aria-label={t("actions.updateTitle")}>
                <Pencil size={15} />
              </button>
              <button className="icon-action danger-action" onClick={() => onDelete(project)} title={t("actions.remove")} aria-label={t("actions.remove")}>
                <Trash2 size={15} />
              </button>
            </div>
          </motion.article>
        ))}
      </motion.section>
    </main>
  );
}

function Sidebar({
  project,
  sections,
  expandedFolders,
  setExpandedFolders,
  setActiveSectionId,
  onCreateSection,
  onRenameSection,
  onSettings,
  fileTree,
  fileTreeError,
  onRefreshFiles,
  onOpenTextFile,
  activeSectionId,
  activeFilePath,
  mode,
  onModeChange,
  t,
  language
}: {
  project: ProjectConfig;
  sections: ManuscriptSection[];
  expandedFolders: Record<string, boolean>;
  setExpandedFolders: (value: Record<string, boolean>) => void;
  setActiveSectionId: (id: string) => void;
  onCreateSection: () => void;
  onRenameSection: (section: ManuscriptSection) => void;
  onSettings: () => void;
  fileTree: FileTreeNode[];
  fileTreeError: string;
  onRefreshFiles: () => void;
  onOpenTextFile: (path: string, extension?: string) => void;
  activeSectionId: string;
  activeFilePath?: string;
  mode: SidebarMode;
  onModeChange: (mode: SidebarMode) => void;
  t: Translate;
  language: Language;
}) {
  return (
    <aside className="sidebar">
      <div className="sidebar-scroll">
        <div className="sidebar-title">
          <Folder size={16} /> {displayTitle(project.title, language)}
          {mode === "files" && (
            <button className="tree-icon-btn" onClick={onRefreshFiles} title="Refresh files">
              <RefreshCw size={13} />
            </button>
          )}
        </div>
        <div className="sidebar-tabs" role="tablist">
          <button
            className={mode === "writing" ? "sidebar-tab active" : "sidebar-tab"}
            onClick={() => onModeChange("writing")}
            role="tab"
            aria-selected={mode === "writing"}
            title="Writing mode: manuscript sections only"
          >
            <Pencil size={13} /> Writing
          </button>
          <button
            className={mode === "files" ? "sidebar-tab active" : "sidebar-tab"}
            onClick={() => onModeChange("files")}
            role="tab"
            aria-selected={mode === "files"}
            title="Files mode: full project tree"
          >
            <Folder size={13} /> Files
          </button>
        </div>
        {mode === "writing" && (
          <div className="sidebar-writing">
            <button className="tree-file new-section" onClick={onCreateSection}>
              <Plus size={14} /> {t("project.newSection")}
            </button>
            {sections.length === 0 && (
              <span className="tree-file muted">{t("project.emptyManuscript")}</span>
            )}
            {sections
              .slice()
              .sort((a, b) => a.order - b.order)
              .map((section, index) => {
                const isActive = activeSectionId === section.id;
                return (
                  <div key={section.id} className={isActive ? "section-pill active" : "section-pill"}>
                    <button
                      className="section-pill-main"
                      onClick={() => setActiveSectionId(section.id)}
                      title={section.path}
                    >
                      <span className="section-pill-num">{String(index + 1).padStart(2, "0")}</span>
                      <span className="section-pill-title">{section.title || "(untitled)"}</span>
                    </button>
                    <button
                      className="tree-icon-btn"
                      onClick={() => onRenameSection(section)}
                      title="Rename title; file path stays unchanged"
                    >
                      <Pencil size={12} />
                    </button>
                  </div>
                );
              })}
          </div>
        )}
        {mode === "files" && (
          <div className="sidebar-files">
            {fileTreeError && <div className="tree-error">{fileTreeError}</div>}
            {fileTree.map((node) => (
              <FileTreeItem
                key={node.relativePath}
                node={node}
                expandedFolders={expandedFolders}
                setExpandedFolders={setExpandedFolders}
                onOpenTextFile={onOpenTextFile}
                onSelectSection={(section) => {
                  setActiveSectionId(section.id);
                }}
                onRenameSection={onRenameSection}
                sectionByPath={new Map(sections.map((section) => [section.path, section]))}
                activeFilePath={activeFilePath}
              />
            ))}
          </div>
        )}
      </div>
      <button className="sidebar-settings" onClick={onSettings}>
        <Settings size={15} /> {t("project.settings")}
      </button>
    </aside>
  );
}

function FileTreeItem({
  node,
  expandedFolders,
  setExpandedFolders,
  onOpenTextFile,
  onSelectSection,
  onRenameSection,
  sectionByPath,
  activeFilePath,
  depth = 0
}: {
  node: FileTreeNode;
  expandedFolders: Record<string, boolean>;
  setExpandedFolders: (value: Record<string, boolean>) => void;
  onOpenTextFile: (path: string, extension?: string) => void;
  onSelectSection: (section: ManuscriptSection) => void;
  onRenameSection: (section: ManuscriptSection) => void;
  sectionByPath: Map<string, ManuscriptSection>;
  activeFilePath?: string;
  depth?: number;
}) {
  if (node.kind === "directory") {
    const open = expandedFolders[node.relativePath] ?? depth < 1;
    return (
      <div className="tree-block">
        <button className="tree-folder" style={{ paddingLeft: 8 + depth * 10 }} onClick={() => setExpandedFolders({ ...expandedFolders, [node.relativePath]: !open })}>
          <ChevronDown className={open ? "chev open" : "chev"} size={15} />
          <Folder size={14} /> {node.name}
        </button>
        <AnimatePresence initial={false}>
          {open && (
            <motion.div className="tree-children" initial={{ height: 0, opacity: 0 }} animate={{ height: "auto", opacity: 1 }} exit={{ height: 0, opacity: 0 }}>
              {(node.children ?? []).map((child) => (
                <FileTreeItem
                  key={child.relativePath}
                  node={child}
                  expandedFolders={expandedFolders}
                  setExpandedFolders={setExpandedFolders}
                  onOpenTextFile={onOpenTextFile}
                  onSelectSection={onSelectSection}
                  onRenameSection={onRenameSection}
                  sectionByPath={sectionByPath}
                  activeFilePath={activeFilePath}
                  depth={depth + 1}
                />
              ))}
            </motion.div>
          )}
        </AnimatePresence>
      </div>
    );
  }
  const section = sectionByPath.get(node.relativePath);
  const isText = isTextFile(node.name, node.extension);
  const active = activeFilePath === node.relativePath;
  const className = isText
    ? (active ? "tree-file text-file active" : "tree-file text-file")
    : (active ? "tree-file disabled active" : "tree-file disabled");
  return (
    <div className="tree-file-row">
      <button
        className={className}
        style={{ marginLeft: 12 + depth * 10 }}
        onClick={isText ? () => onOpenTextFile(node.relativePath, node.extension) : undefined}
        title={isText ? node.relativePath : "Binary file, not previewable"}
        disabled={!isText}
      >
        <FileText size={14} /> {section?.title ?? node.name}
      </button>
      {section && (
        <button className="tree-icon-btn" onClick={() => onRenameSection(section)} title="Rename title; file path stays unchanged">
          <Pencil size={13} />
        </button>
      )}
    </div>
  );
}
function RightPanel(props: {
  tab: ToolTab;
  setTab: (tab: ToolTab) => void;
  project: ProjectConfig;
  references: ReferenceItem[];
  bibtex: string;
  setBibtex: (value: string) => void;
  saveBibtex: () => void;
  insertCitation: (citekey: string) => void;
  citationTasks: CitationTask[];
  scanTasks: () => void;
  setTaskStatus: (taskId: string, status: CitationStatus) => void;
  literature: LiteratureItem[];
  litForm: { filename: string; path: string; linkedCitekey: string; notes: string };
  setLitForm: (value: { filename: string; path: string; linkedCitekey: string; notes: string }) => void;
  addLiterature: (event: FormEvent) => void;
  litQuery: string;
  setLitQuery: (value: string) => void;
  runLiteratureSearch: () => void;
  litSearching: boolean;
  aiInstruction: string;
  setAiInstruction: (value: string) => void;
  generateProposal: (instruction?: string) => void;
  aiLoading: boolean;
  proposal: AIProposal | null;
  setProposal: (proposal: AIProposal | null) => void;
  applyProposal: () => void;
  agentMode: AgentMode;
  setAgentMode: (mode: AgentMode) => void;
  agentSkillId: string;
  setAgentSkillId: (skillId: string) => void;
  agentRequest: string;
  setAgentRequest: (request: string) => void;
  agentSkills: AgentSkill[];
  agentRun: AgentRun | null;
  agentLogs: AgentLogEntry[];
  agentLoading: boolean;
  runAgent: () => void;
  applyAgentChange: (changeId: string) => void;
  rejectAgentRun: () => void;
  claims: ClaimRecord[];
  claimText: string;
  setClaimText: (value: string) => void;
  addClaimRecord: () => void;
  exportJob: ExportJob | null;
  exportRunning: boolean;
  exportWarnings: ExportValidationWarning[];
  runExport: (mode: ManuscriptMode) => void;
  exportManifest: () => void;
  exportProjectFolder: () => void;
  openOutputFolder: () => void;
  openProjectFolder: () => void;
  updateProjectMetadata: (partial: Partial<Pick<ProjectConfig, "title" | "author" | "authors" | "targetJournal" | "manuscriptMode" | "citationStyle" | "exportMode">>) => void;
  combinedDraft: string;
  t: Translate;
  language: Language;
}) {
  const tabs: Array<[ToolTab, string, ReactNode]> = [
    ["info", props.t("project.projectInfo"), <FileText size={14} />],
    ["agent", "Agent", <Brain size={14} />],
    ["references", "Refs", <BookOpen size={14} />],
    ["citations", "Cites", <Clipboard size={14} />],
    ["literature", "Library", <Library size={14} />],
    ["claims", "Claims", <Check size={14} />],
    ["export", props.t("tools.export"), <Download size={14} />]
  ];

  return (
    <aside className="right-panel">
      <div className="tab-strip">
        {tabs.map(([id, label, icon]) => (
          <button key={id} className={props.tab === id ? "tab active" : "tab"} onClick={() => props.setTab(id)}>
            {icon} {label}
          </button>
        ))}
      </div>
      <AnimatePresence mode="wait">
        <motion.div key={props.tab} variants={panelVariants} initial="hidden" animate="show" exit="exit" className="tool-body">
          {props.tab === "info" && <ProjectInfoTool {...props} />}
          {props.tab === "agent" && <AgentTool {...props} />}
          {props.tab === "references" && <ReferenceTool {...props} />}
          {props.tab === "citations" && <CitationTool {...props} />}
          {props.tab === "literature" && <LiteratureTool {...props} />}
          {props.tab === "claims" && <ClaimTool {...props} />}
          {props.tab === "export" && <ExportTool {...props} />}
        </motion.div>
      </AnimatePresence>
    </aside>
  );
}

function ProjectInfoTool(props: Parameters<typeof RightPanel>[0]) {
  return (
    <>
      <h2>{props.t("project.projectInfo")}</h2>
      <button className="secondary-btn wide" onClick={props.openProjectFolder}>
        <ExternalLink size={14} /> {props.t("actions.openProjectFolder")}
      </button>
      <div className="stack">
        <label>
          {props.t("modal.paperTitle")}
          <input
            value={props.project.title === "Untitled Paper" ? "" : props.project.title}
            placeholder={displayTitle(props.project.title, props.language)}
            onChange={(event) => props.updateProjectMetadata({ title: event.target.value })}
          />
        </label>
        <label>
          {props.t("modal.author")}
          <input
            value={props.project.authors?.join(", ") ?? props.project.author ?? ""}
            placeholder={props.t("app.noAuthors")}
            onChange={(event) => props.updateProjectMetadata({ author: event.target.value })}
          />
        </label>
        <label>
          {props.t("modal.journal")}
          <input
            value={props.project.targetJournal === "Unspecified Journal" ? "" : props.project.targetJournal}
            placeholder={props.t("app.noJournal")}
            onChange={(event) => props.updateProjectMetadata({ targetJournal: event.target.value })}
          />
        </label>
        <label>
          {props.t("settings.citationStyle")}
          <input value={props.project.citationStyle} onChange={(event) => props.updateProjectMetadata({ citationStyle: event.target.value })} />
        </label>
        <label>
          {props.t("settings.exportMode")}
          <select value={props.project.exportMode} onChange={(event) => props.updateProjectMetadata({ exportMode: event.target.value as ManuscriptMode })}>
            <option value="markdown">markdown</option>
            <option value="word">word</option>
            <option value="latex">latex</option>
          </select>
        </label>
      </div>
    </>
  );
}

function AgentTool(props: Parameters<typeof RightPanel>[0]) {
  const filteredSkills = props.agentSkills.filter((skill) => skill.type === props.agentMode);
  const pendingChanges = props.agentRun?.changes.filter((change) => change.status === "pending") ?? [];

  return (
    <>
      <h2>Agent Panel</h2>
      <div className="agent-summary">
        <span className="mode-chip">Project</span>
        <strong>{displayTitle(props.project.title, props.language)}</strong>
        <small>{props.project.rootPath}</small>
      </div>

      <div className="stack">
        <label>
          Mode
          <select value={props.agentMode} onChange={(event) => { props.setAgentMode(event.target.value as AgentMode); props.setAgentSkillId("auto"); }}>
            <option value="ask">Ask</option>
            <option value="edit">Edit</option>
            <option value="operate">Operate</option>
          </select>
        </label>
        <label>
          Skill
          <select value={props.agentSkillId} onChange={(event) => props.setAgentSkillId(event.target.value)}>
            <option value="auto">Auto</option>
            {filteredSkills.map((skill) => <option value={skill.id} key={skill.id}>{skill.name}</option>)}
          </select>
        </label>
        <label>
          User request
          <textarea className="small-area" value={props.agentRequest} onChange={(event) => props.setAgentRequest(event.target.value)} />
        </label>
      </div>

      <button className="primary-btn wide" disabled={props.agentLoading} onClick={props.runAgent}>
        {props.agentLoading ? <Loader2 className="spin" size={15} /> : <Sparkles size={15} />} Run Agent
      </button>

      {props.agentRun && (
        <div className="agent-run">
          <div className="proposal-card">
            <span className={`status ${props.agentRun.status}`}>{props.agentRun.status}</span>
            <strong>{props.agentRun.skillId}</strong>
            <p>{props.agentRun.report}</p>
          </div>

          <details open>
            <summary>Agent Plan</summary>
            <div className="agent-list">
              <strong>{props.agentRun.plan.summary}</strong>
              {props.agentRun.plan.steps.map((step) => <span key={step}>{step}</span>)}
            </div>
          </details>

          <details>
            <summary>Files Read</summary>
            <pre>{props.agentRun.filesRead.join("\n") || "None"}</pre>
          </details>

          <details open={pendingChanges.length > 0}>
            <summary>Files To Change</summary>
            <pre>{props.agentRun.plan.filesToChange.join("\n") || "None"}</pre>
          </details>

          {props.agentRun.changes.map((change) => (
            <div className="proposal-card" key={change.id}>
              <span className={`status ${change.status}`}>{change.status}</span>
              <strong>{change.path}</strong>
              <pre className="diff-preview">{change.diff}</pre>
              {change.status === "pending" && (
                <div className="row-actions">
                  <button onClick={() => props.applyAgentChange(change.id)}>Apply</button>
                  <button onClick={props.rejectAgentRun}>Reject</button>
                  <button onClick={() => navigator.clipboard?.writeText(change.diff)}>Copy diff</button>
                </div>
              )}
            </div>
          ))}
        </div>
      )}

      <details>
        <summary>Agent Log</summary>
        <div className="card-list">
          {props.agentLogs.length === 0 && <div className="validation-card info"><span>No Agent runs yet.</span></div>}
          {props.agentLogs.map((entry) => (
            <div className="task-card" key={entry.id}>
              <span className={`status ${entry.success ? "success" : "failed"}`}>{entry.success ? "success" : "failed"}</span>
              <strong>{entry.skillId}</strong>
              <p>{entry.request}</p>
              <small>{entry.mode} · {entry.createdAt}</small>
            </div>
          ))}
        </div>
      </details>
    </>
  );
}

function AiTool(props: Parameters<typeof RightPanel>[0]) {
  const quick = ["Rewrite selected paragraph", "Generate introduction from references", "Suggest citations", "Check unsupported claims", "Generate abstract"];
  return (
    <>
      <h2>AI Assistant</h2>
      <textarea className="small-area" value={props.aiInstruction} onChange={(event) => props.setAiInstruction(event.target.value)} />
      <div className="quick-grid">
        {quick.map((item) => (
          <button key={item} onClick={() => props.generateProposal(item)}>{item}</button>
        ))}
      </div>
      <button className="primary-btn wide" disabled={props.aiLoading} onClick={() => props.generateProposal()}>
        {props.aiLoading ? <Loader2 className="spin" size={15} /> : <Sparkles size={15} />} Generate proposal
      </button>
      {props.aiLoading && <div className="skeleton"><span /><span /><span /></div>}
      <AnimatePresence>
        {props.proposal && (
          <motion.div className="proposal-card" initial={{ opacity: 0, scale: 0.97 }} animate={{ opacity: 1, scale: 1 }} exit={{ opacity: 0 }}>
            <span className="mode-chip">LLM</span>
            <p>{props.proposal.proposedText}</p>
            <div className="row-actions">
              <button onClick={props.applyProposal}>Apply</button>
              <button onClick={() => props.setProposal({ ...props.proposal!, status: "accepted" })}>Accept</button>
              <button onClick={() => props.setProposal(null)}>Reject</button>
              <button onClick={() => navigator.clipboard?.writeText(props.proposal?.proposedText ?? "")}>Copy</button>
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </>
  );
}

function ReferenceTool(props: Parameters<typeof RightPanel>[0]) {
  return (
    <>
      <h2>Reference Manager</h2>
      <textarea className="bibtex-area" placeholder="@article{Zhang2023,...}" value={props.bibtex} onChange={(event) => props.setBibtex(event.target.value)} />
      <button className="primary-btn wide" onClick={props.saveBibtex}>Parse + save references.bib</button>
      <div className="card-list">
        {props.references.map((ref) => (
          <div className="reference-card" key={ref.citekey}>
            <strong>{ref.citekey}</strong>
            <p>{ref.title}</p>
            <small>{ref.authors.join(", ")} {ref.year}</small>
            <button onClick={() => props.insertCitation(ref.citekey)}>Insert {formatCitation(props.project.manuscriptMode, ref.citekey)}</button>
          </div>
        ))}
      </div>
    </>
  );
}

function CitationTool(props: Parameters<typeof RightPanel>[0]) {
  return (
    <>
      <h2>Citation Tasks</h2>
      <button className="primary-btn wide" onClick={props.scanTasks}>Scan Word placeholders</button>
      <div className="card-list">
        {props.citationTasks.map((task) => (
          <motion.div className="task-card" key={task.id} layout>
            <span className={`status ${task.status}`}>{task.status}</span>
            <strong>{task.placeholder}</strong>
            <p>{task.reference?.title ?? "No matched reference metadata"}</p>
            <div className="row-actions">
              <button onClick={() => props.setTaskStatus(task.id, "inserted")}>Mark inserted</button>
              <button onClick={() => navigator.clipboard?.writeText(task.citekey)}><Copy size={13} /> Key</button>
              <button onClick={() => navigator.clipboard?.writeText(task.reference?.title ?? "")}>Title</button>
              <button onClick={() => navigator.clipboard?.writeText(task.reference?.doi ?? "")}>DOI</button>
            </div>
          </motion.div>
        ))}
      </div>
    </>
  );
}

function LiteratureTool(props: Parameters<typeof RightPanel>[0]) {
  return (
    <>
      <h2>Literature Library</h2>
      <form className="stack" onSubmit={props.addLiterature}>
        <input placeholder="filename.pdf" value={props.litForm.filename} onChange={(event) => props.setLitForm({ ...props.litForm, filename: event.target.value })} />
        <input placeholder="local path" value={props.litForm.path} onChange={(event) => props.setLitForm({ ...props.litForm, path: event.target.value })} />
        <input placeholder="linked citekey" value={props.litForm.linkedCitekey} onChange={(event) => props.setLitForm({ ...props.litForm, linkedCitekey: event.target.value })} />
        <textarea className="small-area" placeholder="notes" value={props.litForm.notes} onChange={(event) => props.setLitForm({ ...props.litForm, notes: event.target.value })} />
        <button className="primary-btn wide">Add PDF record</button>
      </form>
      <div className="search-row">
        <input placeholder="Search literature notes" value={props.litQuery} onChange={(event) => props.setLitQuery(event.target.value)} />
        <button onClick={props.runLiteratureSearch}>{props.litSearching ? <Loader2 className="spin" size={15} /> : <Search size={15} />}</button>
      </div>
      <div className="card-list">
        {props.literature.map((item) => (
          <div className="reference-card" key={item.id}>
            <strong>{item.filename}</strong>
            <p>{item.notes}</p>
            <small>{item.embeddingStatus} · {item.linkedCitekey ?? "unlinked"}</small>
          </div>
        ))}
      </div>
    </>
  );
}

function ClaimTool(props: Parameters<typeof RightPanel>[0]) {
  return (
    <>
      <h2>Evidence Claims</h2>
      <textarea className="small-area" placeholder="Claim with citation syntax..." value={props.claimText} onChange={(event) => props.setClaimText(event.target.value)} />
      <button className="primary-btn wide" onClick={props.addClaimRecord}>Add claim</button>
      <div className="card-list">
        {props.claims.map((claim) => (
          <div className="task-card" key={claim.id}>
            <span className={`status ${claim.status}`}>{claim.status}</span>
            <p>{claim.claim}</p>
            <small>{claim.section} · {claim.citationKeys.join(", ") || "no citation"}</small>
          </div>
        ))}
      </div>
    </>
  );
}

function cleanExportPath(raw: string): string {
  if (!raw) return "";
  return raw
    .replace(/^\\\\\?\\/, "")
    .replace(/\\/g, "/");
}

function exportKindTitle(job: ExportJob | null): string {
  if (!job) return "Export";
  return "Export ready";
}

function ExportResult({ job, warnings, t, onOpenOutputFolder }: {
  job: ExportJob;
  warnings: ExportValidationWarning[];
  t: Translate;
  onOpenOutputFolder: () => void;
}) {
  const status = (job.status ?? "running").toLowerCase();
  const path = cleanExportPath(job.outputPath);
  const copyPath = async () => {
    try {
      if (navigator?.clipboard?.writeText && job.outputPath) {
        await navigator.clipboard.writeText(job.outputPath);
      }
    } catch {
      // ignore clipboard failures (e.g. window focus)
    }
  };
  const statusIcon = status === "success" ? <Check size={13} />
    : status === "failed" ? <X size={13} />
    : status === "warning" ? <AlertTriangle size={13} />
    : <Loader2 className="spin" size={13} />;
  return (
    <div className="export-result">
      <div className="export-result-head">
        <span className={`export-status-pill ${status}`}>
          {statusIcon}
          <span>{status}</span>
        </span>
        <span className="export-kind">{exportKindTitle(job)}</span>
      </div>
      {path ? (
        <div className="export-path">
          <code title={job.outputPath}>{path}</code>
          <div className="export-path-actions">
            <button className="icon-btn" onClick={copyPath} title="Copy path">
              <Copy size={13} />
            </button>
            <button className="secondary-btn" onClick={onOpenOutputFolder}>
              <ExternalLink size={13} /> {t("actions.openOutputFolder")}
            </button>
          </div>
        </div>
      ) : (
        <p className="export-path muted">{t("export.preparing")}</p>
      )}
      {warnings.length > 0 && (
        <div className="export-warnings">
          {warnings.map((warning) => (
            <div className={`validation-card ${warning.severity}`} key={warning.id}>
              {warning.severity === "error" ? <X size={14} /> :
                warning.severity === "warning" ? <AlertTriangle size={14} /> :
                <Check size={14} />}
              <div>
                <strong>{warning.severity}</strong>
                <span>{warning.message}</span>
              </div>
            </div>
          ))}
        </div>
      )}
      {job.logs.length > 0 && (
        <details className="export-logs">
          <summary>Details</summary>
          <pre>{job.logs.join("\n")}</pre>
        </details>
      )}
    </div>
  );
}

function ExportTool(props: Parameters<typeof RightPanel>[0]) {
  return (
    <>
      <h2>{props.t("tools.export")}</h2>
      <div className="quick-grid">
        <button className="export-primary" onClick={() => props.runExport("markdown")}>{props.t("export.markdownPackage")}</button>
        <button onClick={props.exportProjectFolder}>{props.t("export.projectFolder")}</button>
        <button onClick={() => props.runExport("word")}>{props.t("export.wordDraft")}</button>
        <button onClick={() => props.runExport("latex")}>{props.t("export.latexProject")}</button>
        <button onClick={props.exportManifest}>{props.t("export.manifestJson")}</button>
      </div>
      {props.exportRunning && <div className="running-dots">{props.t("export.running")}<span>.</span><span>.</span><span>.</span></div>}
      {props.exportJob && (
        <ExportResult
          job={props.exportJob}
          warnings={props.exportWarnings}
          t={props.t}
          onOpenOutputFolder={props.openOutputFolder}
        />
      )}
      <details>
        <summary>{props.t("export.combinedPreview")}</summary>
        <pre>{props.combinedDraft.slice(0, 1200)}</pre>
      </details>
    </>
  );
}
function SettingsPage({
  settings,
  setSettings,
  onBack,
  onTestConnection,
  onFetchModels,
  models,
  status,
  t
}: {
  settings: AppSettings;
  setSettings: (settings: AppSettings) => void;
  onBack: () => void;
  onTestConnection: () => void;
  onFetchModels: () => void;
  models: string[];
  status: string;
  t: Translate;
}) {
  const themeOptions: Array<[ThemeMode, string]> = [["light", "Light"], ["dark", "Dark"], ["system", "System"], ["eyeCare", "Eye-care"]];
  function updateLlmProvider(provider: LlmProviderKind) {
    const previous = settings.llmProvider;
    const baseUrl = provider === "anthropic" ? "https://api.anthropic.com/v1" : provider === "local" ? "http://localhost:11434/v1" : "https://api.openai.com/v1";
    const model = provider === "anthropic" ? "claude-3-5-sonnet-latest" : provider === "local" ? "llama3.1" : "gpt-4.1-mini";
    setSettings({
      ...settings,
      llmProvider: {
        ...previous,
        provider,
        baseUrl: previous.baseUrl.trim() === "" || previous.baseUrl.includes("api.openai.com") || previous.baseUrl.includes("api.anthropic.com") ? baseUrl : previous.baseUrl,
        model: previous.model.trim() === "" || previous.model === "gpt-4.1-mini" || previous.model.startsWith("claude-") ? model : previous.model
      }
    });
  }
  return (
    <main className="settings-page">
      <div className="settings-header">
        <div>
          <p className="eyebrow">{t("tools.settings")}</p>
          <h1>Settings</h1>
        </div>
        <button className="secondary-btn" onClick={onBack}>Back</button>
      </div>
      <div className="settings-grid">
        <section className="settings-card">
          <h2>Appearance</h2>
          <SettingField label={t("settings.colorTheme")} description="Applies immediately across the app.">
            <select value={settings.themeMode} onChange={(event) => setSettings({ ...settings, themeMode: event.target.value as ThemeMode })}>
              {themeOptions.map(([value, label]) => <option value={value} key={value}>{label}</option>)}
            </select>
          </SettingField>
          <SettingField label={t("settings.language")} description="Switches UI language immediately.">
            <select value={settings.language} onChange={(event) => setSettings({ ...settings, language: event.target.value as Language })}>
              <option value="en">{t("settings.english")}</option>
              <option value="zh">{t("settings.chinese")}</option>
            </select>
          </SettingField>
        </section>
        <section className="settings-card">
          <h2>Project Defaults</h2>
          <SettingField label={t("settings.workspaceRoot")} description="Default folder used for new PaperForge workspaces.">
            <input value={settings.workspaceRoot} onChange={(event) => setSettings({ ...settings, workspaceRoot: event.target.value })} />
          </SettingField>
          <SettingField label={t("settings.defaultMode")} description="Default writing/citation mode for new papers.">
            <select value={settings.defaultManuscriptMode} onChange={(event) => setSettings({ ...settings, defaultManuscriptMode: event.target.value as ManuscriptMode })}>
              <option value="word">word</option>
              <option value="latex">latex</option>
              <option value="markdown">markdown</option>
            </select>
          </SettingField>
          <SettingField label={t("settings.citationStyle")} description="Default citation style metadata.">
            <input value={settings.defaultCitationStyle} onChange={(event) => setSettings({ ...settings, defaultCitationStyle: event.target.value })} />
          </SettingField>
          <SettingField label={t("settings.exportMode")} description="Default export mode for new projects.">
            <select value={settings.defaultExportMode} onChange={(event) => setSettings({ ...settings, defaultExportMode: event.target.value as ManuscriptMode })}>
              <option value="markdown">markdown</option>
              <option value="word">word</option>
              <option value="latex">latex</option>
            </select>
          </SettingField>
        </section>
        <section className="settings-card wide-card">
          <h2>AI Provider</h2>
          <div className="settings-two-col">
            <SettingField label={t("settings.provider")} description="OpenAI-compatible is the main supported provider.">
              <select value={settings.llmProvider.provider} onChange={(event) => updateLlmProvider(event.target.value as LlmProviderKind)}>
                <option value="openai-compatible">OpenAI-compatible</option>
                <option value="openai">OpenAI</option>
                <option value="anthropic">Anthropic</option>
                <option value="local">Local</option>
              </select>
            </SettingField>
            <SettingField label={t("settings.baseUrl")} description="Provider API base URL, for example https://api.openai.com/v1.">
              <input value={settings.llmProvider.baseUrl} onChange={(event) => setSettings({ ...settings, llmProvider: { ...settings.llmProvider, baseUrl: event.target.value } })} />
            </SettingField>
            <SettingField label={t("settings.apiKey")} description="Stored locally. Never exported or committed.">
              <input type="password" value={settings.llmProvider.apiKey} onChange={(event) => setSettings({ ...settings, llmProvider: { ...settings.llmProvider, apiKey: event.target.value } })} />
            </SettingField>
            <SettingField label={t("settings.model")} description="Choose fetched model or type one manually.">
              <input list="ai-models" value={settings.llmProvider.model} onChange={(event) => setSettings({ ...settings, llmProvider: { ...settings.llmProvider, model: event.target.value } })} />
              <datalist id="ai-models">{models.map((model) => <option value={model} key={model} />)}</datalist>
            </SettingField>
          </div>
          <div className="row-actions">
            <button onClick={onFetchModels}>Fetch Models</button>
            <button onClick={onTestConnection}>Test Connection</button>
          </div>
          {status && <p className="settings-status">{status}</p>}
        </section>
        <section className="settings-card">
          <h2>Export</h2>
          <p>Markdown package remains the stable export target. Word and LaTeX use Pandoc draft exports and report warnings separately from success.</p>
        </section>
        <section className="settings-card">
          <h2>About</h2>
          <p>PaperForge v{APP_VERSION}</p>
          <p>Repository: github.com/YezhiHuan/PaperForge</p>
        </section>
      </div>
    </main>
  );
}

function SettingField({ label, description, children }: { label: string; description: string; children: ReactNode }) {
  return (
    <label className="setting-field">
      <span>{label}</span>
      <small>{description}</small>
      {children}
    </label>
  );
}

function StatusPanel({ logs, tasks, exportJob }: { logs: AppLog[]; tasks: CitationTask[]; exportJob: ExportJob | null }) {
  const pending = tasks.filter((task) => task.status === "pending").length;
  const latest = logs[0]?.message ?? "No app events yet.";
  return (
    <footer className="status-panel">
      <div className="status-card"><strong>Citation Queue</strong><span>{pending} pending Word placeholder(s)</span></div>
      <div className="status-card"><strong>Export</strong><span>{exportJob?.status ?? "idle"}</span></div>
      <div className="status-card activity"><strong>App Activity</strong><span>{latest}</span></div>
      <div className="log-strip" aria-label="Recent app logs">
        {logs.map((log) => <span className={`log ${log.level}`} key={log.id}>{log.message}</span>)}
      </div>
    </footer>
  );
}

function CreateProjectModal({
  form,
  setForm,
  onClose,
  onSubmit,
  t
}: {
  form: typeof emptyProjectForm;
  setForm: (form: typeof emptyProjectForm) => void;
  onClose: () => void;
  onSubmit: (event: FormEvent) => void;
  t: Translate;
}) {
  const setTemplate = (templateId: SectionTemplateId) => {
    const template = sectionTemplateOptions.find((item) => item.id === templateId) ?? sectionTemplateOptions[0];
    setForm({ ...form, sectionTemplate: templateId, sectionNames: template.sections });
  };
  const updateSectionName = (index: number, value: string) => {
    setForm({ ...form, sectionNames: form.sectionNames.map((name, itemIndex) => (itemIndex === index ? value : name)), sectionTemplate: "empty" });
  };
  const moveSection = (index: number, direction: -1 | 1) => {
    const nextIndex = index + direction;
    if (nextIndex < 0 || nextIndex >= form.sectionNames.length) return;
    const next = [...form.sectionNames];
    const [item] = next.splice(index, 1);
    next.splice(nextIndex, 0, item);
    setForm({ ...form, sectionNames: next, sectionTemplate: "empty" });
  };
  const removeSection = (index: number) => {
    setForm({ ...form, sectionNames: form.sectionNames.filter((_, itemIndex) => itemIndex !== index), sectionTemplate: "empty" });
  };

  return (
    <motion.div className="modal-backdrop" initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }}>
      <motion.form className="modal" onSubmit={onSubmit} initial={{ opacity: 0, scale: 0.96, y: 10 }} animate={{ opacity: 1, scale: 1, y: 0 }} exit={{ opacity: 0, scale: 0.96 }}>
        <button type="button" className="close-btn" onClick={onClose}><X size={16} /></button>
        <h2>{t("modal.createTitle")}</h2>
        <input placeholder={t("modal.paperTitle")} value={form.title} onChange={(event) => setForm({ ...form, title: event.target.value })} />
        <input placeholder={t("modal.author")} value={form.author} onChange={(event) => setForm({ ...form, author: event.target.value })} />
        <input placeholder={t("modal.journal")} value={form.targetJournal} onChange={(event) => setForm({ ...form, targetJournal: event.target.value })} />
        <select value={form.manuscriptMode} onChange={(event) => setForm({ ...form, manuscriptMode: event.target.value as ManuscriptMode })}>
          <option value="word">word</option>
          <option value="latex">latex</option>
          <option value="markdown">markdown</option>
        </select>
        <input placeholder={t("modal.workspaceRoot")} value={form.workspaceRoot} onChange={(event) => setForm({ ...form, workspaceRoot: event.target.value })} />
        <section className="section-builder">
          <div>
            <h3>{t("modal.sections")}</h3>
            <p className="modal-note">{t("modal.sectionNote")}</p>
          </div>
          <label>{t("modal.template")}
            <select value={form.sectionTemplate} onChange={(event) => setTemplate(event.target.value as SectionTemplateId)}>
              {sectionTemplateOptions.map((template) => <option value={template.id} key={template.id}>{template.label}</option>)}
            </select>
          </label>
          <label>{t("modal.naming")}
            <select value={form.sectionNaming} onChange={(event) => setForm({ ...form, sectionNaming: event.target.value as SectionNamingMode })}>
              <option value="numbered">numbered</option>
              <option value="slugOnly">slug only</option>
            </select>
          </label>
          <div className="section-list">
            {form.sectionNames.length === 0 && <div className="section-empty-note">{t("modal.emptySections")}</div>}
            {form.sectionNames.map((sectionName, index) => (
              <div className="section-row" key={`${index}-${sectionName}`}>
                <span>{String(index + 1).padStart(2, "0")}</span>
                <input placeholder="Section title" value={sectionName} onChange={(event) => updateSectionName(index, event.target.value)} />
                <button type="button" onClick={() => moveSection(index, -1)} disabled={index === 0}>Up</button>
                <button type="button" onClick={() => moveSection(index, 1)} disabled={index === form.sectionNames.length - 1}>Down</button>
                <button type="button" className="danger-action" onClick={() => removeSection(index)}><Trash2 size={13} /></button>
              </div>
            ))}
          </div>
          <button type="button" className="secondary-btn wide" onClick={() => setForm({ ...form, sectionNames: [...form.sectionNames, ""], sectionTemplate: "empty" })}>
            <Plus size={14} /> {t("actions.addCustomSection")}
          </button>
        </section>
        <button className="primary-btn wide">{t("modal.generate")}</button>
      </motion.form>
    </motion.div>
  );
}

function ImportProjectModal({
  form,
  setForm,
  onClose,
  onSubmit,
  t
}: {
  form: typeof emptyImportForm;
  setForm: (form: typeof emptyImportForm) => void;
  onClose: () => void;
  onSubmit: (event: FormEvent) => void;
  t: Translate;
}) {
  return (
    <motion.div className="modal-backdrop" initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }}>
      <motion.form className="modal" onSubmit={onSubmit} initial={{ opacity: 0, scale: 0.96, y: 10 }} animate={{ opacity: 1, scale: 1, y: 0 }} exit={{ opacity: 0, scale: 0.96 }}>
        <button type="button" className="close-btn" onClick={onClose}><X size={16} /></button>
        <h2>{t("modal.importTitle")}</h2>
        <p className="modal-note">{t("modal.importNote")}</p>
        <input required placeholder="F:\\Papers\\My_Project" value={form.rootPath} onChange={(event) => setForm({ rootPath: event.target.value })} />
        <button className="primary-btn wide"><FolderOpen size={15} /> {t("modal.importFolder")}</button>
      </motion.form>
    </motion.div>
  );
}

function AppDialog({ dialog, onClose }: { dialog: DialogState; onClose: () => void }) {
  const [value, setValue] = useState(dialog.kind === "input" ? dialog.defaultValue : "");
  const [error, setError] = useState("");
  const close = () => {
    if (dialog.kind === "confirm") dialog.resolve(false);
    if (dialog.kind === "input") dialog.resolve(null);
    if (dialog.kind === "message") dialog.resolve();
    onClose();
  };
  const confirm = () => {
    if (dialog.kind === "confirm") dialog.resolve(true);
    if (dialog.kind === "message") dialog.resolve();
    if (dialog.kind === "input") {
      const validation = dialog.validate?.(value);
      if (validation) {
        setError(validation);
        return;
      }
      dialog.resolve(value);
    }
    onClose();
  };
  return (
    <motion.div
      className="modal-backdrop"
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
      onMouseDown={close}
      onKeyDown={(event) => {
        if (event.key === "Escape") close();
        if (event.key === "Enter" && (event.ctrlKey || dialog.kind !== "input")) confirm();
      }}
    >
      <motion.div className="modal dialog-modal" onMouseDown={(event) => event.stopPropagation()} initial={{ opacity: 0, scale: 0.96, y: 10 }} animate={{ opacity: 1, scale: 1, y: 0 }} exit={{ opacity: 0, scale: 0.96 }}>
        <button type="button" className="close-btn" onClick={close}><X size={16} /></button>
        <h2>{dialog.title}</h2>
        <p className="modal-note">{dialog.description}</p>
        {dialog.kind === "input" && (
          <>
            <input autoFocus placeholder={dialog.placeholder} value={value} onChange={(event) => { setValue(event.target.value); setError(""); }} onKeyDown={(event) => { if (event.key === "Enter") confirm(); }} />
            {error && <span className="field-error">{error}</span>}
          </>
        )}
        <div className="dialog-actions">
          {dialog.kind !== "message" && <button className="secondary-btn" onClick={close}>Cancel</button>}
          <button className={dialog.kind === "confirm" && dialog.danger ? "primary-btn danger-confirm" : "primary-btn"} onClick={confirm}>
            {dialog.kind === "message" ? "OK" : dialog.confirmLabel}
          </button>
        </div>
      </motion.div>
    </motion.div>
  );
}

export default App;
