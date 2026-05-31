import { AnimatePresence, motion } from "framer-motion";
import {
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
import { createClaim, formatCitation, markdownToPreview, mergeSections, nowIso, sectionTemplateOptions } from "./lib/domain";
import { APP_VERSION } from "./version";
import type {
  AIProposal,
  AppLog,
  AppSettings,
  CitationStatus,
  CitationTask,
  ClaimRecord,
  ExportValidationWarning,
  ExportJob,
  LiteratureItem,
  ManuscriptMode,
  ManuscriptSection,
  ProjectConfig,
  ReferenceItem,
  SectionNamingMode,
  SectionTemplateId,
  ThemeMode
} from "./types";

type ToolTab = "ai" | "references" | "citations" | "literature" | "claims" | "export" | "settings";

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
  const [toolTab, setToolTab] = useState<ToolTab>("ai");
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [projectForm, setProjectForm] = useState(emptyProjectForm);
  const [showImportModal, setShowImportModal] = useState(false);
  const [importForm, setImportForm] = useState(emptyImportForm);
  const [expandedFolders, setExpandedFolders] = useState<Record<string, boolean>>({
    manuscript: true,
    references: true,
    literature: false,
    ai: false,
    outputs: false
  });
  const [bibtex, setBibtex] = useState("");
  const [aiInstruction, setAiInstruction] = useState("Rewrite selected paragraph");
  const [aiLoading, setAiLoading] = useState(false);
  const [proposal, setProposal] = useState<AIProposal | null>(null);
  const [litForm, setLitForm] = useState({ filename: "", path: "", linkedCitekey: "", notes: "" });
  const [litQuery, setLitQuery] = useState("");
  const [litSearching, setLitSearching] = useState(false);
  const [exportJob, setExportJob] = useState<ExportJob | null>(null);
  const [exportRunning, setExportRunning] = useState(false);
  const [exportWarnings, setExportWarnings] = useState<ExportValidationWarning[]>([]);
  const [draftSettings, setDraftSettings] = useState<AppSettings>(defaultSettings);
  const [claimText, setClaimText] = useState("");

  const activeSection = useMemo(
    () => sections.find((section) => section.id === activeSectionId) ?? sections[0],
    [activeSectionId, sections]
  );

  const addLog = (level: AppLog["level"], message: string) => {
    const log = api.appLog(level, message);
    setLogs((current) => [log, ...current].slice(0, 8));
    void api.appendAppLog(log).catch(() => undefined);
  };

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
    document.documentElement.dataset.theme = settings.themeMode;
  }, [settings.themeMode]);

  async function selectProject(project: ProjectConfig) {
    setActiveProject(project);
    const [loadedSections, loadedRefs, loadedTasks, loadedLit, loadedClaims] = await Promise.all([
      api.readSections(project.id),
      api.listReferences(project.id),
      api.scanCitationTasks(project.id),
      api.listLiterature(project.id),
      api.listClaims(project.id)
    ]);
    setSections(loadedSections);
    setActiveSectionId(loadedSections[0]?.id ?? "");
    setReferences(loadedRefs);
    setCitationTasks(loadedTasks);
    setLiterature(loadedLit);
    setClaims(loadedClaims);
    addLog("success", `Opened ${project.title}`);
  }

  async function createProject(event: FormEvent) {
    event.preventDefault();
    const project = await api.createProject({
      ...projectForm,
      workspaceRoot: projectForm.workspaceRoot || settings.workspaceRoot
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
    const ok = window.confirm(`Remove "${project.title}" from PaperForge project list? Local folder is not deleted.`);
    if (!ok) return;
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
    }
    addLog("warning", `Removed project from app list: ${project.title}. Local files kept.`);
  }

  async function exportProjectManifest(project: ProjectConfig) {
    const manifest = await api.exportProjectManifest(project.id);
    const blob = new Blob([manifest], { type: "application/json" });
    const url = URL.createObjectURL(blob);
    const link = document.createElement("a");
    link.href = url;
    link.download = `${project.title.replace(/[^\w.-]+/g, "_")}_paperforge_manifest.json`;
    link.click();
    URL.revokeObjectURL(url);
    addLog("success", `Exported project manifest: ${project.title}`);
  }

  async function saveActiveSection() {
    if (!activeProject || !activeSection) return;
    const saved = await api.saveSection(activeProject.id, activeSection);
    setSections((current) => current.map((section) => (section.id === saved.id ? { ...saved, updatedAt: nowIso() } : section)));
    addLog("success", `Saved ${activeSection.filename}`);
  }

  async function createSectionFromPrompt() {
    if (!activeProject) return;
    const title = window.prompt("Section title");
    if (!title?.trim()) return;
    const section = await api.createSection(activeProject.id, { title });
    const loadedSections = await api.readSections(activeProject.id);
    const updatedProject = await api.readProjectConfig(activeProject.id);
    setActiveProject(updatedProject);
    setProjects((current) => current.map((project) => (project.id === updatedProject.id ? updatedProject : project)));
    setSections(loadedSections);
    setActiveSectionId(section.id);
    addLog("success", `Created section: ${section.title}`);
  }

  async function renameSectionFromPrompt(section: ManuscriptSection) {
    if (!activeProject) return;
    const title = window.prompt("Rename section title. File path stays unchanged.", section.title);
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
    if (!activeSection) return;
    setSections((current) => current.map((section) => (section.id === activeSection.id ? { ...section, content } : section)));
  }

  function insertCitation(citekey: string) {
    if (!activeProject || !activeSection) return;
    const citation = formatCitation(activeProject.manuscriptMode, citekey);
    updateActiveSection(`${activeSection.content}${activeSection.content.endsWith(" ") ? "" : " "}${citation}`);
    addLog("info", `Inserted ${citation}`);
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
      addLog("info", `MOCK literature search returned ${results.length} item(s).`);
    }, 420);
  }

  async function generateProposal(instruction = aiInstruction) {
    if (!activeProject || !activeSection) return;
    setAiLoading(true);
    window.setTimeout(async () => {
      const generated = await api.generateAiProposal(activeProject.id, activeSection.id, instruction, "", settings);
      setProposal(generated);
      setAiLoading(false);
      addLog("info", "AI proposal generated as mock/provider abstraction.");
    }, 520);
  }

  async function applyProposal() {
    if (!activeProject || !activeSection || !proposal) return;
    const updated = await api.applyAiProposal(activeProject.id, proposal, activeSection);
    setSections((current) => current.map((section) => (section.id === updated.id ? updated : section)));
    setProposal({ ...proposal, status: "applied" });
    addLog("success", "AI proposal applied. Change recorded in app log, no Git commit.");
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
    window.setTimeout(async () => {
      const job = await api.exportProject(activeProject.id, mode, sections);
      setExportJob(job);
      setExportRunning(false);
      addLog("success", `${mode} export finished: ${job.outputPath}`);
    }, 620);
  }

  async function openOutputFolder() {
    if (!exportJob?.outputPath) return;
    const opened = await api.openOutputFolder(outputFolderFromPath(exportJob.outputPath));
    addLog(opened ? "info" : "warning", opened ? "Opened output folder." : "Output folder open unavailable in browser mode.");
  }

  async function saveSettings() {
    const saved = await api.saveSettings(draftSettings);
    setSettings(saved);
    addLog("success", "Settings saved. API key kept out of project.json.");
  }

  return (
    <div className="min-h-screen bg-[var(--bg)] text-[var(--text)]">
      <TopBar
        project={activeProject}
        onDashboard={() => setActiveProject(null)}
        onNew={openCreateModal}
        version={APP_VERSION}
      />

      {!activeProject ? (
        <Dashboard
          projects={projects}
          onOpen={selectProject}
          onNew={openCreateModal}
          onImport={() => setShowImportModal(true)}
          onDelete={deleteProject}
          onExport={exportProjectManifest}
        />
      ) : (
        <main className="workspace-grid">
          <Sidebar
            project={activeProject}
            sections={sections}
            activeSectionId={activeSection?.id}
            expandedFolders={expandedFolders}
            setExpandedFolders={setExpandedFolders}
            setActiveSectionId={setActiveSectionId}
            onCreateSection={createSectionFromPrompt}
            onRenameSection={renameSectionFromPrompt}
          />

          <section className="editor-shell">
            <div className="editor-toolbar">
              <div>
                <p className="eyebrow">Manuscript</p>
                <h1>{activeSection?.title ?? "No section"}</h1>
              </div>
              <div className="toolbar-actions">
                <button disabled={!activeSection} className={editorMode === "edit" ? "seg active" : "seg"} onClick={() => setEditorMode("edit")}>
                  Edit
                </button>
                <button disabled={!activeSection} className={editorMode === "preview" ? "seg active" : "seg"} onClick={() => setEditorMode("preview")}>
                  Preview
                </button>
                <button className="secondary-btn" onClick={createSectionFromPrompt}>
                  <Plus size={15} /> Create section
                </button>
                <button className="primary-btn" disabled={!activeSection} onClick={saveActiveSection}>
                  <Save size={15} /> Save
                </button>
              </div>
            </div>

            {activeSection ? (
              <AnimatePresence mode="wait">
                {editorMode === "edit" ? (
                  <motion.textarea
                    key="editor"
                    variants={panelVariants}
                    initial="hidden"
                    animate="show"
                    exit="exit"
                    className="manuscript-editor"
                    value={activeSection.content}
                    onChange={(event) => updateActiveSection(event.target.value)}
                  />
                ) : (
                  <motion.article
                    key="preview"
                    variants={panelVariants}
                    initial="hidden"
                    animate="show"
                    exit="exit"
                    className="preview"
                    dangerouslySetInnerHTML={{ __html: markdownToPreview(activeSection.content) }}
                  />
                )}
              </AnimatePresence>
            ) : (
              <motion.div className="manuscript-empty" variants={panelVariants} initial="hidden" animate="show">
                <FileText size={34} />
                <h2>No manuscript sections yet.</h2>
                <p>No manuscript sections yet. Create your first section to start writing.</p>
                <button className="primary-btn" onClick={createSectionFromPrompt}>
                  <Plus size={15} /> Create section
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
            claims={claims}
            claimText={claimText}
            setClaimText={setClaimText}
            addClaimRecord={addClaimRecord}
            exportJob={exportJob}
            exportRunning={exportRunning}
            exportWarnings={exportWarnings}
            runExport={runExport}
            openOutputFolder={openOutputFolder}
            settings={draftSettings}
            setSettings={setDraftSettings}
            saveSettings={saveSettings}
            combinedDraft={mergeSections(sections)}
          />

          <StatusPanel logs={logs} tasks={citationTasks} exportJob={exportJob} />
        </main>
      )}

      <AnimatePresence>
        {showCreateModal && (
          <CreateProjectModal
            form={projectForm}
            setForm={setProjectForm}
            onClose={() => setShowCreateModal(false)}
            onSubmit={createProject}
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
          />
        )}
      </AnimatePresence>
    </div>
  );
}

function TopBar({ project, onDashboard, onNew, version }: { project: ProjectConfig | null; onDashboard: () => void; onNew: () => void; version: string }) {
  return (
    <header className="topbar">
      <button className="brand" onClick={onDashboard}>
        <Moon size={18} />
        <span>PaperForge</span>
        <small>v{version}</small>
      </button>
      <div className="topbar-project">{project ? `${project.title} · ${project.manuscriptMode}` : "Local-first manuscript workspace"}</div>
      <button className="primary-btn" onClick={onNew}>
        <Plus size={15} /> New Project
      </button>
    </header>
  );
}

function Dashboard({
  projects,
  onOpen,
  onNew,
  onImport,
  onDelete,
  onExport
}: {
  projects: ProjectConfig[];
  onOpen: (project: ProjectConfig) => void;
  onNew: () => void;
  onImport: () => void;
  onDelete: (project: ProjectConfig) => void;
  onExport: (project: ProjectConfig) => void;
}) {
  return (
    <main className="dashboard">
      <section className="dashboard-hero">
        <p className="eyebrow">Research IDE</p>
        <h1>Write papers from local folders, citations, evidence, and AI proposals.</h1>
        <button className="primary-btn large" onClick={onNew}>
          <Plus size={18} /> Create Paper Project
        </button>
        <button className="secondary-btn large" onClick={onImport}>
          <FolderOpen size={18} /> Import Existing
        </button>
      </section>
      <motion.section className="project-grid" initial="hidden" animate="show">
        {projects.length === 0 && <div className="empty-state">No projects yet. Create one to generate workspace structure.</div>}
        {projects.map((project, index) => (
          <motion.article
            className="project-card"
            key={project.id}
            variants={cardVariants}
            custom={index}
          >
            <button className="project-open" onClick={() => onOpen(project)}>
              <span className="mode-chip">{project.manuscriptMode}</span>
              <h2>{project.title}</h2>
              <p>{project.author || "Unknown author"}</p>
              <small>{project.rootPath}</small>
            </button>
            <div className="project-actions">
              <button onClick={() => onExport(project)}><Download size={13} /> Export</button>
              <button className="danger-action" onClick={() => onDelete(project)}><X size={13} /> Remove</button>
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
  activeSectionId,
  expandedFolders,
  setExpandedFolders,
  setActiveSectionId,
  onCreateSection,
  onRenameSection
}: {
  project: ProjectConfig;
  sections: ManuscriptSection[];
  activeSectionId?: string;
  expandedFolders: Record<string, boolean>;
  setExpandedFolders: (value: Record<string, boolean>) => void;
  setActiveSectionId: (id: string) => void;
  onCreateSection: () => void;
  onRenameSection: (section: ManuscriptSection) => void;
}) {
  const folders = ["manuscript", "references", "literature", "figures", "data", "ai", "outputs"];
  return (
    <aside className="sidebar">
      <div className="sidebar-title">
        <Folder size={16} /> {project.title}
      </div>
      {folders.map((folder) => (
        <div key={folder} className="tree-block">
          <button
            className="tree-folder"
            onClick={() => setExpandedFolders({ ...expandedFolders, [folder]: !expandedFolders[folder] })}
          >
            <ChevronDown className={expandedFolders[folder] ? "chev open" : "chev"} size={15} />
            {folder}
          </button>
          <AnimatePresence initial={false}>
            {expandedFolders[folder] && (
              <motion.div className="tree-children" initial={{ height: 0, opacity: 0 }} animate={{ height: "auto", opacity: 1 }} exit={{ height: 0, opacity: 0 }}>
                {folder === "manuscript"
                  ? (
                    <>
                      <button className="tree-file new-section" onClick={onCreateSection}>
                        <Plus size={14} /> New Section
                      </button>
                      {sections.length === 0 && <span className="tree-file muted">Empty manuscript</span>}
                      {sections.map((section) => (
                        <div className={section.id === activeSectionId ? "tree-file-row active" : "tree-file-row"} key={section.id}>
                          <button
                            className="tree-file"
                            onClick={() => setActiveSectionId(section.id)}
                            title={`${section.title} · ${section.path}`}
                          >
                            <FileText size={14} /> {section.title}
                          </button>
                          <button className="tree-icon-btn" onClick={() => onRenameSection(section)} title="Rename title; file path stays unchanged">
                            <Pencil size={13} />
                          </button>
                        </div>
                      ))}
                    </>
                  )
                  : <span className="tree-file muted">{folder === "references" ? "references.bib" : folder === "ai" ? "claims.json" : "MVP folder"}</span>}
              </motion.div>
            )}
          </AnimatePresence>
        </div>
      ))}
    </aside>
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
  claims: ClaimRecord[];
  claimText: string;
  setClaimText: (value: string) => void;
  addClaimRecord: () => void;
  exportJob: ExportJob | null;
  exportRunning: boolean;
  exportWarnings: ExportValidationWarning[];
  runExport: (mode: ManuscriptMode) => void;
  openOutputFolder: () => void;
  settings: AppSettings;
  setSettings: (settings: AppSettings) => void;
  saveSettings: () => void;
  combinedDraft: string;
}) {
  const tabs: Array<[ToolTab, string, ReactNode]> = [
    ["ai", "AI", <Brain size={14} />],
    ["references", "Refs", <BookOpen size={14} />],
    ["citations", "Cites", <Clipboard size={14} />],
    ["literature", "Library", <Library size={14} />],
    ["claims", "Claims", <Check size={14} />],
    ["export", "Export", <Download size={14} />],
    ["settings", "Settings", <Settings size={14} />]
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
          {props.tab === "ai" && <AiTool {...props} />}
          {props.tab === "references" && <ReferenceTool {...props} />}
          {props.tab === "citations" && <CitationTool {...props} />}
          {props.tab === "literature" && <LiteratureTool {...props} />}
          {props.tab === "claims" && <ClaimTool {...props} />}
          {props.tab === "export" && <ExportTool {...props} />}
          {props.tab === "settings" && <SettingsTool {...props} />}
        </motion.div>
      </AnimatePresence>
    </aside>
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
            <span className="mode-chip">mock/provider</span>
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
        <input placeholder="mock search" value={props.litQuery} onChange={(event) => props.setLitQuery(event.target.value)} />
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

function ExportTool(props: Parameters<typeof RightPanel>[0]) {
  return (
    <>
      <h2>Export</h2>
      <div className="quick-grid">
        <button onClick={() => props.runExport("word")}>Word draft</button>
        <button onClick={() => props.runExport("latex")}>LaTeX</button>
        <button onClick={() => props.runExport("markdown")}>Markdown/Pandoc</button>
      </div>
      {props.exportRunning && <div className="running-dots">Export running<span>.</span><span>.</span><span>.</span></div>}
      {props.exportWarnings.length > 0 && (
        <div className="card-list">
          {props.exportWarnings.map((warning) => (
            <div className={`validation-card ${warning.severity}`} key={warning.id}>
              <strong>{warning.severity}</strong>
              <span>{warning.message}</span>
            </div>
          ))}
        </div>
      )}
      {props.exportJob && (
        <div className="proposal-card">
          <span className={`status ${props.exportJob.status}`}>{props.exportJob.status}</span>
          <strong>{props.exportJob.outputPath || "Preparing output"}</strong>
          {props.exportJob.logs.map((line) => <p key={line}>{line}</p>)}
          {props.exportJob.outputPath && (
            <button className="secondary-btn wide" onClick={props.openOutputFolder}>
              <ExternalLink size={14} /> Open output folder
            </button>
          )}
        </div>
      )}
      <details>
        <summary>Combined draft preview</summary>
        <pre>{props.combinedDraft.slice(0, 1200)}</pre>
      </details>
    </>
  );
}

function SettingsTool(props: Parameters<typeof RightPanel>[0]) {
  const themeOptions: Array<[ThemeMode, string]> = [["dark", "Dark"], ["light", "Light"], ["eyeCare", "Eye-care"]];
  return (
    <>
      <h2>Settings</h2>
      <div className="stack">
        <label>Workspace root<input value={props.settings.workspaceRoot} onChange={(event) => props.setSettings({ ...props.settings, workspaceRoot: event.target.value })} /></label>
        <label>Default mode
          <select value={props.settings.defaultManuscriptMode} onChange={(event) => props.setSettings({ ...props.settings, defaultManuscriptMode: event.target.value as ManuscriptMode })}>
            <option value="word">word</option>
            <option value="latex">latex</option>
            <option value="markdown">markdown</option>
          </select>
        </label>
        <label>Color theme
          <select value={props.settings.themeMode} onChange={(event) => props.setSettings({ ...props.settings, themeMode: event.target.value as ThemeMode })}>
            {themeOptions.map(([value, label]) => <option value={value} key={value}>{label}</option>)}
          </select>
        </label>
        <label>Base URL<input value={props.settings.llmProvider.baseUrl} onChange={(event) => props.setSettings({ ...props.settings, llmProvider: { ...props.settings.llmProvider, baseUrl: event.target.value } })} /></label>
        <label>API key<input type="password" value={props.settings.llmProvider.apiKey} onChange={(event) => props.setSettings({ ...props.settings, llmProvider: { ...props.settings.llmProvider, apiKey: event.target.value } })} /></label>
        <label>Model<input value={props.settings.llmProvider.model} onChange={(event) => props.setSettings({ ...props.settings, llmProvider: { ...props.settings.llmProvider, model: event.target.value } })} /></label>
        <label>Citation style<input value={props.settings.defaultCitationStyle} onChange={(event) => props.setSettings({ ...props.settings, defaultCitationStyle: event.target.value })} /></label>
        <button className="primary-btn wide" onClick={props.saveSettings}>Save settings</button>
      </div>
    </>
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
  onSubmit
}: {
  form: typeof emptyProjectForm;
  setForm: (form: typeof emptyProjectForm) => void;
  onClose: () => void;
  onSubmit: (event: FormEvent) => void;
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
        <h2>Create paper project</h2>
        <input required placeholder="Paper title" value={form.title} onChange={(event) => setForm({ ...form, title: event.target.value })} />
        <input placeholder="Author" value={form.author} onChange={(event) => setForm({ ...form, author: event.target.value })} />
        <input placeholder="Target journal" value={form.targetJournal} onChange={(event) => setForm({ ...form, targetJournal: event.target.value })} />
        <select value={form.manuscriptMode} onChange={(event) => setForm({ ...form, manuscriptMode: event.target.value as ManuscriptMode })}>
          <option value="word">word</option>
          <option value="latex">latex</option>
          <option value="markdown">markdown</option>
        </select>
        <input placeholder="Workspace root (optional)" value={form.workspaceRoot} onChange={(event) => setForm({ ...form, workspaceRoot: event.target.value })} />
        <section className="section-builder">
          <div>
            <h3>Manuscript Sections / 论文章节</h3>
            <p className="modal-note">Default is Empty manuscript. Blank section names are ignored.</p>
          </div>
          <label>Template
            <select value={form.sectionTemplate} onChange={(event) => setTemplate(event.target.value as SectionTemplateId)}>
              {sectionTemplateOptions.map((template) => <option value={template.id} key={template.id}>{template.label}</option>)}
            </select>
          </label>
          <label>Section file naming
            <select value={form.sectionNaming} onChange={(event) => setForm({ ...form, sectionNaming: event.target.value as SectionNamingMode })}>
              <option value="numbered">numbered</option>
              <option value="slugOnly">slug only</option>
            </select>
          </label>
          <div className="section-list">
            {form.sectionNames.length === 0 && <div className="section-empty-note">No sections selected. Project will start with an empty manuscript.</div>}
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
            <Plus size={14} /> Add custom section
          </button>
        </section>
        <button className="primary-btn wide">Generate local folder structure</button>
      </motion.form>
    </motion.div>
  );
}

function ImportProjectModal({
  form,
  setForm,
  onClose,
  onSubmit
}: {
  form: typeof emptyImportForm;
  setForm: (form: typeof emptyImportForm) => void;
  onClose: () => void;
  onSubmit: (event: FormEvent) => void;
}) {
  return (
    <motion.div className="modal-backdrop" initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }}>
      <motion.form className="modal" onSubmit={onSubmit} initial={{ opacity: 0, scale: 0.96, y: 10 }} animate={{ opacity: 1, scale: 1, y: 0 }} exit={{ opacity: 0, scale: 0.96 }}>
        <button type="button" className="close-btn" onClick={onClose}><X size={16} /></button>
        <h2>Import existing project</h2>
        <p className="modal-note">Enter a PaperForge project folder path. Existing manuscripts are not overwritten.</p>
        <input required placeholder="F:\\Papers\\My_Project" value={form.rootPath} onChange={(event) => setForm({ rootPath: event.target.value })} />
        <button className="primary-btn wide"><FolderOpen size={15} /> Import folder</button>
      </motion.form>
    </motion.div>
  );
}

export default App;
