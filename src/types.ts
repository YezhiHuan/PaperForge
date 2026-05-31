export type ManuscriptMode = "word" | "latex" | "markdown";
export type CitationBackend = "zotero_word_plugin" | "bibtex" | "pandoc";
export type CitationStatus = "pending" | "inserted" | "ignored";
export type EmbeddingStatus = "not_indexed" | "indexed" | "failed";
export type ProposalStatus = "pending" | "accepted" | "rejected" | "applied";
export type ClaimStatus = "verified" | "needs_citation" | "unsupported";
export type ExportStatus = "pending" | "running" | "success" | "failed";
export type AppLogLevel = "info" | "warning" | "error" | "success";
export type ThemeMode = "dark" | "light" | "eyeCare";
export type ExportWarningSeverity = "info" | "warning" | "error";
export type SectionNamingMode = "numbered" | "slugOnly";
export type SectionTemplateId = "empty" | "standard" | "engineeringSimulation" | "review";
export type SectionStatus = "draft" | "review" | "done";
export type ProjectActivityType = "section.created" | "section.renamed" | "section.updated";

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
  title: string;
  author: string;
  targetJournal: string;
  manuscriptMode: ManuscriptMode;
  rootPath: string;
  createdAt: string;
  updatedAt: string;
  citationBackend: CitationBackend;
  manuscript: ManuscriptManifest;
}

export interface ProjectCreateInput {
  title: string;
  author: string;
  targetJournal: string;
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
  baseUrl: string;
  apiKey: string;
  model: string;
}

export interface AppSettings {
  workspaceRoot: string;
  defaultManuscriptMode: ManuscriptMode;
  llmProvider: LlmProviderSettings;
  defaultCitationStyle: string;
  defaultExportMode: ManuscriptMode;
  themeMode: ThemeMode;
}

export interface FileTreeNode {
  name: string;
  path: string;
  kind: "file" | "folder";
  children?: FileTreeNode[];
}
