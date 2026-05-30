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
    workspace_root: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProjectConfig {
    id: String,
    title: String,
    author: String,
    target_journal: String,
    manuscript_mode: ManuscriptMode,
    root_path: String,
    created_at: String,
    updated_at: String,
    citation_backend: CitationBackend,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum ManuscriptMode {
    Word,
    Latex,
    Markdown,
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
struct ManuscriptSection {
    id: String,
    filename: String,
    title: String,
    order: u32,
    content: String,
    updated_at: String,
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
#[serde(rename_all = "camelCase")]
enum ThemeMode {
    Dark,
    Light,
    EyeCare,
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

fn safe_folder_name(title: &str) -> String {
    let cleaned: String = title
        .chars()
        .filter(|ch| !matches!(ch, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*') && !ch.is_control())
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
        default_export_mode: ManuscriptMode::Word,
        theme_mode: ThemeMode::Dark,
    }
}

fn read_registry() -> Result<Vec<ProjectConfig>, String> {
    let path = registry_path()?;
    if !path.exists() {
        return Ok(vec![]);
    }
    let raw = fs::read_to_string(path).map_err(|err| err.to_string())?;
    serde_json::from_str(&raw).map_err(|err| err.to_string())
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

fn initial_sections() -> Vec<ManuscriptSection> {
    let data = [
        ("01_abstract.md", "Abstract", 1, "## Abstract\n\nDraft the study objective, methods, key results, and conclusion.\n"),
        ("02_introduction.md", "Introduction", 2, "## Introduction\n\nFrame the research gap and cite prior work.\n"),
        ("03_methods.md", "Methods", 3, "## Methods\n\nDescribe materials, setup, datasets, and analysis methods.\n"),
        ("04_results.md", "Results", 4, "## Results\n\nReport findings with traceable evidence.\n"),
        ("05_discussion.md", "Discussion", 5, "## Discussion\n\nInterpret results, limitations, and implications.\n"),
        ("06_conclusion.md", "Conclusion", 6, "## Conclusion\n\nSummarize contribution and next work.\n"),
    ];
    data.into_iter()
        .map(|(filename, title, order, content)| ManuscriptSection {
            id: filename.trim_end_matches(".md").to_string(),
            filename: filename.to_string(),
            title: title.to_string(),
            order,
            content: content.to_string(),
            updated_at: now_iso(),
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
    fs::write(
        root.join("project.json"),
        serde_json::to_string_pretty(project).map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;
    fs::write(root.join("manuscript/paper.docx"), []).map_err(|err| err.to_string())?;
    fs::write(root.join("manuscript/main.tex"), basic_latex_template()).map_err(|err| err.to_string())?;
    fs::write(root.join("references/references.bib"), "").map_err(|err| err.to_string())?;
    fs::write(root.join("templates/word_template.docx"), []).map_err(|err| err.to_string())?;
    fs::write(root.join("ai/claims.json"), "[]").map_err(|err| err.to_string())?;
    fs::write(root.join("literature/literature.json"), "[]").map_err(|err| err.to_string())?;
    for section in initial_sections() {
        let path = root.join("manuscript/sections").join(&section.filename);
        if !path.exists() {
            fs::write(path, section.content).map_err(|err| err.to_string())?;
        }
    }
    Ok(())
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
    let workspace_root = input
        .workspace_root
        .clone()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or(settings.workspace_root);
    let workspace_path = PathBuf::from(workspace_root);
    let root_path = workspace_path.join(safe_folder_name(&input.title));
    let timestamp = now_iso();
    let citation_backend = citation_backend(&input.manuscript_mode);
    let project = ProjectConfig {
        id: format!("project_{}", Uuid::new_v4()),
        title: if input.title.trim().is_empty() { "Untitled Paper".to_string() } else { input.title.trim().to_string() },
        author: input.author.trim().to_string(),
        target_journal: input.target_journal.trim().to_string(),
        manuscript_mode: input.manuscript_mode,
        root_path: root_path.to_string_lossy().to_string(),
        created_at: timestamp.clone(),
        updated_at: timestamp,
        citation_backend,
    };
    ensure_structure(&project)?;
    let mut projects = read_registry()?;
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
    let mut projects = read_registry()?;
    projects = projects
        .into_iter()
        .map(|item| if item.id == project.id { project.clone() } else { item })
        .collect();
    write_registry(&projects)?;
    write_json(&PathBuf::from(&project.root_path).join("project.json"), &project)?;
    Ok(project)
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
    let project = projects.iter().find(|project| project.id == project_id).cloned();
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
    let project = project_by_id(&project_id)?;
    let mut sections = vec![];
    for template in initial_sections() {
        let path = PathBuf::from(&project.root_path)
            .join("manuscript/sections")
            .join(&template.filename);
        let content = if path.exists() {
            fs::read_to_string(&path).map_err(|err| err.to_string())?
        } else {
            template.content
        };
        sections.push(ManuscriptSection { content, ..template });
    }
    Ok(sections)
}

#[tauri::command]
fn read_section(project_id: String, filename: String) -> Result<ManuscriptSection, String> {
    list_sections(project_id)?
        .into_iter()
        .find(|section| section.filename == filename)
        .ok_or_else(|| "Section not found".to_string())
}

#[tauri::command]
fn save_section(project_id: String, section: ManuscriptSection) -> Result<ManuscriptSection, String> {
    let project = project_by_id(&project_id)?;
    let mut saved = section;
    saved.updated_at = now_iso();
    fs::write(
        PathBuf::from(&project.root_path)
            .join("manuscript/sections")
            .join(&saved.filename),
        &saved.content,
    )
    .map_err(|err| err.to_string())?;
    Ok(saved)
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
    let pattern = format!(r#"(?is){}\s*=\s*(?:\{{([^{{}}]*(?:\{{[^{{}}]*\}}[^{{}}]*)*)\}}|"([^"]*)")"#, field);
    Regex::new(&pattern)
        .ok()
        .and_then(|re| re.captures(body))
        .and_then(|captures| captures.get(1).or_else(|| captures.get(2)).map(|m| m.as_str().to_string()))
        .unwrap_or_default()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[tauri::command]
fn parse_bibtex(bibtex: String) -> Result<Vec<ReferenceItem>, String> {
    let entry_re = Regex::new(r"(?is)@\w+\s*\{\s*([^,\s]+)\s*,(.*?)(?=\n@\w+\s*\{|$)").map_err(|err| err.to_string())?;
    let mut refs = vec![];
    for capture in entry_re.captures_iter(&bibtex) {
        let citekey = capture.get(1).map(|m| m.as_str()).unwrap_or("").trim().to_string();
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
                if title.is_empty() { "(untitled)".to_string() } else { title }
            },
            authors,
            year: bib_field(body, "year"),
            journal: {
                let journal = bib_field(body, "journal");
                if journal.is_empty() { bib_field(body, "booktitle") } else { journal }
            },
            doi: bib_field(body, "doi"),
            abstract_text: {
                let value = bib_field(body, "abstract");
                if value.is_empty() { None } else { Some(value) }
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
    fs::write(PathBuf::from(&project.root_path).join("references/references.bib"), &bibtex).map_err(|err| err.to_string())?;
    let refs = parse_bibtex(bibtex)?;
    write_json(&PathBuf::from(&project.root_path).join("references/references.json"), &refs)?;
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
    let placeholder_re = Regex::new(r"\[CITE:\s*([A-Za-z0-9_:.+-]+)\s*\]").map_err(|err| err.to_string())?;
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
fn update_citation_task_status(project_id: String, task_id: String, status: CitationStatus) -> Result<Vec<CitationTask>, String> {
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
fn add_literature_item(project_id: String, item: LiteratureItemInput) -> Result<LiteratureItem, String> {
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
fn search_literature_mock(project_id: String, query: String) -> Result<Vec<LiteratureItem>, String> {
    let items = list_literature(project_id)?;
    let q = query.to_lowercase();
    if q.trim().is_empty() {
        return Ok(items);
    }
    Ok(items
        .into_iter()
        .filter(|item| {
            [item.filename.as_str(), item.path.as_str(), item.notes.as_str(), item.linked_citekey.as_deref().unwrap_or("")]
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
    write_json(&PathBuf::from(&project.root_path).join("ai/claims.json"), &claims)?;
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
fn update_claim_status(project_id: String, claim_id: String, status: ClaimStatus) -> Result<Vec<ClaimRecord>, String> {
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
fn apply_ai_proposal(project_id: String, proposal: AiProposal, section: ManuscriptSection) -> Result<ManuscriptSection, String> {
    let mut next = section;
    next.content = if proposal.original_text.trim().is_empty() {
        format!("{}\n\n{}\n", next.content.trim(), proposal.proposed_text)
    } else {
        next.content.replace(&proposal.original_text, &proposal.proposed_text)
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

#[tauri::command]
fn export_word_draft(project_id: String) -> Result<ExportJob, String> {
    let project = project_by_id(&project_id)?;
    let root = PathBuf::from(&project.root_path);
    let output = root.join("outputs/combined_word_draft.md");
    fs::write(&output, merged_sections(&project_id)?).map_err(|err| err.to_string())?;
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
    let body = merged_sections(&project_id)?
        .replace("[CITE: ", "\\cite{")
        .replace(']', "}");
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
        logs: vec!["LaTeX main.tex generated.".to_string(), "references.bib copied when available.".to_string()],
        created_at: now_iso(),
    })
}

#[tauri::command]
fn export_markdown_pandoc(project_id: String) -> Result<ExportJob, String> {
    let project = project_by_id(&project_id)?;
    let root = PathBuf::from(&project.root_path);
    let output = root.join("outputs/combined.md");
    fs::write(&output, merged_sections(&project_id)?).map_err(|err| err.to_string())?;
    let command = "pandoc combined.md --bibliography ../references/references.bib --csl ../references/csl/style.csl -o paper.docx";
    fs::write(root.join("outputs/pandoc_command.txt"), command).map_err(|err| err.to_string())?;
    Ok(ExportJob {
        id: format!("export_{}", Uuid::new_v4()),
        project_id,
        mode: ManuscriptMode::Markdown,
        status: ExportStatus::Success,
        output_path: output.to_string_lossy().to_string(),
        logs: vec!["combined.md generated.".to_string(), format!("Pandoc command: {}", command)],
        created_at: now_iso(),
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            create_project,
            list_projects,
            open_project,
            read_project_config,
            update_project_config,
            ensure_project_structure,
            delete_project,
            export_project_manifest,
            list_sections,
            read_section,
            save_section,
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
            read_settings,
            save_settings,
            generate_ai_proposal,
            apply_ai_proposal
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn main() {
    run();
}
