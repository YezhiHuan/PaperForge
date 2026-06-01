export type ManuscriptMode = "word" | "latex" | "markdown";
export type CitationBackend = "zotero_word_plugin" | "bibtex" | "pandoc";
export type CitationStatus = "pending" | "inserted" | "ignored";
export type EmbeddingStatus = "not_indexed" | "indexed" | "failed";
export type ProposalStatus = "pending" | "accepted" | "rejected" | "applied";
export type ClaimStatus = "verified" | "needs_citation" | "unsupported";
export type ExportStatus = "pending" | "running" | "success" | "failed";
export type AppLogLevel = "info" | "warning" | "error" | "success";
export type ThemeMode = "light" | "dark" | "system" | "eyeCare";
export type Language = "en" | "zh";
export type LlmProviderKind = "openai-compatible" | "openai" | "anthropic" | "local";
export type ExportWarningSeverity = "info" | "warning" | "error";
export type SectionNamingMode = "numbered" | "slugOnly";
export type SectionTemplateId = "empty" | "standard" | "engineeringSimulation" | "review";
export type SectionStatus = "draft" | "review" | "done";
export type ProjectActivityType = "section.created" | "section.renamed" | "section.updated";
export type AgentMode = "ask" | "edit" | "operate";
export type AgentRunStatus = "planned" | "completed" | "applied" | "rejected" | "failed";
export type AgentChangeStatus = "pending" | "applied" | "rejected";
export type AgentSkillType = AgentMode;

export interface ManuscriptManifestSection {
  id: string;
  title: string;
  path: string;
  order: number;
  status: SectionStatus;
  createdAt: string;
  updatedAt: string;
}

export interface ManuscriptManifest {
  sectionNaming: SectionNamingMode;
  sections: ManuscriptManifestSection[];
}

export interface ProjectConfig {
  id: string;
  version: string;
  title: string;
  author: string;
  authors: string[];
  targetJournal: string;
  journal?: string;
  language: Language;
  citationStyle: string;
  exportMode: ManuscriptMode;
  manuscriptMode: ManuscriptMode;
  rootPath: string;
  createdAt: string;
  updatedAt: string;
  citationBackend: CitationBackend;
  manuscript: ManuscriptManifest;
  sections: ManuscriptManifestSection[];
}

export interface ProjectCreateInput {
  title: string;
  author: string;
  targetJournal: string;
  citationStyle?: string;
  exportMode?: ManuscriptMode;
  manuscriptMode: ManuscriptMode;
  workspaceRoot?: string;
  sectionNaming: SectionNamingMode;
  sectionNames: string[];
}

export interface ProjectImportInput {
  rootPath: string;
}

export interface ManuscriptSection {
  id: string;
  filename: string;
  title: string;
  order: number;
  content: string;
  updatedAt: string;
  path: string;
  status: SectionStatus;
  createdAt: string;
}

export interface SectionCreateInput {
  title: string;
}

export interface SectionRenameInput {
  sectionId: string;
  title: string;
}

export interface ReferenceItem {
  citekey: string;
  title: string;
  authors: string[];
  year: string;
  journal: string;
  doi: string;
  abstract?: string;
  zoteroItemKey?: string;
  libraryId?: string;
  pdfPath?: string;
}

export interface CitationTask {
  id: string;
  sectionId: string;
  placeholder: string;
  citekey: string;
  status: CitationStatus;
  reference?: ReferenceItem;
}

export interface LiteratureItem {
  id: string;
  filename: string;
  path: string;
  linkedCitekey?: string;
  notes: string;
  abstract?: string;
  embeddingStatus: EmbeddingStatus;
}

export interface AIProposal {
  id: string;
  sectionId: string;
  instruction: string;
  originalText: string;
  proposedText: string;
  citationKeys: string[];
  createdAt: string;
  status: ProposalStatus;
}

export interface AgentSkill {
  id: string;
  name: string;
  type: AgentSkillType;
  description: string;
  allowedTools: string[];
  requiresDiff: boolean;
  requiresConfirmation: boolean;
  writesFiles: boolean;
  riskLevel: "low" | "medium" | "high";
}

export interface AgentPlan {
  summary: string;
  steps: string[];
  filesToRead: string[];
  filesToChange: string[];
}

export interface AgentFileChange {
  id: string;
  path: string;
  changeType: "create" | "update";
  originalContent: string;
  proposedContent: string;
  diff: string;
  status: AgentChangeStatus;
}

export interface AgentToolResult {
  tool: string;
  ok: boolean;
  message: string;
  data?: unknown;
  error?: string;
  reason?: string;
}

export interface AgentRun {
  id: string;
  projectId: string;
  mode: AgentMode;
  skillId: string;
  request: string;
  status: AgentRunStatus;
  plan: AgentPlan;
  filesRead: string[];
  filesChanged: string[];
  report: string;
  changes: AgentFileChange[];
  toolResults: AgentToolResult[];
  createdAt: string;
  updatedAt: string;
}

export interface AgentLogEntry {
  id: string;
  runId: string;
  projectId: string;
  mode: AgentMode;
  skillId: string;
  request: string;
  tools: string[];
  filesRead: string[];
  filesChanged: string[];
  success: boolean;
  error?: string;
  createdAt: string;
}

export interface ClaimRecord {
  id: string;
  section: string;
  claim: string;
  citationKeys: string[];
  evidenceChunkIds: string[];
  status: ClaimStatus;
}

export interface ExportJob {
  id: string;
  projectId: string;
  mode: ManuscriptMode;
  status: ExportStatus;
  outputPath: string;
  logs: string[];
  createdAt: string;
}

export interface ExportValidationWarning {
  id: string;
  severity: ExportWarningSeverity;
  message: string;
}

export interface AppLog {
  id: string;
  level: AppLogLevel;
  message: string;
  createdAt: string;
}

export interface ProjectActivity {
  id: string;
  type: ProjectActivityType;
  message: string;
  sectionId?: string;
  createdAt: string;
}

export interface LlmProviderSettings {
  provider: LlmProviderKind;
  baseUrl: string;
  apiKey: string;
  model: string;
  temperature: number;
  maxTokens: number;
}

export interface AppSettings {
  workspaceRoot: string;
  defaultManuscriptMode: ManuscriptMode;
  llmProvider: LlmProviderSettings;
  defaultCitationStyle: string;
  defaultExportMode: ManuscriptMode;
  themeMode: ThemeMode;
  language: Language;
  sidebarMode?: SidebarMode;
}

export type SidebarMode = "writing" | "files";

export interface WorkspaceConfig {
  version: string;
  workspaceName: string;
  createdAt: string;
  updatedAt: string;
  papersDir: string;
  defaultLanguage: Language;
}

export interface FileTreeNode {
  name: string;
  path: string;
  relativePath: string;
  kind: "file" | "directory";
  extension?: string;
  children?: FileTreeNode[];
}

export interface TextFilePayload {
  path: string;
  content: string;
}
