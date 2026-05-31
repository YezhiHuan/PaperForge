use chrono::Utc;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProjectCreateInput {
    title: String,
    author: String,
    target_journal: String,
    manuscript_mode: ManuscriptMode,
    citation_style: Option<String>,
    export_mode: Option<ManuscriptMode>,
    workspace_root: Option<String>,
    section_naming: SectionNamingMode,
    section_names: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProjectConfig {
    id: String,
    #[serde(default = "default_project_title")]
    title: String,
    #[serde(default)]
    author: String,
    #[serde(default)]
    authors: Vec<String>,
    #[serde(default = "default_target_journal")]
    target_journal: String,
    #[serde(default = "default_citation_style")]
    citation_style: String,
    #[serde(default = "default_export_mode")]
    export_mode: ManuscriptMode,
    manuscript_mode: ManuscriptMode,
    root_path: String,
    created_at: String,
    updated_at: String,
    citation_backend: CitationBackend,
    #[serde(default = "default_manuscript_manifest")]
    manuscript: ManuscriptManifest,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct ProjectMetadataPatch {
    title: Option<String>,
    author: Option<String>,
    authors: Option<Vec<String>>,
    target_journal: Option<String>,
    manuscript_mode: Option<ManuscriptMode>,
    citation_style: Option<String>,
    export_mode: Option<ManuscriptMode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ManuscriptManifest {
    section_naming: SectionNamingMode,
    sections: Vec<ManuscriptManifestSection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ManuscriptManifestSection {
    id: String,
    title: String,
    path: String,
    order: u32,
    status: SectionStatus,
    created_at: String,
    updated_at: String,
}

fn default_manuscript_manifest() -> ManuscriptManifest {
    ManuscriptManifest {
        section_naming: SectionNamingMode::Numbered,
        sections: vec![],
    }
}

fn default_project_title() -> String {
    "Untitled Paper".to_string()
}

fn default_target_journal() -> String {
    "Unspecified Journal".to_string()
}

fn default_citation_style() -> String {
    "apa".to_string()
}

fn default_export_mode() -> ManuscriptMode {
    ManuscriptMode::Markdown
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum ManuscriptMode {
    Word,
    Latex,
    Markdown,
}

impl Default for ManuscriptMode {
    fn default() -> Self {
        ManuscriptMode::Markdown
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum CitationBackend {
    ZoteroWordPlugin,
    Bibtex,
    Pandoc,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
enum SectionNamingMode {
    Numbered,
    SlugOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum SectionStatus {
    Draft,
    Review,
    Done,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ManuscriptSection {
    id: String,
    filename: String,
    title: String,
    order: u32,
    content: String,
    updated_at: String,
    path: String,
    status: SectionStatus,
    created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SectionCreateInput {
    title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SectionRenameInput {
    section_id: String,
    title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReferenceItem {
    citekey: String,
    title: String,
    authors: Vec<String>,
    year: String,
    journal: String,
    doi: String,
    #[serde(rename = "abstract")]
    #[serde(skip_serializing_if = "Option::is_none")]
    abstract_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    zotero_item_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    library_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pdf_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CitationTask {
    id: String,
    section_id: String,
    placeholder: String,
    citekey: String,
    status: CitationStatus,
    reference: Option<ReferenceItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum CitationStatus {
    Pending,
    Inserted,
    Ignored,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LiteratureItem {
    id: String,
    filename: String,
    path: String,
    linked_citekey: Option<String>,
    notes: String,
    #[serde(rename = "abstract")]
    abstract_text: Option<String>,
    embedding_status: EmbeddingStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum EmbeddingStatus {
    NotIndexed,
    Indexed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LiteratureItemInput {
    filename: String,
    path: String,
    linked_citekey: Option<String>,
    notes: String,
    #[serde(rename = "abstract")]
    abstract_text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ClaimRecord {
    id: String,
    section: String,
    claim: String,
    citation_keys: Vec<String>,
    evidence_chunk_ids: Vec<String>,
    status: ClaimStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ClaimStatus {
    Verified,
    NeedsCitation,
    Unsupported,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AiProposal {
    id: String,
    section_id: String,
    instruction: String,
    original_text: String,
    proposed_text: String,
    citation_keys: Vec<String>,
    created_at: String,
    status: ProposalStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum ProposalStatus {
    Pending,
    Accepted,
    Rejected,
    Applied,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExportJob {
    id: String,
    project_id: String,
    mode: ManuscriptMode,
    status: ExportStatus,
    output_path: String,
    logs: Vec<String>,
    created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum ExportStatus {
    Pending,
    Running,
    Success,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum AppLogLevel {
    Info,
    Warning,
    Error,
    Success,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AppLog {
    id: String,
    level: AppLogLevel,
    message: String,
    created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum ProjectActivityType {
    #[serde(rename = "section.created")]
    SectionCreated,
    #[serde(rename = "section.renamed")]
    SectionRenamed,
    #[serde(rename = "section.updated")]
    SectionUpdated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProjectActivity {
    id: String,
    #[serde(rename = "type")]
    activity_type: ProjectActivityType,
    message: String,
    section_id: Option<String>,
    created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
enum ThemeMode {
    Dark,
    Light,
    EyeCare,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Language {
    En,
    Zh,
}

fn default_language() -> Language {
    Language::En
}

fn default_theme_mode() -> ThemeMode {
    ThemeMode::Dark
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LlmProviderSettings {
    base_url: String,
    api_key: String,
    model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AppSettings {
    workspace_root: String,
    default_manuscript_mode: ManuscriptMode,
    llm_provider: LlmProviderSettings,
    default_citation_style: String,
    default_export_mode: ManuscriptMode,
    #[serde(default = "default_theme_mode")]
    theme_mode: ThemeMode,
    #[serde(default = "default_language")]
    language: Language,
}

fn now_iso() -> String {
    Utc::now().to_rfc3339()
}

fn app_root() -> Result<PathBuf, String> {
    std::env::current_dir().map_err(|err| err.to_string())
}

fn local_dir() -> Result<PathBuf, String> {
    let path = app_root()?.join(".local");
    fs::create_dir_all(&path).map_err(|err| err.to_string())?;
    Ok(path)
}

fn registry_path() -> Result<PathBuf, String> {
    Ok(local_dir()?.join("projects.json"))
}

fn settings_path() -> Result<PathBuf, String> {
    Ok(local_dir()?.join("settings.json"))
}

fn app_logs_path() -> Result<PathBuf, String> {
    Ok(local_dir()?.join("app_logs.json"))
}

fn safe_folder_name(title: &str) -> String {
    let cleaned: String = title
        .chars()
        .filter(|ch| {
            !matches!(ch, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*') && !ch.is_control()
        })
        .collect();
    let collapsed = cleaned.split_whitespace().collect::<Vec<_>>().join("_");
    if collapsed.is_empty() {
        "Paper_Project".to_string()
    } else {
        collapsed
    }
}

fn citation_backend(mode: &ManuscriptMode) -> CitationBackend {
    match mode {
        ManuscriptMode::Word => CitationBackend::ZoteroWordPlugin,
        ManuscriptMode::Latex => CitationBackend::Bibtex,
        ManuscriptMode::Markdown => CitationBackend::Pandoc,
    }
}

fn default_settings() -> AppSettings {
    AppSettings {
        workspace_root: "workspace".to_string(),
        default_manuscript_mode: ManuscriptMode::Word,
        llm_provider: LlmProviderSettings {
            base_url: "https://api.openai.com/v1".to_string(),
            api_key: String::new(),
            model: "gpt-4.1-mini".to_string(),
        },
        default_citation_style: "apa".to_string(),
        default_export_mode: ManuscriptMode::Markdown,
        theme_mode: ThemeMode::Dark,
        language: Language::En,
    }
}

fn read_registry() -> Result<Vec<ProjectConfig>, String> {
    let path = registry_path()?;
    if !path.exists() {
        return Ok(vec![]);
    }
    let raw = fs::read_to_string(path).map_err(|err| err.to_string())?;
    let projects: Vec<ProjectConfig> = serde_json::from_str(&raw).map_err(|err| err.to_string())?;
    Ok(projects.into_iter().map(normalize_project).collect())
}

fn write_registry(projects: &[ProjectConfig]) -> Result<(), String> {
    fs::write(
        registry_path()?,
        serde_json::to_string_pretty(projects).map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())
}

fn project_by_id(project_id: &str) -> Result<ProjectConfig, String> {
    read_registry()?
        .into_iter()
        .find(|project| project.id == project_id)
        .ok_or_else(|| "Project not found".to_string())
}

fn normalize_project(mut project: ProjectConfig) -> ProjectConfig {
    if project.title.trim().is_empty() {
        project.title = default_project_title();
    } else {
        project.title = project.title.trim().to_string();
    }
    project.author = project.author.trim().to_string();
    if project.authors.is_empty() && !project.author.is_empty() {
        project.authors = project
            .author
            .split(',')
            .map(|item| item.trim().to_string())
            .filter(|item| !item.is_empty())
            .collect();
    }
    if project.target_journal.trim().is_empty() {
        project.target_journal = default_target_journal();
    } else {
        project.target_journal = project.target_journal.trim().to_string();
    }
    if project.citation_style.trim().is_empty() {
        project.citation_style = default_citation_style();
    }
    project
}

fn project_manifest_path(root: &Path) -> PathBuf {
    root.join("paperforge.project.json")
}

fn legacy_project_manifest_path(root: &Path) -> PathBuf {
    root.join("project.json")
}

fn activity_path(root: &Path) -> PathBuf {
    root.join("logs/activity.json")
}

fn slugify_title(title: &str) -> String {
    let mut slug = String::new();
    let mut last_dash = false;
    for ch in title.trim().to_lowercase().chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch);
            last_dash = false;
        } else if !last_dash && !slug.is_empty() {
            slug.push('-');
            last_dash = true;
        }
    }
    slug.trim_matches('-').to_string()
}

fn make_section_filename(
    title: &str,
    index: usize,
    naming: &SectionNamingMode,
    existing: &mut Vec<String>,
) -> String {
    let fallback = match naming {
        SectionNamingMode::Numbered => "section".to_string(),
        SectionNamingMode::SlugOnly => format!("section-{:03}", index),
    };
    let slug = {
        let value = slugify_title(title);
        if value.is_empty() {
            fallback
        } else {
            value
        }
    };
    let base = match naming {
        SectionNamingMode::Numbered => format!("{:02}_{}", index, slug),
        SectionNamingMode::SlugOnly => slug,
    };
    let mut filename = format!("{}.md", base);
    let mut suffix = 2;
    while existing.iter().any(|item| item == &filename) {
        filename = format!("{}_{}.md", base, suffix);
        suffix += 1;
    }
    existing.push(filename.clone());
    filename
}

fn section_from_manifest(
    project: &ProjectConfig,
    manifest: &ManuscriptManifestSection,
) -> Result<ManuscriptSection, String> {
    let root = PathBuf::from(&project.root_path);
    let path = root.join(&manifest.path);
    let content = if path.exists() {
        fs::read_to_string(&path).map_err(|err| err.to_string())?
    } else {
        String::new()
    };
    let filename = PathBuf::from(&manifest.path)
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("section.md")
        .to_string();
    Ok(ManuscriptSection {
        id: manifest.id.clone(),
        filename,
        title: manifest.title.clone(),
        order: manifest.order,
        content,
        updated_at: manifest.updated_at.clone(),
        path: manifest.path.clone(),
        status: manifest.status.clone(),
        created_at: manifest.created_at.clone(),
    })
}

fn manifest_from_section(section: &ManuscriptSection) -> ManuscriptManifestSection {
    ManuscriptManifestSection {
        id: section.id.clone(),
        title: section.title.clone(),
        path: section.path.clone(),
        order: section.order,
        status: section.status.clone(),
        created_at: section.created_at.clone(),
        updated_at: section.updated_at.clone(),
    }
}

fn create_initial_sections(
    section_names: &[String],
    naming: &SectionNamingMode,
) -> Vec<ManuscriptSection> {
    let timestamp = now_iso();
    let mut existing = vec![];
    section_names
        .iter()
        .map(|title| title.trim())
        .filter(|title| !title.is_empty())
        .enumerate()
        .map(|(index, title)| {
            let order = (index + 1) as u32;
            let filename = make_section_filename(title, index + 1, naming, &mut existing);
            ManuscriptSection {
                id: format!("section_{}", Uuid::new_v4()),
                filename: filename.clone(),
                title: title.to_string(),
                order,
                content: format!("## {}\n\n", title),
                updated_at: timestamp.clone(),
                path: format!("manuscript/sections/{}", filename),
                status: SectionStatus::Draft,
                created_at: timestamp.clone(),
            }
        })
        .collect()
}

fn scan_existing_section_files(project: &ProjectConfig) -> Result<Vec<ManuscriptSection>, String> {
    let root = PathBuf::from(&project.root_path);
    let section_dir = root.join("manuscript/sections");
    if !section_dir.exists() {
        return Ok(vec![]);
    }
    let mut entries = fs::read_dir(section_dir)
        .map_err(|err| err.to_string())?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().and_then(|value| value.to_str()) == Some("md"))
        .collect::<Vec<_>>();
    entries.sort_by_key(|entry| entry.path());
    let timestamp = now_iso();
    entries
        .into_iter()
        .enumerate()
        .map(|(index, entry)| {
            let path = entry.path();
            let filename = path
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or("section.md")
                .to_string();
            let content = fs::read_to_string(&path).unwrap_or_default();
            let title = content
                .lines()
                .find_map(|line| line.strip_prefix("## ").or_else(|| line.strip_prefix("# ")))
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
                .unwrap_or_else(|| {
                    filename
                        .trim_end_matches(".md")
                        .replace('_', " ")
                        .replace('-', " ")
                });
            Ok(ManuscriptSection {
                id: format!("section_{}", Uuid::new_v4()),
                filename: filename.clone(),
                title,
                order: (index + 1) as u32,
                content,
                updated_at: timestamp.clone(),
                path: format!("manuscript/sections/{}", filename),
                status: SectionStatus::Draft,
                created_at: timestamp.clone(),
            })
        })
        .collect()
}

fn ensure_structure(project: &ProjectConfig) -> Result<(), String> {
    let root = PathBuf::from(&project.root_path);
    let folders = [
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
        "outputs",
    ];
    for folder in folders {
        fs::create_dir_all(root.join(folder)).map_err(|err| err.to_string())?;
    }
    fs::create_dir_all(root.join("logs")).map_err(|err| err.to_string())?;
    let raw_project = serde_json::to_string_pretty(project).map_err(|err| err.to_string())?;
    fs::write(project_manifest_path(&root), raw_project.as_bytes())
        .map_err(|err| err.to_string())?;
    fs::write(legacy_project_manifest_path(&root), raw_project.as_bytes())
        .map_err(|err| err.to_string())?;
    write_if_missing(&root.join("manuscript/paper.docx"), &[])?;
    write_if_missing(
        &root.join("manuscript/main.tex"),
        basic_latex_template().as_bytes(),
    )?;
    write_if_missing(&root.join("references/references.bib"), b"")?;
    write_if_missing(&root.join("templates/word_template.docx"), &[])?;
    write_if_missing(&root.join("ai/claims.json"), b"[]")?;
    write_if_missing(&root.join("literature/literature.json"), b"[]")?;
    write_if_missing(&activity_path(&root), b"[]")?;
    for manifest in &project.manuscript.sections {
        let path = root.join(&manifest.path);
        let content = format!("## {}\n\n", manifest.title);
        write_if_missing(&path, content.as_bytes())?;
    }
    Ok(())
}

fn write_if_missing(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if path.exists() {
        return Ok(());
    }
    fs::write(path, bytes).map_err(|err| err.to_string())
}

fn basic_latex_template() -> &'static str {
    "\\documentclass{article}\n\\usepackage[utf8]{inputenc}\n\\begin{document}\n\n% PaperForge generated draft.\n\n\\bibliographystyle{plain}\n\\bibliography{references}\n\\end{document}\n"
}

fn read_json_vec<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<Vec<T>, String> {
    if !path.exists() {
        return Ok(vec![]);
    }
    let raw = fs::read_to_string(path).map_err(|err| err.to_string())?;
    serde_json::from_str(&raw).map_err(|err| err.to_string())
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    fs::write(
        path,
        serde_json::to_string_pretty(value).map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())
}

#[tauri::command]
fn read_app_logs() -> Result<Vec<AppLog>, String> {
    read_json_vec(&app_logs_path()?)
}

#[tauri::command]
fn append_app_log(log: AppLog) -> Result<Vec<AppLog>, String> {
    let path = app_logs_path()?;
    let mut logs: Vec<AppLog> = read_json_vec(&path)?;
    logs.insert(0, log);
    logs.truncate(80);
    write_json(&path, &logs)?;
    Ok(logs)
}

fn append_project_activity(
    project: &ProjectConfig,
    activity_type: ProjectActivityType,
    message: String,
    section_id: Option<String>,
) -> Result<Vec<ProjectActivity>, String> {
    let root = PathBuf::from(&project.root_path);
    fs::create_dir_all(root.join("logs")).map_err(|err| err.to_string())?;
    let path = activity_path(&root);
    let mut activities: Vec<ProjectActivity> = read_json_vec(&path)?;
    activities.insert(
        0,
        ProjectActivity {
            id: format!("activity_{}", Uuid::new_v4()),
            activity_type,
            message,
            section_id,
            created_at: now_iso(),
        },
    );
    write_json(&path, &activities)?;
    Ok(activities)
}

#[tauri::command]
fn read_settings() -> Result<AppSettings, String> {
    let path = settings_path()?;
    if !path.exists() {
        let settings = default_settings();
        write_json(&path, &settings)?;
        return Ok(settings);
    }
    let raw = fs::read_to_string(path).map_err(|err| err.to_string())?;
    serde_json::from_str(&raw).map_err(|err| err.to_string())
}

#[tauri::command]
fn save_settings(settings: AppSettings) -> Result<AppSettings, String> {
    write_json(&settings_path()?, &settings)?;
    Ok(settings)
}

#[tauri::command]
fn create_project(input: ProjectCreateInput) -> Result<ProjectConfig, String> {
    let settings = read_settings()?;
    let title = if input.title.trim().is_empty() {
        default_project_title()
    } else {
        input.title.trim().to_string()
    };
    let author = input.author.trim().to_string();
    let authors = if author.is_empty() {
        vec![]
    } else {
        author
            .split(',')
            .map(|item| item.trim().to_string())
            .filter(|item| !item.is_empty())
            .collect()
    };
    let target_journal = if input.target_journal.trim().is_empty() {
        default_target_journal()
    } else {
        input.target_journal.trim().to_string()
    };
    let workspace_root = input
        .workspace_root
        .clone()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or(settings.workspace_root);
    let workspace_path = PathBuf::from(workspace_root);
    let root_path = workspace_path.join(safe_folder_name(&title));
    let timestamp = now_iso();
    let citation_backend = citation_backend(&input.manuscript_mode);
    let sections = create_initial_sections(&input.section_names, &input.section_naming);
    let project = ProjectConfig {
        id: format!("project_{}", Uuid::new_v4()),
        title,
        author,
        authors,
        target_journal,
        citation_style: input
            .citation_style
            .clone()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or(settings.default_citation_style),
        export_mode: input.export_mode.unwrap_or(settings.default_export_mode),
        manuscript_mode: input.manuscript_mode,
        root_path: root_path.to_string_lossy().to_string(),
        created_at: timestamp.clone(),
        updated_at: timestamp,
        citation_backend,
        manuscript: ManuscriptManifest {
            section_naming: input.section_naming,
            sections: sections.iter().map(manifest_from_section).collect(),
        },
    };
    ensure_structure(&project)?;
    let mut projects = read_registry()?;
    projects.insert(0, project.clone());
    write_registry(&projects)?;
    Ok(project)
}

#[tauri::command]
fn import_project_folder(root_path: String) -> Result<ProjectConfig, String> {
    let root = PathBuf::from(root_path.trim());
    if root.as_os_str().is_empty() {
        return Err("Project folder path is required".to_string());
    }
    fs::create_dir_all(&root).map_err(|err| err.to_string())?;
    let project_json = if project_manifest_path(&root).exists() {
        project_manifest_path(&root)
    } else {
        legacy_project_manifest_path(&root)
    };
    let mut project = if project_json.exists() {
        let raw = fs::read_to_string(&project_json).map_err(|err| err.to_string())?;
        serde_json::from_str::<ProjectConfig>(&raw).map_err(|err| err.to_string())?
    } else {
        let title = root
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("Imported Paper")
            .replace('_', " ");
        let timestamp = now_iso();
        ProjectConfig {
            id: format!("project_{}", Uuid::new_v4()),
            title,
            author: String::new(),
            authors: vec![],
            target_journal: default_target_journal(),
            citation_style: default_citation_style(),
            export_mode: ManuscriptMode::Markdown,
            manuscript_mode: ManuscriptMode::Word,
            root_path: root.to_string_lossy().to_string(),
            created_at: timestamp.clone(),
            updated_at: timestamp,
            citation_backend: CitationBackend::ZoteroWordPlugin,
            manuscript: default_manuscript_manifest(),
        }
    };
    project = normalize_project(project);
    project.root_path = root.to_string_lossy().to_string();
    if project.id.trim().is_empty() {
        project.id = format!("project_{}", Uuid::new_v4());
    }
    ensure_structure(&project)?;
    let mut projects = read_registry()?;
    projects.retain(|item| item.id != project.id && item.root_path != project.root_path);
    projects.insert(0, project.clone());
    write_registry(&projects)?;
    Ok(project)
}

#[tauri::command]
fn list_projects() -> Result<Vec<ProjectConfig>, String> {
    read_registry()
}

#[tauri::command]
fn open_project(project_id: String) -> Result<ProjectConfig, String> {
    project_by_id(&project_id)
}

#[tauri::command]
fn read_project_config(project_id: String) -> Result<ProjectConfig, String> {
    project_by_id(&project_id)
}

#[tauri::command]
fn update_project_config(project: ProjectConfig) -> Result<ProjectConfig, String> {
    let project = normalize_project(project);
    let mut projects = read_registry()?;
    projects = projects
        .into_iter()
        .map(|item| {
            if item.id == project.id {
                project.clone()
            } else {
                item
            }
        })
        .collect();
    write_registry(&projects)?;
    let root = PathBuf::from(&project.root_path);
    write_json(&project_manifest_path(&root), &project)?;
    write_json(&legacy_project_manifest_path(&root), &project)?;
    Ok(project)
}

#[tauri::command]
fn update_project_metadata(
    project_id: String,
    partial: ProjectMetadataPatch,
) -> Result<ProjectConfig, String> {
    let mut project = project_by_id(&project_id)?;
    if let Some(title) = partial.title {
        project.title = if title.trim().is_empty() {
            default_project_title()
        } else {
            title.trim().to_string()
        };
    }
    if let Some(author) = partial.author {
        project.author = author.trim().to_string();
        if partial.authors.is_none() {
            project.authors = if project.author.is_empty() {
                vec![]
            } else {
                project
                    .author
                    .split(',')
                    .map(|item| item.trim().to_string())
                    .filter(|item| !item.is_empty())
                    .collect()
            };
        }
    }
    if let Some(authors) = partial.authors {
        project.authors = authors
            .into_iter()
            .map(|item| item.trim().to_string())
            .filter(|item| !item.is_empty())
            .collect();
        project.author = project.authors.join(", ");
    }
    if let Some(target_journal) = partial.target_journal {
        project.target_journal = if target_journal.trim().is_empty() {
            default_target_journal()
        } else {
            target_journal.trim().to_string()
        };
    }
    if let Some(mode) = partial.manuscript_mode {
        project.manuscript_mode = mode;
        project.citation_backend = citation_backend(&project.manuscript_mode);
    }
    if let Some(citation_style) = partial.citation_style {
        project.citation_style = if citation_style.trim().is_empty() {
            default_citation_style()
        } else {
            citation_style.trim().to_string()
        };
    }
    if let Some(export_mode) = partial.export_mode {
        project.export_mode = export_mode;
    }
    project.updated_at = now_iso();
    update_project_config(project)
}

#[tauri::command]
fn ensure_project_structure(project_id: String) -> Result<bool, String> {
    let project = project_by_id(&project_id)?;
    ensure_structure(&project)?;
    Ok(true)
}

#[tauri::command]
fn delete_project(project_id: String, delete_files: bool) -> Result<bool, String> {
    let mut projects = read_registry()?;
    let project = projects
        .iter()
        .find(|project| project.id == project_id)
        .cloned();
    projects.retain(|project| project.id != project_id);
    write_registry(&projects)?;
    if delete_files {
        if let Some(project) = project {
            let path = PathBuf::from(project.root_path);
            if path.exists() {
                fs::remove_dir_all(path).map_err(|err| err.to_string())?;
            }
        }
    }
    Ok(true)
}

#[tauri::command]
fn export_project_manifest(project_id: String) -> Result<String, String> {
    let project = project_by_id(&project_id)?;
    let root = PathBuf::from(&project.root_path);
    let manifest = serde_json::json!({
        "exportedAt": now_iso(),
        "project": project,
        "sections": list_sections(project_id.clone())?,
        "references": list_references(project_id.clone())?,
        "citationTasks": scan_citation_tasks(project_id.clone())?,
        "literature": list_literature(project_id.clone())?,
        "claims": list_claims(project_id.clone())?,
        "note": "PaperForge MVP manifest export. User manuscripts stay local."
    });
    let raw = serde_json::to_string_pretty(&manifest).map_err(|err| err.to_string())?;
    fs::create_dir_all(root.join("outputs")).map_err(|err| err.to_string())?;
    fs::write(root.join("outputs/project_manifest.json"), &raw).map_err(|err| err.to_string())?;
    Ok(raw)
}

#[tauri::command]
fn list_sections(project_id: String) -> Result<Vec<ManuscriptSection>, String> {
    let mut project = project_by_id(&project_id)?;
    if project.manuscript.sections.is_empty() {
        let scanned = scan_existing_section_files(&project)?;
        if !scanned.is_empty() {
            project.manuscript.sections = scanned.iter().map(manifest_from_section).collect();
            update_project_config(project.clone())?;
            return Ok(scanned);
        }
    }
    project
        .manuscript
        .sections
        .iter()
        .map(|section| section_from_manifest(&project, section))
        .collect()
}

#[tauri::command]
fn read_section(project_id: String, filename: String) -> Result<ManuscriptSection, String> {
    list_sections(project_id)?
        .into_iter()
        .find(|section| section.filename == filename)
        .ok_or_else(|| "Section not found".to_string())
}

#[tauri::command]
fn save_section(
    project_id: String,
    section: ManuscriptSection,
) -> Result<ManuscriptSection, String> {
    let mut project = project_by_id(&project_id)?;
    let mut saved = section;
    saved.updated_at = now_iso();
    fs::write(
        PathBuf::from(&project.root_path).join(&saved.path),
        &saved.content,
    )
    .map_err(|err| err.to_string())?;
    for manifest in &mut project.manuscript.sections {
        if manifest.id == saved.id {
            manifest.title = saved.title.clone();
            manifest.updated_at = saved.updated_at.clone();
            manifest.status = saved.status.clone();
        }
    }
    update_project_config(project)?;
    Ok(saved)
}

#[tauri::command]
fn create_section(
    project_id: String,
    input: SectionCreateInput,
) -> Result<ManuscriptSection, String> {
    let mut project = project_by_id(&project_id)?;
    let title = input.title.trim();
    if title.is_empty() {
        return Err("Section title is required".to_string());
    }
    let root = PathBuf::from(&project.root_path);
    let order = project.manuscript.sections.len() + 1;
    let mut existing = project
        .manuscript
        .sections
        .iter()
        .filter_map(|section| {
            PathBuf::from(&section.path)
                .file_name()
                .and_then(|value| value.to_str())
                .map(|value| value.to_string())
        })
        .collect::<Vec<_>>();
    let filename = make_section_filename(
        title,
        order,
        &project.manuscript.section_naming,
        &mut existing,
    );
    let timestamp = now_iso();
    let section = ManuscriptSection {
        id: format!("section_{}", Uuid::new_v4()),
        filename: filename.clone(),
        title: title.to_string(),
        order: order as u32,
        content: format!("## {}\n\n", title),
        updated_at: timestamp.clone(),
        path: format!("manuscript/sections/{}", filename),
        status: SectionStatus::Draft,
        created_at: timestamp.clone(),
    };
    fs::write(root.join(&section.path), &section.content).map_err(|err| err.to_string())?;
    project
        .manuscript
        .sections
        .push(manifest_from_section(&section));
    update_project_config(project.clone())?;
    append_project_activity(
        &project,
        ProjectActivityType::SectionCreated,
        format!("Created section: {}", section.title),
        Some(section.id.clone()),
    )?;
    Ok(section)
}

#[tauri::command]
fn rename_section(
    project_id: String,
    input: SectionRenameInput,
) -> Result<ManuscriptSection, String> {
    let mut project = project_by_id(&project_id)?;
    let title = input.title.trim();
    if title.is_empty() {
        return Err("Section title is required".to_string());
    }
    let timestamp = now_iso();
    let mut found = None;
    for manifest in &mut project.manuscript.sections {
        if manifest.id == input.section_id {
            manifest.title = title.to_string();
            manifest.updated_at = timestamp.clone();
            found = Some(manifest.clone());
        }
    }
    let manifest = found.ok_or_else(|| "Section not found".to_string())?;
    update_project_config(project.clone())?;
    append_project_activity(
        &project,
        ProjectActivityType::SectionRenamed,
        format!(
            "Renamed section: {}. File path kept: {}",
            manifest.title, manifest.path
        ),
        Some(manifest.id.clone()),
    )?;
    section_from_manifest(&project, &manifest)
}

#[tauri::command]
fn list_project_tree(_project_id: String) -> Result<Vec<String>, String> {
    Ok(vec![
        "manuscript".to_string(),
        "references".to_string(),
        "literature".to_string(),
        "figures".to_string(),
        "data".to_string(),
        "ai".to_string(),
        "outputs".to_string(),
    ])
}

fn bib_field(body: &str, field: &str) -> String {
    let pattern = format!(
        r#"(?is){}\s*=\s*(?:\{{([^{{}}]*(?:\{{[^{{}}]*\}}[^{{}}]*)*)\}}|"([^"]*)")"#,
        field
    );
    Regex::new(&pattern)
        .ok()
        .and_then(|re| re.captures(body))
        .and_then(|captures| {
            captures
                .get(1)
                .or_else(|| captures.get(2))
                .map(|m| m.as_str().to_string())
        })
        .unwrap_or_default()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[tauri::command]
fn parse_bibtex(bibtex: String) -> Result<Vec<ReferenceItem>, String> {
    let entry_re = Regex::new(r"(?is)@\w+\s*\{\s*([^,\s]+)\s*,(.*?)(?=\n@\w+\s*\{|$)")
        .map_err(|err| err.to_string())?;
    let mut refs = vec![];
    for capture in entry_re.captures_iter(&bibtex) {
        let citekey = capture
            .get(1)
            .map(|m| m.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        let body = capture.get(2).map(|m| m.as_str()).unwrap_or("");
        let authors = bib_field(body, "author")
            .split(" and ")
            .filter(|item| !item.trim().is_empty())
            .map(|item| item.trim().to_string())
            .collect();
        refs.push(ReferenceItem {
            citekey,
            title: {
                let title = bib_field(body, "title");
                if title.is_empty() {
                    "(untitled)".to_string()
                } else {
                    title
                }
            },
            authors,
            year: bib_field(body, "year"),
            journal: {
                let journal = bib_field(body, "journal");
                if journal.is_empty() {
                    bib_field(body, "booktitle")
                } else {
                    journal
                }
            },
            doi: bib_field(body, "doi"),
            abstract_text: {
                let value = bib_field(body, "abstract");
                if value.is_empty() {
                    None
                } else {
                    Some(value)
                }
            },
            zotero_item_key: None,
            library_id: None,
            pdf_path: None,
        });
    }
    Ok(refs)
}

#[tauri::command]
fn save_bibtex(project_id: String, bibtex: String) -> Result<Vec<ReferenceItem>, String> {
    let project = project_by_id(&project_id)?;
    fs::write(
        PathBuf::from(&project.root_path).join("references/references.bib"),
        &bibtex,
    )
    .map_err(|err| err.to_string())?;
    let refs = parse_bibtex(bibtex)?;
    write_json(
        &PathBuf::from(&project.root_path).join("references/references.json"),
        &refs,
    )?;
    Ok(refs)
}

#[tauri::command]
fn list_references(project_id: String) -> Result<Vec<ReferenceItem>, String> {
    let project = project_by_id(&project_id)?;
    let refs_path = PathBuf::from(&project.root_path).join("references/references.json");
    if refs_path.exists() {
        return read_json_vec(&refs_path);
    }
    let bib_path = PathBuf::from(&project.root_path).join("references/references.bib");
    let bibtex = fs::read_to_string(bib_path).unwrap_or_default();
    parse_bibtex(bibtex)
}

#[tauri::command]
fn scan_citation_tasks(project_id: String) -> Result<Vec<CitationTask>, String> {
    let sections = list_sections(project_id.clone())?;
    let refs = list_references(project_id.clone())?;
    let project = project_by_id(&project_id)?;
    let task_path = PathBuf::from(&project.root_path).join("references/citation_tasks.json");
    let previous: Vec<CitationTask> = read_json_vec(&task_path)?;
    let placeholder_re =
        Regex::new(r"\[CITE:\s*([A-Za-z0-9_:.+-]+)\s*\]").map_err(|err| err.to_string())?;
    let mut tasks = vec![];
    for section in sections {
        for capture in placeholder_re.captures_iter(&section.content) {
            let placeholder = capture.get(0).map(|m| m.as_str()).unwrap_or("").to_string();
            let citekey = capture.get(1).map(|m| m.as_str()).unwrap_or("").to_string();
            let previous_status = previous
                .iter()
                .find(|task| task.section_id == section.id && task.placeholder == placeholder)
                .map(|task| task.status.clone())
                .unwrap_or(CitationStatus::Pending);
            let reference = refs.iter().find(|item| item.citekey == citekey).cloned();
            tasks.push(CitationTask {
                id: format!("{}_{}_{}", section.id, tasks.len(), citekey),
                section_id: section.id.clone(),
                placeholder,
                citekey,
                status: previous_status,
                reference,
            });
        }
    }
    write_json(&task_path, &tasks)?;
    Ok(tasks)
}

#[tauri::command]
fn update_citation_task_status(
    project_id: String,
    task_id: String,
    status: CitationStatus,
) -> Result<Vec<CitationTask>, String> {
    let project = project_by_id(&project_id)?;
    let task_path = PathBuf::from(&project.root_path).join("references/citation_tasks.json");
    let mut tasks: Vec<CitationTask> = read_json_vec(&task_path)?;
    for task in &mut tasks {
        if task.id == task_id {
            task.status = status.clone();
        }
    }
    write_json(&task_path, &tasks)?;
    Ok(tasks)
}

#[tauri::command]
fn add_literature_item(
    project_id: String,
    item: LiteratureItemInput,
) -> Result<LiteratureItem, String> {
    let project = project_by_id(&project_id)?;
    let path = PathBuf::from(&project.root_path).join("literature/literature.json");
    let mut items: Vec<LiteratureItem> = read_json_vec(&path)?;
    let item = LiteratureItem {
        id: format!("lit_{}", Uuid::new_v4()),
        filename: item.filename,
        path: item.path,
        linked_citekey: item.linked_citekey,
        notes: item.notes,
        abstract_text: item.abstract_text,
        embedding_status: EmbeddingStatus::NotIndexed,
    };
    items.insert(0, item.clone());
    write_json(&path, &items)?;
    Ok(item)
}

#[tauri::command]
fn list_literature(project_id: String) -> Result<Vec<LiteratureItem>, String> {
    let project = project_by_id(&project_id)?;
    read_json_vec(&PathBuf::from(&project.root_path).join("literature/literature.json"))
}

#[tauri::command]
fn search_literature_mock(
    project_id: String,
    query: String,
) -> Result<Vec<LiteratureItem>, String> {
    let items = list_literature(project_id)?;
    let q = query.to_lowercase();
    if q.trim().is_empty() {
        return Ok(items);
    }
    Ok(items
        .into_iter()
        .filter(|item| {
            [
                item.filename.as_str(),
                item.path.as_str(),
                item.notes.as_str(),
                item.linked_citekey.as_deref().unwrap_or(""),
            ]
            .iter()
            .any(|value| value.to_lowercase().contains(&q))
        })
        .collect())
}

#[tauri::command]
fn list_claims(project_id: String) -> Result<Vec<ClaimRecord>, String> {
    let project = project_by_id(&project_id)?;
    read_json_vec(&PathBuf::from(&project.root_path).join("ai/claims.json"))
}

#[tauri::command]
fn save_claims(project_id: String, claims: Vec<ClaimRecord>) -> Result<Vec<ClaimRecord>, String> {
    let project = project_by_id(&project_id)?;
    write_json(
        &PathBuf::from(&project.root_path).join("ai/claims.json"),
        &claims,
    )?;
    Ok(claims)
}

#[tauri::command]
fn add_claim(project_id: String, claim: ClaimRecord) -> Result<ClaimRecord, String> {
    let mut claims = list_claims(project_id.clone())?;
    claims.insert(0, claim.clone());
    save_claims(project_id, claims)?;
    Ok(claim)
}

#[tauri::command]
fn update_claim_status(
    project_id: String,
    claim_id: String,
    status: ClaimStatus,
) -> Result<Vec<ClaimRecord>, String> {
    let mut claims = list_claims(project_id.clone())?;
    for claim in &mut claims {
        if claim.id == claim_id {
            claim.status = status.clone();
        }
    }
    save_claims(project_id, claims)
}

#[tauri::command]
fn generate_ai_proposal(
    _project_id: String,
    section_id: String,
    instruction: String,
    selected_text: String,
    settings: AppSettings,
) -> Result<AiProposal, String> {
    let prefix = if settings.llm_provider.api_key.trim().is_empty() {
        "MOCK: no API key configured."
    } else {
        "Provider abstraction ready; mock proposal:"
    };
    Ok(AiProposal {
        id: format!("proposal_{}", Uuid::new_v4()),
        section_id,
        instruction,
        original_text: selected_text.clone(),
        proposed_text: format!(
            "{}\n\n{} can be revised with clearer contribution, cautious claims, and citation hooks such as [CITE: Zhang2023].",
            prefix,
            if selected_text.trim().is_empty() { "This paragraph" } else { selected_text.as_str() }
        ),
        citation_keys: vec!["Zhang2023".to_string()],
        created_at: now_iso(),
        status: ProposalStatus::Pending,
    })
}

#[tauri::command]
fn apply_ai_proposal(
    project_id: String,
    proposal: AiProposal,
    section: ManuscriptSection,
) -> Result<ManuscriptSection, String> {
    let mut next = section;
    next.content = if proposal.original_text.trim().is_empty() {
        format!("{}\n\n{}\n", next.content.trim(), proposal.proposed_text)
    } else {
        next.content
            .replace(&proposal.original_text, &proposal.proposed_text)
    };
    save_section(project_id, next)
}

fn merged_sections(project_id: &str) -> Result<String, String> {
    let mut sections = list_sections(project_id.to_string())?;
    sections.sort_by_key(|section| section.order);
    Ok(sections
        .into_iter()
        .map(|section| section.content.trim().to_string())
        .collect::<Vec<_>>()
        .join("\n\n"))
}

fn convert_citations_for_mode(markdown: &str, mode: &ManuscriptMode) -> Result<String, String> {
    let word_re =
        Regex::new(r"\[CITE:\s*([A-Za-z0-9_:.+-]+)\s*\]").map_err(|err| err.to_string())?;
    let pandoc_re = Regex::new(r"\[@([A-Za-z0-9_:.+-]+)\]").map_err(|err| err.to_string())?;
    let latex_re = Regex::new(r"\\cite\{([^}]+)\}").map_err(|err| err.to_string())?;
    let converted = match mode {
        ManuscriptMode::Word => {
            let step = latex_re.replace_all(markdown, |captures: &regex::Captures| {
                format!("[CITE: {}]", captures[1].trim())
            });
            pandoc_re
                .replace_all(&step, |captures: &regex::Captures| {
                    format!("[CITE: {}]", captures[1].trim())
                })
                .to_string()
        }
        ManuscriptMode::Latex => {
            let step = word_re.replace_all(markdown, |captures: &regex::Captures| {
                format!("\\cite{{{}}}", captures[1].trim())
            });
            pandoc_re
                .replace_all(&step, |captures: &regex::Captures| {
                    format!("\\cite{{{}}}", captures[1].trim())
                })
                .to_string()
        }
        ManuscriptMode::Markdown => {
            let step = word_re.replace_all(markdown, |captures: &regex::Captures| {
                format!("[@{}]", captures[1].trim())
            });
            latex_re
                .replace_all(&step, |captures: &regex::Captures| {
                    format!("[@{}]", captures[1].trim())
                })
                .to_string()
        }
    };
    Ok(converted)
}

fn copy_dir_recursive(
    src: &Path,
    dst: &Path,
    files: &mut Vec<String>,
    prefix: &str,
) -> Result<(), String> {
    if !src.exists() {
        return Ok(());
    }
    fs::create_dir_all(dst).map_err(|err| err.to_string())?;
    for entry in fs::read_dir(src).map_err(|err| err.to_string())? {
        let entry = entry.map_err(|err| err.to_string())?;
        let src_path = entry.path();
        let name = entry.file_name();
        let dst_path = dst.join(&name);
        let rel = format!("{}/{}", prefix, name.to_string_lossy()).replace('\\', "/");
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path, files, &rel)?;
        } else {
            fs::copy(&src_path, &dst_path).map_err(|err| err.to_string())?;
            files.push(rel);
        }
    }
    Ok(())
}

fn copy_optional_file(
    src: &Path,
    dst: &Path,
    rel: &str,
    files: &mut Vec<String>,
    skipped: &mut Vec<serde_json::Value>,
) -> Result<(), String> {
    if src.exists() {
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent).map_err(|err| err.to_string())?;
        }
        fs::copy(src, dst).map_err(|err| err.to_string())?;
        files.push(rel.to_string());
    } else {
        skipped.push(serde_json::json!({ "path": rel, "reason": "File does not exist" }));
    }
    Ok(())
}

fn markdown_package_body(project_id: &str) -> Result<(String, Vec<String>), String> {
    let mut sections = list_sections(project_id.to_string())?;
    sections.sort_by_key(|section| section.order);
    if sections.is_empty() {
        return Ok((
            "<!-- Empty manuscript exported by PaperForge -->\n".to_string(),
            vec![],
        ));
    }
    let h1_re = Regex::new(r"(?m)^#\s+").map_err(|err| err.to_string())?;
    let mut warnings = vec![];
    let mut chunks = vec![];
    for section in sections {
        let content =
            convert_citations_for_mode(section.content.trim(), &ManuscriptMode::Markdown)?;
        if h1_re.is_match(&content) {
            warnings.push(format!(
                "Section '{}' already contains a level-one heading.",
                section.title
            ));
        }
        chunks.push(format!("# {}\n\n{}", section.title, content));
    }
    Ok((chunks.join("\n\n"), warnings))
}

#[tauri::command]
fn export_markdown_package(project_id: String) -> Result<ExportJob, String> {
    let project = normalize_project(project_by_id(&project_id)?);
    let root = PathBuf::from(&project.root_path);
    let stamp = Utc::now().format("%Y%m%d-%H%M%S").to_string();
    let output = root
        .join("outputs")
        .join(format!("paperforge-export-{}", stamp));
    fs::create_dir_all(&output).map_err(|err| err.to_string())?;

    let mut files: Vec<String> = vec![];
    let mut skipped: Vec<serde_json::Value> = vec![];
    let mut warnings: Vec<String> = vec![];

    write_json(&output.join("manifest.json"), &project)?;
    files.push("manifest.json".to_string());

    let (body, body_warnings) = markdown_package_body(&project_id)?;
    warnings.extend(body_warnings);
    fs::write(output.join("manuscript.md"), body).map_err(|err| err.to_string())?;
    files.push("manuscript.md".to_string());

    fs::create_dir_all(output.join("sections")).map_err(|err| err.to_string())?;
    let mut sections = list_sections(project_id.clone())?;
    sections.sort_by_key(|section| section.order);
    for section in sections {
        let src = root.join(&section.path);
        let dst = output.join("sections").join(&section.filename);
        let rel = format!("sections/{}", section.filename);
        copy_optional_file(&src, &dst, &rel, &mut files, &mut skipped)?;
    }

    let bib_src = {
        let library = root.join("references/library.bib");
        if library.exists() {
            library
        } else {
            root.join("references/references.bib")
        }
    };
    copy_optional_file(
        &bib_src,
        &output.join("references/library.bib"),
        "references/library.bib",
        &mut files,
        &mut skipped,
    )?;
    copy_optional_file(
        &root.join("references/references.json"),
        &output.join("references/references.json"),
        "references/references.json",
        &mut files,
        &mut skipped,
    )?;
    copy_optional_file(
        &root.join("references/citation_tasks.json"),
        &output.join("references/citation-queue.json"),
        "references/citation-queue.json",
        &mut files,
        &mut skipped,
    )?;
    let papers_src = {
        let papers = root.join("literature/papers.json");
        if papers.exists() {
            papers
        } else {
            root.join("literature/literature.json")
        }
    };
    copy_optional_file(
        &papers_src,
        &output.join("literature/papers.json"),
        "literature/papers.json",
        &mut files,
        &mut skipped,
    )?;
    copy_dir_recursive(
        &root.join("figures"),
        &output.join("figures"),
        &mut files,
        "figures",
    )?;
    copy_dir_recursive(&root.join("data"), &output.join("data"), &mut files, "data")?;
    copy_optional_file(
        &root.join("ai/claims.json"),
        &output.join("claims/claims.json"),
        "claims/claims.json",
        &mut files,
        &mut skipped,
    )?;

    let report = serde_json::json!({
        "exportedAt": now_iso(),
        "projectTitle": project.title,
        "mode": "markdown",
        "outputDir": output.to_string_lossy(),
        "files": files,
        "skipped": skipped,
        "warnings": warnings,
    });
    write_json(&output.join("export-report.json"), &report)?;

    Ok(ExportJob {
        id: format!("export_{}", Uuid::new_v4()),
        project_id,
        mode: ManuscriptMode::Markdown,
        status: ExportStatus::Success,
        output_path: output.to_string_lossy().to_string(),
        logs: vec![
            "Markdown package exported.".to_string(),
            "Includes manifest.json, manuscript.md, sections/, export-report.json.".to_string(),
        ],
        created_at: now_iso(),
    })
}

#[tauri::command]
fn export_word_draft_placeholder(project_id: String) -> Result<ExportJob, String> {
    Ok(ExportJob {
        id: format!("export_{}", Uuid::new_v4()),
        project_id,
        mode: ManuscriptMode::Word,
        status: ExportStatus::Pending,
        output_path: String::new(),
        logs: vec!["Coming soon. Recommended route: Markdown/Pandoc to DOCX.".to_string()],
        created_at: now_iso(),
    })
}

#[tauri::command]
fn export_latex_placeholder(project_id: String) -> Result<ExportJob, String> {
    Ok(ExportJob {
        id: format!("export_{}", Uuid::new_v4()),
        project_id,
        mode: ManuscriptMode::Latex,
        status: ExportStatus::Pending,
        output_path: String::new(),
        logs: vec!["Coming soon. Current stable export is Markdown package.".to_string()],
        created_at: now_iso(),
    })
}

#[tauri::command]
fn export_word_draft(project_id: String) -> Result<ExportJob, String> {
    let project = project_by_id(&project_id)?;
    let root = PathBuf::from(&project.root_path);
    let output = root.join("outputs/combined_word_draft.md");
    fs::write(
        &output,
        convert_citations_for_mode(&merged_sections(&project_id)?, &ManuscriptMode::Word)?,
    )
    .map_err(|err| err.to_string())?;
    let tasks = scan_citation_tasks(project_id.clone())?;
    write_json(&root.join("outputs/citation_tasks.json"), &tasks)?;
    Ok(ExportJob {
        id: format!("export_{}", Uuid::new_v4()),
        project_id,
        mode: ManuscriptMode::Word,
        status: ExportStatus::Success,
        output_path: output.to_string_lossy().to_string(),
        logs: vec![
            "Word draft exported as merged Markdown.".to_string(),
            "Kept [CITE: key] placeholders for Zotero Word plugin.".to_string(),
            "Generated citation_tasks.json.".to_string(),
        ],
        created_at: now_iso(),
    })
}

#[tauri::command]
fn export_latex(project_id: String) -> Result<ExportJob, String> {
    let project = project_by_id(&project_id)?;
    let root = PathBuf::from(&project.root_path);
    let output = root.join("outputs/main.tex");
    let body = convert_citations_for_mode(&merged_sections(&project_id)?, &ManuscriptMode::Latex)?;
    let tex = format!("\\documentclass{{article}}\n\\usepackage[utf8]{{inputenc}}\n\\begin{{document}}\n\n{}\n\n\\bibliographystyle{{plain}}\n\\bibliography{{references}}\n\\end{{document}}\n", body);
    fs::write(&output, tex).map_err(|err| err.to_string())?;
    let bib_src = root.join("references/references.bib");
    let bib_dst = root.join("outputs/references.bib");
    if bib_src.exists() {
        fs::copy(bib_src, bib_dst).map_err(|err| err.to_string())?;
    }
    Ok(ExportJob {
        id: format!("export_{}", Uuid::new_v4()),
        project_id,
        mode: ManuscriptMode::Latex,
        status: ExportStatus::Success,
        output_path: output.to_string_lossy().to_string(),
        logs: vec![
            "LaTeX main.tex generated.".to_string(),
            "references.bib copied when available.".to_string(),
        ],
        created_at: now_iso(),
    })
}

#[tauri::command]
fn export_markdown_pandoc(project_id: String) -> Result<ExportJob, String> {
    let project = project_by_id(&project_id)?;
    let root = PathBuf::from(&project.root_path);
    let output = root.join("outputs/combined.md");
    fs::write(
        &output,
        convert_citations_for_mode(&merged_sections(&project_id)?, &ManuscriptMode::Markdown)?,
    )
    .map_err(|err| err.to_string())?;
    let command = "pandoc combined.md --bibliography ../references/references.bib --csl ../references/csl/style.csl -o paper.docx";
    fs::write(root.join("outputs/pandoc_command.txt"), command).map_err(|err| err.to_string())?;
    Ok(ExportJob {
        id: format!("export_{}", Uuid::new_v4()),
        project_id,
        mode: ManuscriptMode::Markdown,
        status: ExportStatus::Success,
        output_path: output.to_string_lossy().to_string(),
        logs: vec![
            "combined.md generated.".to_string(),
            format!("Pandoc command: {}", command),
        ],
        created_at: now_iso(),
    })
}

#[tauri::command]
fn open_path(path: String) -> Result<bool, String> {
    let target = PathBuf::from(path);
    if !target.exists() {
        return Err("Path does not exist".to_string());
    }
    #[cfg(target_os = "windows")]
    let mut command = {
        let mut command = std::process::Command::new("explorer");
        command.arg(target);
        command
    };
    #[cfg(target_os = "macos")]
    let mut command = {
        let mut command = std::process::Command::new("open");
        command.arg(target);
        command
    };
    #[cfg(all(unix, not(target_os = "macos")))]
    let mut command = {
        let mut command = std::process::Command::new("xdg-open");
        command.arg(target);
        command
    };
    command.spawn().map_err(|err| err.to_string())?;
    Ok(true)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            create_project,
            import_project_folder,
            list_projects,
            open_project,
            read_project_config,
            update_project_config,
            update_project_metadata,
            ensure_project_structure,
            delete_project,
            export_project_manifest,
            list_sections,
            read_section,
            save_section,
            create_section,
            rename_section,
            list_project_tree,
            parse_bibtex,
            save_bibtex,
            list_references,
            scan_citation_tasks,
            update_citation_task_status,
            add_literature_item,
            list_literature,
            search_literature_mock,
            list_claims,
            save_claims,
            add_claim,
            update_claim_status,
            export_word_draft,
            export_latex,
            export_markdown_pandoc,
            export_markdown_package,
            export_word_draft_placeholder,
            export_latex_placeholder,
            read_settings,
            save_settings,
            generate_ai_proposal,
            apply_ai_proposal,
            read_app_logs,
            append_app_log,
            open_path
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn main() {
    run();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static CWD_LOCK: Mutex<()> = Mutex::new(());

    fn temp_app_dir(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("paperforge_test_{}_{}", name, Uuid::new_v4()))
    }

    fn with_temp_cwd<T>(name: &str, run: impl FnOnce(PathBuf) -> T) -> T {
        let _guard = CWD_LOCK.lock().expect("cwd lock");
        let original = std::env::current_dir().expect("current dir");
        let dir = temp_app_dir(name);
        fs::create_dir_all(&dir).expect("temp dir");
        std::env::set_current_dir(&dir).expect("set temp cwd");
        let result = run(dir.clone());
        std::env::set_current_dir(original).expect("restore cwd");
        let _ = fs::remove_dir_all(dir);
        result
    }

    fn create_input(
        title: &str,
        naming: SectionNamingMode,
        names: Vec<&str>,
    ) -> ProjectCreateInput {
        ProjectCreateInput {
            title: title.to_string(),
            author: "Tester".to_string(),
            target_journal: String::new(),
            manuscript_mode: ManuscriptMode::Word,
            citation_style: None,
            export_mode: None,
            workspace_root: Some("workspace".to_string()),
            section_naming: naming,
            section_names: names.into_iter().map(|value| value.to_string()).collect(),
        }
    }

    #[test]
    fn empty_manuscript_creates_no_section_files_and_valid_manifest() {
        with_temp_cwd("empty", |_dir| {
            let project = create_project(create_input(
                "Empty Paper",
                SectionNamingMode::Numbered,
                vec![],
            ))
            .expect("project");
            let root = PathBuf::from(&project.root_path);
            assert!(root.join("manuscript/sections").exists());
            assert!(list_sections(project.id.clone())
                .expect("sections")
                .is_empty());
            let manifest: ProjectConfig = serde_json::from_str(
                &fs::read_to_string(root.join("paperforge.project.json")).expect("manifest raw"),
            )
            .expect("manifest json");
            assert!(manifest.manuscript.sections.is_empty());
        });
    }

    #[test]
    fn template_names_use_numbered_files_and_manifest_paths() {
        with_temp_cwd("standard", |_dir| {
            let project = create_project(create_input(
                "Standard Paper",
                SectionNamingMode::Numbered,
                vec!["Abstract", "Introduction", "Methods"],
            ))
            .expect("project");
            let sections = list_sections(project.id).expect("sections");
            assert_eq!(
                sections
                    .iter()
                    .map(|item| item.filename.as_str())
                    .collect::<Vec<_>>(),
                vec!["01_abstract.md", "02_introduction.md", "03_methods.md",]
            );
            assert!(PathBuf::from(&project.root_path)
                .join("paperforge.project.json")
                .exists());
        });
    }

    #[test]
    fn slug_only_chinese_and_duplicate_names_are_safe() {
        with_temp_cwd("slug", |_dir| {
            let project = create_project(create_input(
                "Slug Paper",
                SectionNamingMode::SlugOnly,
                vec!["引言", "Introduction", "Introduction"],
            ))
            .expect("project");
            let sections = list_sections(project.id).expect("sections");
            assert_eq!(sections[0].filename, "section-001.md");
            assert_eq!(sections[1].filename, "introduction.md");
            assert_eq!(sections[2].filename, "introduction_2.md");
        });
    }

    #[test]
    fn create_and_rename_section_update_manifest_and_activity() {
        with_temp_cwd("activity", |_dir| {
            let project = create_project(create_input(
                "Activity Paper",
                SectionNamingMode::Numbered,
                vec![],
            ))
            .expect("project");
            let section = create_section(
                project.id.clone(),
                SectionCreateInput {
                    title: "新增章节".to_string(),
                },
            )
            .expect("section");
            assert_eq!(section.filename, "01_section.md");
            let renamed = rename_section(
                project.id.clone(),
                SectionRenameInput {
                    section_id: section.id.clone(),
                    title: "Renamed Section".to_string(),
                },
            )
            .expect("renamed");
            assert_eq!(renamed.filename, "01_section.md");
            assert_eq!(renamed.title, "Renamed Section");
            let root = PathBuf::from(project_by_id(&project.id).expect("project").root_path);
            let manifest: ProjectConfig = serde_json::from_str(
                &fs::read_to_string(root.join("paperforge.project.json")).expect("manifest raw"),
            )
            .expect("manifest json");
            assert_eq!(manifest.manuscript.sections[0].title, "Renamed Section");
            assert_eq!(
                manifest.manuscript.sections[0].path,
                "manuscript/sections/01_section.md"
            );
            let activities: Vec<ProjectActivity> = serde_json::from_str(
                &fs::read_to_string(root.join("logs/activity.json")).expect("activity raw"),
            )
            .expect("activity json");
            assert!(activities
                .iter()
                .any(|item| matches!(&item.activity_type, ProjectActivityType::SectionCreated)));
            assert!(activities
                .iter()
                .any(|item| matches!(&item.activity_type, ProjectActivityType::SectionRenamed)));
        });
    }

    #[test]
    fn blank_metadata_gets_safe_defaults() {
        with_temp_cwd("metadata", |_dir| {
            let mut input = create_input("", SectionNamingMode::Numbered, vec![]);
            input.author = String::new();
            let project = create_project(input).expect("project");
            assert_eq!(project.title, "Untitled Paper");
            assert!(project.authors.is_empty());
            assert_eq!(project.target_journal, "Unspecified Journal");
            assert_eq!(project.citation_style, "apa");
            assert!(matches!(project.export_mode, ManuscriptMode::Markdown));
            let root = PathBuf::from(&project.root_path);
            let manifest: ProjectConfig = serde_json::from_str(
                &fs::read_to_string(root.join("paperforge.project.json")).expect("manifest raw"),
            )
            .expect("manifest json");
            assert_eq!(manifest.title, "Untitled Paper");
            assert_eq!(manifest.target_journal, "Unspecified Journal");
        });
    }

    #[test]
    fn markdown_package_export_writes_report_and_sections() {
        with_temp_cwd("markdown_export", |_dir| {
            let project = create_project(create_input(
                "Export Paper",
                SectionNamingMode::Numbered,
                vec!["Introduction", "Methods"],
            ))
            .expect("project");
            fs::write(
                PathBuf::from(&project.root_path).join("references/references.bib"),
                "@article{Zhang2023,title={Test}}",
            )
            .expect("bib");
            let job = export_markdown_package(project.id.clone()).expect("export");
            assert!(matches!(job.status, ExportStatus::Success));
            let output = PathBuf::from(job.output_path);
            assert!(output.join("manifest.json").exists());
            assert!(output.join("manuscript.md").exists());
            assert!(output.join("sections/01_introduction.md").exists());
            assert!(output.join("sections/02_methods.md").exists());
            assert!(output.join("references/library.bib").exists());
            let report_raw = fs::read_to_string(output.join("export-report.json")).expect("report");
            assert!(report_raw.contains("manuscript.md"));
            assert!(!output.join(".git").exists());
        });
    }
}
