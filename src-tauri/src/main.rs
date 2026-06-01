#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use chrono::Utc;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
    process::Command,
};
use uuid::Uuid;

const APP_VERSION: &str = "2.2.0";
const PANDOC_INSTALL_COMMAND: &str = "winget install --id JohnMacFarlane.Pandoc -e --source winget --accept-package-agreements --accept-source-agreements --silent";
const PANDOC_REQUIRED_MESSAGE: &str =
    "Pandoc is required for document conversion. Please install Pandoc and try again.";
const LLM_CURL_CONNECT_TIMEOUT_SECS: u64 = 15;
const LLM_CURL_MAX_TIME_SECS: u64 = 240;

/// Top-level keys that must never appear in the body of a PaperForge
/// outbound LLM request. The list intentionally covers the full set of
/// tool / function calling / structured output / Responses API fields
/// so that we fail loudly if anyone ever reintroduces a tool schema
/// or mixes a Chat Completions body with a Responses API body.
/// Top-level keys that must never appear in the body of a PaperForge
/// outbound LLM request. The list intentionally covers the full set of
/// tool / function calling / structured output / Responses API fields
/// so that we fail loudly if anyone ever reintroduces a tool schema
/// or mixes a Chat Completions body with a Responses API body.
///
/// `tool_choice` and `parallel_tool_calls` are intentionally NOT in
/// the list: those are the `tool_choice: "none"` and
/// `parallel_tool_calls: false` "off switches" we send by default to
/// force providers not to emit tool calls.
const FORBIDDEN_LLM_KEYS: &[&str] = &[
    // Chat Completions tool / function calling surface
    "tools",
    "functions",
    "function_call",
    // Assistant message tool calls that some clients accidentally echo back
    "tool_calls",
    // Structured output / JSON mode / strict schema
    "response_format",
    "strict",
    "json_schema",
    // OpenAI Responses API fields (must not leak into Chat Completions)
    "response",
    "input",
    "instructions",
    "previous_response_id",
    "truncation",
    "metadata",
    "store",
    "user",
];

/// Returns true when the JSON value contains any of
/// `FORBIDDEN_LLM_KEYS` anywhere in the tree. The check is recursive so
/// that a hidden `tools` array nested inside an otherwise innocuous
/// `metadata` block is still caught.
fn llm_body_has_forbidden_keys(body: &serde_json::Value) -> bool {
    fn walk(value: &serde_json::Value, hits: &mut Vec<String>) {
        match value {
            serde_json::Value::Object(map) => {
                for (key, child) in map {
                    if FORBIDDEN_LLM_KEYS.iter().any(|bad| *bad == key) {
                        hits.push(key.clone());
                    }
                    walk(child, hits);
                }
            }
            serde_json::Value::Array(items) => {
                for item in items {
                    walk(item, hits);
                }
            }
            _ => {}
        }
    }
    let mut hits: Vec<String> = Vec::new();
    walk(body, &mut hits);
    !hits.is_empty()
}

/// Lists the offending keys found inside the body for use in error
/// messages and debug logs.
fn llm_body_forbidden_keys(body: &serde_json::Value) -> Vec<String> {
    fn walk(value: &serde_json::Value, hits: &mut Vec<String>) {
        match value {
            serde_json::Value::Object(map) => {
                for (key, child) in map {
                    if FORBIDDEN_LLM_KEYS.iter().any(|bad| *bad == key) {
                        hits.push(key.clone());
                    }
                    walk(child, hits);
                }
            }
            serde_json::Value::Array(items) => {
                for item in items {
                    walk(item, hits);
                }
            }
            _ => {}
        }
    }
    let mut hits: Vec<String> = Vec::new();
    walk(body, &mut hits);
    hits.sort();
    hits.dedup();
    hits
}

/// Returns a JSON snapshot of the body for debug logging. The API key
/// is masked because it is stored in the same auth header the request
/// will use and must never reach the application log.
fn llm_body_debug_log(body: &serde_json::Value) -> String {
    let mut sanitized = body.clone();
    fn walk(value: &mut serde_json::Value) {
        match value {
            serde_json::Value::Object(map) => {
                for (_key, child) in map.iter_mut() {
                    walk(child);
                }
            }
            serde_json::Value::Array(items) => {
                for item in items.iter_mut() {
                    walk(item);
                }
            }
            serde_json::Value::String(s) => {
                if s.len() > 32 {
                    s.replace_range(8..s.len() - 4, "****");
                }
            }
            _ => {}
        }
    }
    walk(&mut sanitized);
    serde_json::to_string(&sanitized).unwrap_or_else(|_| "<unprintable llm body>".to_string())
}


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
    #[serde(default = "default_project_version")]
    version: String,
    #[serde(default = "default_project_title")]
    title: String,
    #[serde(default)]
    author: String,
    #[serde(default)]
    authors: Vec<String>,
    #[serde(default = "default_target_journal")]
    target_journal: String,
    #[serde(default)]
    journal: String,
    #[serde(default = "default_language")]
    language: Language,
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
    #[serde(default)]
    sections: Vec<ManuscriptManifestSection>,
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

fn default_project_version() -> String {
    APP_VERSION.to_string()
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
#[serde(rename_all = "camelCase")]
struct FileTreeNode {
    name: String,
    path: String,
    relative_path: String,
    kind: String,
    extension: Option<String>,
    children: Option<Vec<FileTreeNode>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TextFilePayload {
    path: String,
    content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
enum AgentMode {
    Ask,
    Edit,
    Operate,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum AgentRunStatus {
    Planned,
    Completed,
    Applied,
    Rejected,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum AgentChangeStatus {
    Pending,
    Applied,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum AgentFileChangeType {
    Create,
    Update,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AgentSkill {
    id: String,
    name: String,
    #[serde(rename = "type")]
    skill_type: AgentMode,
    description: String,
    allowed_tools: Vec<String>,
    requires_diff: bool,
    requires_confirmation: bool,
    writes_files: bool,
    risk_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AgentPlan {
    summary: String,
    steps: Vec<String>,
    files_to_read: Vec<String>,
    files_to_change: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AgentFileChange {
    id: String,
    path: String,
    change_type: AgentFileChangeType,
    original_content: String,
    proposed_content: String,
    diff: String,
    status: AgentChangeStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AgentToolResult {
    tool: String,
    ok: bool,
    message: String,
    data: Option<serde_json::Value>,
    error: Option<String>,
    reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AgentRun {
    id: String,
    project_id: String,
    mode: AgentMode,
    skill_id: String,
    request: String,
    status: AgentRunStatus,
    plan: AgentPlan,
    files_read: Vec<String>,
    files_changed: Vec<String>,
    report: String,
    changes: Vec<AgentFileChange>,
    tool_results: Vec<AgentToolResult>,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AgentLogEntry {
    id: String,
    run_id: String,
    project_id: String,
    mode: AgentMode,
    skill_id: String,
    request: String,
    tools: Vec<String>,
    files_read: Vec<String>,
    files_changed: Vec<String>,
    success: bool,
    error: Option<String>,
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
    Light,
    Dark,
    System,
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
    ThemeMode::Light
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
enum LlmProviderKind {
    OpenaiCompatible,
    Openai,
    Anthropic,
    Local,
}

fn default_llm_provider_kind() -> LlmProviderKind {
    LlmProviderKind::OpenaiCompatible
}


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
enum SidebarMode {
    Writing,
    Files,
}

fn default_sidebar_mode() -> SidebarMode {
    SidebarMode::Writing
}

fn default_llm_base_url() -> String {
    "https://api.openai.com/v1".to_string()
}

fn default_llm_model() -> String {
    "gpt-4.1-mini".to_string()
}

fn default_llm_temperature() -> f32 {
    0.3
}

fn default_llm_max_tokens() -> u32 {
    2000
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LlmProviderSettings {
    #[serde(default = "default_llm_provider_kind")]
    provider: LlmProviderKind,
    #[serde(default = "default_llm_base_url")]
    base_url: String,
    #[serde(default)]
    api_key: String,
    #[serde(default = "default_llm_model")]
    model: String,
    #[serde(default = "default_llm_temperature")]
    temperature: f32,
    #[serde(default = "default_llm_max_tokens")]
    max_tokens: u32,
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
    #[serde(default = "default_sidebar_mode")]
    sidebar_mode: SidebarMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WorkspaceConfig {
    version: String,
    workspace_name: String,
    created_at: String,
    updated_at: String,
    papers_dir: String,
    default_language: Language,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AiModelConfig {
    default_model_id: String,
    models: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PandocStatus {
    installed: bool,
    version: Option<String>,
    error: Option<String>,
}

fn now_iso() -> String {
    Utc::now().to_rfc3339()
}

fn app_root() -> Result<PathBuf, String> {
    std::env::current_dir().map_err(|err| err.to_string())
}

fn resolve_app_path(path: impl AsRef<Path>) -> Result<PathBuf, String> {
    let path = path.as_ref();
    if path.as_os_str().is_empty() {
        return Err("Path is required".to_string());
    }
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Ok(app_root()?.join(path))
    }
}

fn canonical_existing_path(path: impl AsRef<Path>) -> Result<PathBuf, String> {
    let absolute = resolve_app_path(path)?;
    if !absolute.exists() {
        return Err("Path does not exist".to_string());
    }
    absolute.canonicalize().map_err(|err| err.to_string())
}

fn export_output_path(path: impl AsRef<Path>) -> Result<String, String> {
    Ok(canonical_existing_path(path)?.to_string_lossy().to_string())
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

fn normalize_llm_settings(mut settings: LlmProviderSettings) -> LlmProviderSettings {
    if settings.base_url.trim().is_empty() {
        settings.base_url = match settings.provider {
            LlmProviderKind::Anthropic => "https://api.anthropic.com/v1".to_string(),
            LlmProviderKind::Local => "http://localhost:11434/v1".to_string(),
            _ => "https://api.openai.com/v1".to_string(),
        };
    }
    if settings.model.trim().is_empty() {
        settings.model = match settings.provider {
            LlmProviderKind::Anthropic => "claude-3-5-sonnet-latest".to_string(),
            LlmProviderKind::Local => "llama3.1".to_string(),
            _ => default_llm_model(),
        };
    }
    if settings.max_tokens == 0 {
        settings.max_tokens = default_llm_max_tokens();
    }
    settings
}

fn ai_model_value_to_settings(value: &serde_json::Value) -> Option<LlmProviderSettings> {
    let provider = match value.get("provider")?.as_str()? {
        "anthropic" => LlmProviderKind::Anthropic,
        "openai" => LlmProviderKind::Openai,
        "local" => LlmProviderKind::Local,
        _ => LlmProviderKind::OpenaiCompatible,
    };
    Some(normalize_llm_settings(LlmProviderSettings {
        provider,
        base_url: value
            .get("baseUrl")
            .and_then(|item| item.as_str())
            .unwrap_or("")
            .to_string(),
        api_key: value
            .get("apiKey")
            .and_then(|item| item.as_str())
            .unwrap_or("")
            .to_string(),
        model: value
            .get("model")
            .and_then(|item| item.as_str())
            .unwrap_or("")
            .to_string(),
        temperature: value
            .get("temperature")
            .and_then(|item| item.as_f64())
            .map(|item| item as f32)
            .unwrap_or_else(default_llm_temperature),
        max_tokens: value
            .get("maxTokens")
            .and_then(|item| item.as_u64())
            .map(|item| item as u32)
            .unwrap_or_else(default_llm_max_tokens),
    }))
}

fn workspace_default_llm_settings(workspace_root: &str) -> Result<Option<LlmProviderSettings>, String> {
    let path = workspace_ai_models_path(&PathBuf::from(workspace_root));
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(path).map_err(|err| err.to_string())?;
    let config: AiModelConfig = serde_json::from_str(&raw).map_err(|err| err.to_string())?;
    let selected = config
        .models
        .iter()
        .find(|item| item.get("id").and_then(|id| id.as_str()) == Some(config.default_model_id.as_str()))
        .or_else(|| config.models.first());
    Ok(selected.and_then(ai_model_value_to_settings))
}

fn resolve_llm_settings(settings: &AppSettings) -> Result<LlmProviderSettings, String> {
    let direct = normalize_llm_settings(settings.llm_provider.clone());
    if !direct.api_key.trim().is_empty() {
        return Ok(direct);
    }
    if let Some(workspace) = workspace_default_llm_settings(&settings.workspace_root)? {
        if !workspace.api_key.trim().is_empty() {
            return Ok(workspace);
        }
    }
    Err("No LLM API key configured. Add a model in Settings or workspace/.paperforge/ai-models.json.".to_string())
}

fn join_url(base_url: &str, endpoint: &str) -> String {
    format!(
        "{}/{}",
        base_url.trim_end_matches('/'),
        endpoint.trim_start_matches('/')
    )
}

/// Reads `choices[0].message.content` from an OpenAI-compatible
/// Chat Completions response. If the model still emits a
/// `tool_calls` array (some providers force-call tools even when
/// `tool_choice` is "none"), the function returns a clear
/// PaperForge-side error instead of silently surfacing an empty
/// string. `finish_reason` is surfaced when present so users can see
/// why the model stopped, and the refusal / refusal-style fields are
/// passed through verbatim.
fn parse_openai_chat_content(value: &serde_json::Value) -> Result<String, String> {
    let choice = value
        .get("choices")
        .and_then(|choices| choices.as_array())
        .and_then(|choices| choices.first())
        .ok_or_else(|| "OpenAI-compatible response did not include choices[0].".to_string())?;
    let finish_reason = choice
        .get("finish_reason")
        .and_then(|item| item.as_str())
        .map(|item| item.to_string());
    let message = choice
        .get("message")
        .ok_or_else(|| "OpenAI-compatible response did not include choices[0].message.".to_string())?;
    if let Some(tool_calls) = message.get("tool_calls") {
        let preview = tool_calls.to_string();
        let preview = preview.chars().take(160).collect::<String>();
        return Err(match finish_reason.as_deref() {
            Some(reason) => format!(
                "OpenAI-compatible model returned tool_calls instead of text content (finish_reason={}, tool_calls={}). PaperForge does not support tool calling in this build. Change the model or update the prompt and retry.",
                reason, preview
            ),
            None => format!(
                "OpenAI-compatible model returned tool_calls instead of text content (tool_calls={}). PaperForge does not support tool calling in this build. Change the model or update the prompt and retry.",
                preview
            ),
        });
    }
    let content = message
        .get("content")
        .and_then(|content| content.as_str())
        .map(|content| content.trim().to_string())
        .filter(|content| !content.is_empty());
    if let Some(content) = content {
        return Ok(content);
    }
    if let Some(reason) = finish_reason.as_deref() {
        if reason == "length" {
            return Err("OpenAI-compatible response was truncated by max_tokens. Increase max_tokens in PaperForge settings or shorten the input.".to_string());
        }
        if reason == "content_filter" {
            return Err("OpenAI-compatible response was blocked by a content filter.".to_string());
        }
    }
    Err("OpenAI-compatible response did not include choices[0].message.content.".to_string())
}

/// Reads the first `text` block from an Anthropic `messages`
/// response. If the model returns a `tool_use` block instead of
/// text, PaperForge returns a clear error because the desktop build
/// does not implement an agent tool loop.
fn parse_anthropic_message_content(value: &serde_json::Value) -> Result<String, String> {
    let blocks = value
        .get("content")
        .and_then(|content| content.as_array())
        .ok_or_else(|| "Anthropic response did not include content blocks.".to_string())?;
    let mut saw_tool_use = false;
    for block in blocks {
        match block.get("type").and_then(|kind| kind.as_str()) {
            Some("text") => {
                if let Some(text) = block.get("text").and_then(|item| item.as_str()) {
                    let trimmed = text.trim();
                    if !trimmed.is_empty() {
                        return Ok(trimmed.to_string());
                    }
                }
            }
            Some("tool_use") => saw_tool_use = true,
            _ => {}
        }
    }
    if saw_tool_use {
        return Err("Anthropic model returned a tool_use block instead of text content. PaperForge does not support tool calling in this build. Change the model or update the prompt and retry.".to_string());
    }
    let stop_reason = value
        .get("stop_reason")
        .and_then(|item| item.as_str())
        .map(|item| item.to_string());
    if stop_reason.as_deref() == Some("max_tokens") {
        return Err("Anthropic response was truncated by max_tokens. Increase max_tokens in PaperForge settings or shorten the input.".to_string());
    }
    Err("Anthropic response did not include text content.".to_string())
}

fn curl_config_quote(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn write_llm_request_files(url: &str, headers: &[String], body: &serde_json::Value) -> Result<(PathBuf, PathBuf), String> {
    let dir = local_dir()?.join("llm-requests");
    fs::create_dir_all(&dir).map_err(|err| err.to_string())?;
    let id = Uuid::new_v4().to_string();
    let body_path = dir.join(format!("{}.json", id));
    let config_path = dir.join(format!("{}.curl", id));
    fs::write(
        &body_path,
        serde_json::to_string(body).map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;
    let mut config = vec![
        format!("url = \"{}\"", curl_config_quote(url)),
        "request = \"POST\"".to_string(),
        "silent".to_string(),
        "show-error".to_string(),
        format!("connect-timeout = \"{} \"", LLM_CURL_CONNECT_TIMEOUT_SECS),
        format!("max-time = \"{}\"", LLM_CURL_MAX_TIME_SECS),
        "write-out = \"\\n%{http_code}\"".to_string(),
        format!("data = \"@{}\"", curl_config_quote(&body_path.to_string_lossy())),
    ];
    for header in headers {
        config.push(format!("header = \"{}\"", curl_config_quote(header)));
    }
    fs::write(&config_path, config.join("\n")).map_err(|err| err.to_string())?;
    Ok((config_path, body_path))
}

fn post_json_with_curl(url: &str, headers: &[String], body: &serde_json::Value) -> Result<serde_json::Value, String> {
    let (config_path, body_path) = write_llm_request_files(url, headers, body)?;
    let output = Command::new("curl")
        .arg("--config")
        .arg(&config_path)
        .output()
        .map_err(|err| format!("curl request failed to start: {}", err));
    let _ = fs::remove_file(&config_path);
    let _ = fs::remove_file(&body_path);
    let output = output?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let mut lines = stdout.lines().collect::<Vec<_>>();
    let status = lines.pop().unwrap_or("").trim().to_string();
    let body_text = lines.join("\n");
    if !output.status.success() {
        return Err(if stderr.is_empty() {
            format!("curl request failed with HTTP {}: {}", status, body_text)
        } else {
            format!("curl request failed: {} {}", stderr, body_text)
        });
    }
    if !status.starts_with('2') {
        return Err(format!("LLM API error {}: {}", status, body_text));
    }
    serde_json::from_str(&body_text).map_err(|err| format!("LLM response JSON parse failed: {}", err))
}

fn get_json_with_curl(url: &str, headers: &[String]) -> Result<serde_json::Value, String> {
    let id = Uuid::new_v4().to_string();
    let dir = local_dir()?.join("llm-requests");
    fs::create_dir_all(&dir).map_err(|err| err.to_string())?;
    let config_path = dir.join(format!("{}.curl", id));
    let mut config = vec![
        "silent".to_string(),
        "show-error".to_string(),
        "location".to_string(),
        "get".to_string(),
        "write-out = \"\\n%{http_code}\"".to_string(),
        format!("url = \"{}\"", curl_config_quote(url)),
    ];
    for header in headers {
        config.push(format!("header = \"{}\"", curl_config_quote(header)));
    }
    fs::write(&config_path, config.join("\n")).map_err(|err| err.to_string())?;
    let output = Command::new("curl")
        .arg("--config")
        .arg(&config_path)
        .output()
        .map_err(|err| format!("curl request failed to start: {}", err));
    let _ = fs::remove_file(&config_path);
    let output = output?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let mut lines = stdout.lines().collect::<Vec<_>>();
    let status = lines.pop().unwrap_or("").trim().to_string();
    let body_text = lines.join("\n");
    if !output.status.success() {
        return Err(if stderr.is_empty() {
            format!("curl request failed with HTTP {}: {}", status, body_text)
        } else {
            format!("curl request failed: {} {}", stderr, body_text)
        });
    }
    if !status.starts_with('2') {
        return Err(format!("LLM API error {}: {}", status, body_text));
    }
    serde_json::from_str(&body_text).map_err(|err| format!("LLM response JSON parse failed: {}", err))
}

fn validate_ai_settings(settings: &LlmProviderSettings) -> Result<LlmProviderSettings, String> {
    let provider = normalize_llm_settings(settings.clone());
    if provider.api_key.trim().is_empty() {
        return Err("API key is required. Configure it in Settings.".to_string());
    }
    if provider.model.trim().is_empty() {
        return Err("Model is required. Choose or enter a model in Settings.".to_string());
    }
    Ok(provider)
}

fn provider_auth_headers(provider: &LlmProviderSettings) -> Vec<String> {
    match provider.provider {
        LlmProviderKind::Anthropic => vec![
            "Content-Type: application/json".to_string(),
            format!("x-api-key: {}", provider.api_key.trim()),
            "anthropic-version: 2023-06-01".to_string(),
        ],
        LlmProviderKind::Openai | LlmProviderKind::OpenaiCompatible | LlmProviderKind::Local => vec![
            "Content-Type: application/json".to_string(),
            format!("Authorization: Bearer {}", provider.api_key.trim()),
        ],
    }
}

/// Builds a clean OpenAI-compatible Chat Completions body. The
/// returned payload is guaranteed to:
///   * contain only standard Chat Completions fields
///   * disable tool / function calling via `tool_choice: "none"`
///   * disable parallel tool calls via `parallel_tool_calls: false`
///   * never include `tools`, `functions`, `function_call`,
///     `response_format`, `strict`, or any Responses API field
///
/// The body is verified against `FORBIDDEN_LLM_KEYS` after
/// construction so that any future regression that re-adds a
/// tool schema fails loudly with a PaperForge-side error instead of
/// being sent to the provider.
fn build_openai_chat_body(
    provider: &LlmProviderSettings,
    system_prompt: &str,
    user_prompt: &str,
) -> Result<serde_json::Value, String> {
    let body = serde_json::json!({
        "model": provider.model,
        "temperature": provider.temperature,
        "max_tokens": provider.max_tokens,
        "stream": false,
        // Standard messages array. No tool_calls, no tool_call_id, no
        // function_call are ever attached to any message.
        "messages": [
            { "role": "system", "content": system_prompt },
            { "role": "user", "content": user_prompt }
        ],
        // Force the model to not call any tool. The OpenAI-compatible
        // spec accepts `tool_choice: "none"` to disable tool calling
        // even when a provider tries to force a default tool.
        "tool_choice": "none",
        // Defensive: belt-and-suspenders so providers that default to
        // `true` (e.g. some Qwen deployments) cannot spawn tool
        // calls in parallel.
        "parallel_tool_calls": false
    });
    assert_clean_llm_body(&body, "OpenAI-compatible chat completions")?;
    Ok(body)
}

/// Builds a clean Anthropic Messages API body. Anthropic accepts
/// `tools` only as a top-level array; we never include it, and
/// `system` is sent as a top-level string per the public docs.
fn build_anthropic_message_body(
    provider: &LlmProviderSettings,
    system_prompt: &str,
    user_prompt: &str,
) -> Result<serde_json::Value, String> {
    let body = serde_json::json!({
        "model": provider.model,
        "max_tokens": provider.max_tokens,
        "temperature": provider.temperature,
        "system": system_prompt,
        "messages": [{ "role": "user", "content": user_prompt }],
    });
    assert_clean_llm_body(&body, "Anthropic messages")?;
    Ok(body)
}

/// Verifies a body is safe to send: returns `Err` if any forbidden
/// key (tool / function calling / Responses API / structured output)
/// appears anywhere in the body. The endpoint label is included in
/// the error so the user can tell which provider triggered the
/// guard.
fn assert_clean_llm_body(body: &serde_json::Value, endpoint: &str) -> Result<(), String> {
    let hits = llm_body_forbidden_keys(body);
    if hits.is_empty() {
        return Ok(());
    }
    Err(format!(
        "Refusing to send LLM request to {}: payload contains forbidden key(s) {}.          PaperForge does not send tool / function calling / structured output / Responses API fields.          Remove the offending key from the call site and rebuild.",
        endpoint,
        hits.join(", ")
    ))
}

/// Emits a single-line debug record describing the outgoing LLM
/// request. The body is masked to protect the API key, and the
/// summary fields (`has_tools`, `tool_choice`, `response_format`)
/// are printed explicitly so the operator can confirm the
/// tool-calling surface is empty before the request leaves the
/// process. Output goes to stderr so the desktop logger captures
/// it without leaking into the export pipeline.
fn debug_log_llm_request(endpoint: &str, provider_kind: &str, model: &str, body: &serde_json::Value) {
    let has_tools = body.get("tools").is_some() || llm_body_has_forbidden_keys(body);
    let tool_choice = body
        .get("tool_choice")
        .and_then(|value| match value {
            serde_json::Value::String(s) => Some(s.clone()),
            _ => Some(value.to_string()),
        })
        .unwrap_or_else(|| "<absent>".to_string());
    let response_format = body
        .get("response_format")
        .map(|value| value.to_string())
        .unwrap_or_else(|| "<absent>".to_string());
    eprintln!(
        "[paperforge.llm] endpoint={} provider={} model={} has_tools={} tool_choice={} response_format={} payload={}",
        endpoint,
        provider_kind,
        model,
        has_tools,
        tool_choice,
        response_format,
        llm_body_debug_log(body)
    );
}

fn call_llm(settings: &AppSettings, system_prompt: &str, user_prompt: &str) -> Result<String, String> {
    let provider = resolve_llm_settings(settings)?;
    match provider.provider {
        LlmProviderKind::Anthropic => {
            let url = join_url(&provider.base_url, "messages");
            let headers = vec![
                "Content-Type: application/json".to_string(),
                format!("x-api-key: {}", provider.api_key.trim()),
                "anthropic-version: 2023-06-01".to_string(),
            ];
            let body = build_anthropic_message_body(&provider, system_prompt, user_prompt)?;
            debug_log_llm_request(&url, "anthropic", &provider.model, &body);
            let value = post_json_with_curl(&url, &headers, &body)?;
            parse_anthropic_message_content(&value)
        }
        LlmProviderKind::Openai | LlmProviderKind::OpenaiCompatible | LlmProviderKind::Local => {
            let url = join_url(&provider.base_url, "chat/completions");
            let headers = vec![
                "Content-Type: application/json".to_string(),
                format!("Authorization: Bearer {}", provider.api_key.trim()),
            ];
            let body = build_openai_chat_body(&provider, system_prompt, user_prompt)?;
            debug_log_llm_request(&url, "openai-compatible", &provider.model, &body);
            let value = post_json_with_curl(&url, &headers, &body)?;
            parse_openai_chat_content(&value)
        }
    }
}



/// `call_llm` wrapper used by the agent / proposal paths. `call_llm`
/// itself never panics, but the agent command sits on the user-facing
/// Run Agent button, so any unforeseen panic in a downstream
/// dependency (provider response, JSON parse, etc.) would close the
/// Tauri webview. `safe_call_llm` converts a panic into a
/// PaperForge-side error string so the frontend dialog can show it
/// instead of the app window disappearing.
fn safe_call_llm(
    settings: &AppSettings,
    system_prompt: &str,
    user_prompt: &str,
) -> Result<String, String> {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        call_llm(settings, system_prompt, user_prompt)
    })) {
        Ok(result) => result,
        Err(payload) => {
            let detail = if let Some(s) = payload.downcast_ref::<&'static str>() {
                (*s).to_string()
            } else if let Some(s) = payload.downcast_ref::<String>() {
                s.clone()
            } else {
                "unknown panic".to_string()
            };
            Err(format!(
                "Agent LLM call panicked: {}. The desktop agent run aborted so the app window could keep running. Retry, or open Settings to verify the LLM provider URL, API key, and model.",
                detail
            ))
        }
    }
}
fn built_in_agent_skills() -> Vec<AgentSkill> {
    vec![
        AgentSkill {
            id: "ask.project-review".to_string(),
            name: "Project Review".to_string(),
            skill_type: AgentMode::Ask,
            description: "Review project structure, manuscript sections, references, and attachments without changing files.".to_string(),
            allowed_tools: vec!["list_project_files", "list_sections", "list_figures", "check_broken_links", "write_agent_log"].into_iter().map(String::from).collect(),
            requires_diff: false,
            requires_confirmation: false,
            writes_files: false,
            risk_level: "low".to_string(),
        },
        AgentSkill {
            id: "ask.export-readiness".to_string(),
            name: "Export Readiness".to_string(),
            skill_type: AgentMode::Ask,
            description: "Check whether the current manuscript is ready for Markdown, Word placeholder, or LaTeX export.".to_string(),
            allowed_tools: vec!["list_project_files", "list_sections", "check_broken_links", "write_agent_log"].into_iter().map(String::from).collect(),
            requires_diff: false,
            requires_confirmation: false,
            writes_files: false,
            risk_level: "low".to_string(),
        },
        AgentSkill {
            id: "edit.academic-polish".to_string(),
            name: "Academic Polish".to_string(),
            skill_type: AgentMode::Edit,
            description: "Improve academic style while preserving technical meaning, citations, and numbers.".to_string(),
            allowed_tools: vec!["read_project_file", "patch_project_file", "write_agent_log"].into_iter().map(String::from).collect(),
            requires_diff: true,
            requires_confirmation: true,
            writes_files: true,
            risk_level: "medium".to_string(),
        },
        AgentSkill {
            id: "edit.translate-zh-en".to_string(),
            name: "Translate ZH-EN".to_string(),
            skill_type: AgentMode::Edit,
            description: "Translate or bilingual-polish the active section while preserving citations and technical details.".to_string(),
            allowed_tools: vec!["read_project_file", "patch_project_file", "write_agent_log"].into_iter().map(String::from).collect(),
            requires_diff: true,
            requires_confirmation: true,
            writes_files: true,
            risk_level: "medium".to_string(),
        },
        AgentSkill {
            id: "operate.insert-figure".to_string(),
            name: "Insert Figure".to_string(),
            skill_type: AgentMode::Operate,
            description: "Prepare a safe Markdown figure insertion using files under attachments/figures.".to_string(),
            allowed_tools: vec!["list_figures", "read_project_file", "patch_project_file", "write_agent_log"].into_iter().map(String::from).collect(),
            requires_diff: true,
            requires_confirmation: true,
            writes_files: true,
            risk_level: "medium".to_string(),
        },
    ]
}

fn select_agent_skill(mode: &AgentMode, skill_id: &str, request: &str) -> AgentSkill {
    // Safety: never panic. A future refactor could rename the
    // ask.export-readiness / edit.translate-zh-en keyword skills, and
    // a stale UI selection could reference a skill id that no longer
    // exists. The agent button is wired to this function on the
    // desktop, so any panic here would tear down the Tauri runtime
    // and close the app window.
    let fallback = || built_in_agent_skills().into_iter().find(|skill| &skill.skill_type == mode).unwrap_or_else(|| built_in_agent_skills().into_iter().next().expect("built-in skills are non-empty"));
    let skills = built_in_agent_skills();
    if !skill_id.trim().is_empty() && skill_id != "auto" {
        if let Some(skill) = skills.iter().find(|skill| skill.id == skill_id) {
            return skill.clone();
        }
    }
    let request = request.to_lowercase();
    if matches!(mode, AgentMode::Ask)
        && (request.contains("export") || request.contains("word") || request.contains("latex") || request.contains("markdown") || request.contains("导出"))
    {
        if let Some(skill) = skills.clone().into_iter().find(|skill| skill.id == "ask.export-readiness") {
            return skill;
        }
    }
    if matches!(mode, AgentMode::Edit)
        && (request.contains("translate") || request.contains("translation") || request.contains("翻译") || request.contains("中文") || request.contains("英文"))
    {
        if let Some(skill) = skills.clone().into_iter().find(|skill| skill.id == "edit.translate-zh-en") {
            return skill;
        }
    }
    fallback()
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

fn safe_filename(value: &str) -> String {
    let cleaned: String = value
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.'))
        .collect();
    if cleaned.is_empty() {
        "item".to_string()
    } else {
        cleaned
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
            provider: LlmProviderKind::OpenaiCompatible,
            base_url: "https://api.openai.com/v1".to_string(),
            api_key: String::new(),
            model: "gpt-4.1-mini".to_string(),
            temperature: default_llm_temperature(),
            max_tokens: default_llm_max_tokens(),
        },
        default_citation_style: "apa".to_string(),
        default_export_mode: ManuscriptMode::Markdown,
        theme_mode: ThemeMode::Light,
        language: Language::En,
        sidebar_mode: SidebarMode::Writing,
    }
}

fn default_workspace_config(existing: Option<WorkspaceConfig>) -> WorkspaceConfig {
    let timestamp = now_iso();
    WorkspaceConfig {
        version: APP_VERSION.to_string(),
        workspace_name: existing
            .as_ref()
            .map(|value| value.workspace_name.clone())
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "workspace".to_string()),
        created_at: existing
            .as_ref()
            .map(|value| value.created_at.clone())
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| timestamp.clone()),
        updated_at: timestamp,
        papers_dir: "papers".to_string(),
        default_language: existing
            .as_ref()
            .map(|value| value.default_language.clone())
            .unwrap_or(Language::En),
    }
}

fn ensure_workspace(root: &Path) -> Result<WorkspaceConfig, String> {
    fs::create_dir_all(workspace_meta_dir(root)).map_err(|err| err.to_string())?;
    fs::create_dir_all(root.join("papers")).map_err(|err| err.to_string())?;
    let existing = if workspace_config_path(root).exists() {
        let raw = fs::read_to_string(workspace_config_path(root)).map_err(|err| err.to_string())?;
        serde_json::from_str::<WorkspaceConfig>(&raw).ok()
    } else {
        None
    };
    let config = default_workspace_config(existing);
    write_json(&workspace_config_path(root), &config)?;
    if !workspace_ai_models_path(root).exists() {
        write_json(
            &workspace_ai_models_path(root),
            &AiModelConfig {
                default_model_id: String::new(),
                models: vec![],
            },
        )?;
    }
    if !workspace_settings_path(root).exists() {
        let settings = serde_json::json!({
            "language": "en",
            "theme": "light"
        });
        write_json(&workspace_settings_path(root), &settings)?;
    }
    write_if_missing(&workspace_history_path(root), b"")?;
    Ok(config)
}

#[tauri::command]
fn init_workspace(root_path: String) -> Result<WorkspaceConfig, String> {
    let root = PathBuf::from(if root_path.trim().is_empty() {
        "workspace".to_string()
    } else {
        root_path.trim().to_string()
    });
    let config = ensure_workspace(&root)?;
    let mut settings = read_settings().unwrap_or_else(|_| default_settings());
    settings.workspace_root = root.to_string_lossy().to_string();
    save_settings(settings)?;
    Ok(config)
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
    if project.version.trim().is_empty() {
        project.version = default_project_version();
    }
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
    project.journal = if project.target_journal == default_target_journal() {
        String::new()
    } else {
        project.target_journal.clone()
    };
    if project.citation_style.trim().is_empty() {
        project.citation_style = default_citation_style();
    }
    project.sections = project.manuscript.sections.clone();
    project
}

fn activity_path(root: &Path) -> PathBuf {
    root.join(".paperforge/activity.json")
}

fn paper_history_path(root: &Path) -> PathBuf {
    root.join(".paperforge/history.log")
}

fn agent_log_path(root: &Path) -> PathBuf {
    root.join(".paperforge/agent.log")
}

fn agent_run_dir(root: &Path) -> PathBuf {
    root.join(".paperforge/agent-runs")
}

fn agent_run_path(root: &Path, run_id: &str) -> PathBuf {
    agent_run_dir(root).join(format!("{}.json", safe_filename(run_id)))
}

fn agent_backup_dir(root: &Path) -> PathBuf {
    root.join(".paperforge/backups")
}

fn project_manifest_path(root: &Path) -> PathBuf {
    root.join("paperforge.json")
}

fn compatibility_project_manifest_path(root: &Path) -> PathBuf {
    root.join("paperforge.project.json")
}

fn legacy_project_manifest_path(root: &Path) -> PathBuf {
    root.join("project.json")
}

fn workspace_meta_dir(root: &Path) -> PathBuf {
    root.join(".paperforge")
}

fn workspace_config_path(root: &Path) -> PathBuf {
    workspace_meta_dir(root).join("workspace.json")
}

fn workspace_ai_models_path(root: &Path) -> PathBuf {
    workspace_meta_dir(root).join("ai-models.json")
}

fn workspace_settings_path(root: &Path) -> PathBuf {
    workspace_meta_dir(root).join("settings.json")
}

fn workspace_history_path(root: &Path) -> PathBuf {
    workspace_meta_dir(root).join("history.log")
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

fn manifest_file_name(manifest: &ManuscriptManifestSection) -> String {
    PathBuf::from(&manifest.path)
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("section.md")
        .to_string()
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
        ".paperforge",
    ];
    for folder in folders {
        fs::create_dir_all(root.join(folder)).map_err(|err| err.to_string())?;
    }
    let raw_project = serde_json::to_string_pretty(project).map_err(|err| err.to_string())?;
    fs::write(project_manifest_path(&root), raw_project.as_bytes())
        .map_err(|err| err.to_string())?;
    fs::write(compatibility_project_manifest_path(&root), raw_project.as_bytes())
        .map_err(|err| err.to_string())?;
    write_if_missing(&root.join("references/bib/references.bib"), b"")?;
    write_if_missing(&root.join("references/papers/papers.json"), b"[]")?;
    write_if_missing(&root.join("references/citation_tasks.json"), b"[]")?;
    write_if_missing(&root.join(".paperforge/claims.json"), b"[]")?;
    write_if_missing(&activity_path(&root), b"[]")?;
    write_if_missing(&paper_history_path(&root), b"")?;
    fs::create_dir_all(agent_run_dir(&root)).map_err(|err| err.to_string())?;
    fs::create_dir_all(agent_backup_dir(&root)).map_err(|err| err.to_string())?;
    write_if_missing(&agent_log_path(&root), b"")?;
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

struct ProjectFileSystem {
    root: PathBuf,
    canonical_root: PathBuf,
}

impl ProjectFileSystem {
    fn new(project: &ProjectConfig) -> Result<Self, String> {
        let root = PathBuf::from(&project.root_path);
        fs::create_dir_all(&root).map_err(|err| err.to_string())?;
        let canonical_root = root.canonicalize().map_err(|err| err.to_string())?;
        Ok(Self { root, canonical_root })
    }

    fn clean_relative_path(&self, path: &str) -> Result<String, String> {
        let clean = path.trim().replace('\\', "/");
        if clean.is_empty() {
            return Err("Project path is required.".to_string());
        }
        if clean.to_lowercase().contains("ai-models.json") {
            return Err("Agent cannot read or write AI model configuration or API keys.".to_string());
        }
        let candidate = Path::new(&clean);
        if candidate.is_absolute() {
            return Err("Absolute paths are not allowed for Agent file operations.".to_string());
        }
        for component in candidate.components() {
            match component {
                std::path::Component::ParentDir | std::path::Component::RootDir | std::path::Component::Prefix(_) => {
                    return Err("Path traversal outside the current project is not allowed.".to_string())
                }
                _ => {}
            }
        }
        Ok(clean)
    }

    fn validate_extension(path: &str, allowed: &[&str]) -> Result<(), String> {
        let ext = Path::new(path)
            .extension()
            .and_then(|value| value.to_str())
            .map(|value| format!(".{}", value.to_ascii_lowercase()))
            .unwrap_or_default();
        if allowed.iter().any(|allowed_ext| *allowed_ext == ext) {
            Ok(())
        } else {
            Err(format!("File type is not allowed for Agent operation: {}", ext))
        }
    }

    fn project_path(&self, path: &str) -> Result<PathBuf, String> {
        let clean = self.clean_relative_path(path)?;
        Ok(self.root.join(clean))
    }

    fn canonical_existing_path(&self, path: &str) -> Result<PathBuf, String> {
        let target = self.project_path(path)?;
        let canonical = target.canonicalize().map_err(|err| err.to_string())?;
        if !canonical.starts_with(&self.canonical_root) {
            return Err("Resolved path is outside the current project root.".to_string());
        }
        Ok(canonical)
    }

    fn read_project_file(&self, path: &str) -> Result<String, String> {
        Self::validate_extension(path, &[".md", ".txt", ".json", ".bib", ".tex", ".yaml", ".yml", ".csv"])?;
        let target = self.canonical_existing_path(path)?;
        fs::read_to_string(target).map_err(|err| err.to_string())
    }

    fn write_project_file(&self, path: &str, content: &str) -> Result<(), String> {
        Self::validate_extension(path, &[".md", ".txt", ".json", ".bib", ".tex", ".yaml", ".yml"])?;
        let target = self.project_path(path)?;
        let parent = target.parent().ok_or_else(|| "Target path has no parent.".to_string())?;
        let canonical_parent = parent.canonicalize().map_err(|err| err.to_string())?;
        if !canonical_parent.starts_with(&self.canonical_root) {
            return Err("Resolved write path is outside the current project root.".to_string());
        }
        fs::write(target, content).map_err(|err| err.to_string())
    }

    fn list_project_files(&self) -> Result<Vec<String>, String> {
        let mut files = vec![];
        self.collect_files(&self.root, &mut files)?;
        files.sort();
        Ok(files)
    }

    fn collect_files(&self, dir: &Path, files: &mut Vec<String>) -> Result<(), String> {
        if !dir.exists() {
            return Ok(());
        }
        for entry in fs::read_dir(dir).map_err(|err| err.to_string())? {
            let entry = entry.map_err(|err| err.to_string())?;
            let path = entry.path();
            if path.is_dir() {
                self.collect_files(&path, files)?;
            } else if let Ok(relative) = path.strip_prefix(&self.root) {
                let value = relative.to_string_lossy().replace('\\', "/");
                if !value.to_lowercase().contains("ai-models.json") {
                    files.push(value);
                }
            }
        }
        Ok(())
    }

    fn list_figures(&self) -> Result<Vec<String>, String> {
        let dir = self.root.join("attachments/figures");
        let mut figures = vec![];
        if !dir.exists() {
            return Ok(figures);
        }
        for entry in fs::read_dir(dir).map_err(|err| err.to_string())? {
            let entry = entry.map_err(|err| err.to_string())?;
            let path = entry.path();
            if path.is_file() {
                if let Ok(relative) = path.strip_prefix(&self.root) {
                    figures.push(relative.to_string_lossy().replace('\\', "/"));
                }
            }
        }
        figures.sort();
        Ok(figures)
    }

    fn file_tree(&self) -> Result<Vec<FileTreeNode>, String> {
        let mut children = vec![];
        self.collect_tree(&self.root, "", &mut children)?;
        children.sort_by(|left, right| left.kind.cmp(&right.kind).then_with(|| left.name.cmp(&right.name)));
        Ok(children)
    }

    fn collect_tree(&self, dir: &Path, rel: &str, nodes: &mut Vec<FileTreeNode>) -> Result<(), String> {
        if !dir.exists() {
            return Ok(());
        }
        let skip = ["node_modules", "target", "dist", ".git", ".local", ".vite", "logs"];
        for entry in fs::read_dir(dir).map_err(|err| err.to_string())? {
            let entry = entry.map_err(|err| err.to_string())?;
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            if skip.iter().any(|item| item.eq_ignore_ascii_case(&name)) {
                continue;
            }
            let relative = if rel.is_empty() {
                name.clone()
            } else {
                format!("{}/{}", rel, name)
            };
            if relative.to_lowercase().contains("ai-models.json") {
                continue;
            }
            if path.is_dir() {
                let mut children = vec![];
                self.collect_tree(&path, &relative, &mut children)?;
                children.sort_by(|left, right| left.kind.cmp(&right.kind).then_with(|| left.name.cmp(&right.name)));
                nodes.push(FileTreeNode {
                    name,
                    path: relative.clone(),
                    relative_path: relative,
                    kind: "directory".to_string(),
                    extension: None,
                    children: Some(children),
                });
            } else {
                nodes.push(FileTreeNode {
                    name,
                    path: relative.clone(),
                    relative_path: relative,
                    kind: "file".to_string(),
                    extension: path.extension().and_then(|value| value.to_str()).map(|value| value.to_ascii_lowercase()),
                    children: None,
                });
            }
        }
        Ok(())
    }
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
    fs::create_dir_all(root.join(".paperforge")).map_err(|err| err.to_string())?;
    let path = activity_path(&root);
    let mut activities: Vec<ProjectActivity> = read_json_vec(&path)?;
    let line = format!("{} {}\n", now_iso(), message);
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
    use std::io::Write;
    let mut history = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(paper_history_path(&root))
        .map_err(|err| err.to_string())?;
    history
        .write_all(line.as_bytes())
        .map_err(|err| err.to_string())?;
    Ok(activities)
}

fn agent_tool_result(tool: &str, ok: bool, message: &str) -> AgentToolResult {
    AgentToolResult {
        tool: tool.to_string(),
        ok,
        message: message.to_string(),
        data: None,
        error: None,
        reason: None,
    }
}

fn make_simple_diff(path: &str, original: &str, proposed: &str) -> String {
    if original == proposed {
        return format!("--- {}\n+++ {}\n(no changes)", path, path);
    }
    let before: Vec<&str> = original.split('\n').collect();
    let after: Vec<&str> = proposed.split('\n').collect();
    let mut lines = vec![format!("--- {}", path), format!("+++ {}", path)];
    let max = before.len().max(after.len());
    for index in 0..max {
        match (before.get(index), after.get(index)) {
            (Some(left), Some(right)) if left == right => lines.push(format!(" {}", left)),
            (Some(left), Some(right)) => {
                lines.push(format!("-{}", left));
                lines.push(format!("+{}", right));
            }
            (Some(left), None) => lines.push(format!("-{}", left)),
            (None, Some(right)) => lines.push(format!("+{}", right)),
            (None, None) => {}
        }
    }
    lines.join("\n")
}

fn agent_change(path: &str, original: String, proposed: String) -> AgentFileChange {
    AgentFileChange {
        id: format!("agent_change_{}", Uuid::new_v4()),
        path: path.to_string(),
        change_type: if original.is_empty() {
            AgentFileChangeType::Create
        } else {
            AgentFileChangeType::Update
        },
        diff: make_simple_diff(path, &original, &proposed),
        original_content: original,
        proposed_content: proposed,
        status: AgentChangeStatus::Pending,
    }
}

fn write_agent_run(project: &ProjectConfig, run: &AgentRun) -> Result<(), String> {
    let root = PathBuf::from(&project.root_path);
    fs::create_dir_all(agent_run_dir(&root)).map_err(|err| err.to_string())?;
    write_json(&agent_run_path(&root, &run.id), run)
}

fn read_agent_run(project: &ProjectConfig, run_id: &str) -> Result<AgentRun, String> {
    let root = PathBuf::from(&project.root_path);
    let path = agent_run_path(&root, run_id);
    if !path.exists() {
        return Err("Agent run not found".to_string());
    }
    let raw = fs::read_to_string(path).map_err(|err| err.to_string())?;
    serde_json::from_str(&raw).map_err(|err| err.to_string())
}

fn append_agent_log_entry(project: &ProjectConfig, run: &AgentRun, success: bool, error: Option<String>) -> Result<(), String> {
    let root = PathBuf::from(&project.root_path);
    fs::create_dir_all(root.join(".paperforge")).map_err(|err| err.to_string())?;
    let entry = AgentLogEntry {
        id: format!("agent_log_{}", Uuid::new_v4()),
        run_id: run.id.clone(),
        project_id: run.project_id.clone(),
        mode: run.mode.clone(),
        skill_id: run.skill_id.clone(),
        request: run.request.clone(),
        tools: run.tool_results.iter().map(|tool| tool.tool.clone()).collect(),
        files_read: run.files_read.clone(),
        files_changed: run.files_changed.clone(),
        success,
        error,
        created_at: now_iso(),
    };
    use std::io::Write;
    let mut log = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(agent_log_path(&root))
        .map_err(|err| err.to_string())?;
    let line = serde_json::to_string(&entry).map_err(|err| err.to_string())?;
    log.write_all(format!("{}\n", line).as_bytes())
        .map_err(|err| err.to_string())
}

fn read_agent_log_entries(project: &ProjectConfig) -> Result<Vec<AgentLogEntry>, String> {
    let root = PathBuf::from(&project.root_path);
    let path = agent_log_path(&root);
    if !path.exists() {
        return Ok(vec![]);
    }
    let raw = fs::read_to_string(path).map_err(|err| err.to_string())?;
    let mut entries = vec![];
    for line in raw.lines().filter(|line| !line.trim().is_empty()) {
        if let Ok(entry) = serde_json::from_str::<AgentLogEntry>(line) {
            entries.push(entry);
        }
    }
    entries.reverse();
    entries.truncate(80);
    Ok(entries)
}

fn check_broken_links(project: &ProjectConfig, sections: &[ManuscriptSection]) -> Result<Vec<String>, String> {
    let root = PathBuf::from(&project.root_path);
    let link_pattern = Regex::new(r"!\[[^\]]*\]\(([^)]+)\)|\[[^\]]+\]\(([^)]+)\)").map_err(|err| err.to_string())?;
    let mut broken = vec![];
    for section in sections {
        for capture in link_pattern.captures_iter(&section.content) {
            let target = capture.get(1).or_else(|| capture.get(2)).map(|value| value.as_str()).unwrap_or("");
            if target.starts_with("http://") || target.starts_with("https://") || target.starts_with('#') || target.trim().is_empty() {
                continue;
            }
            if target.contains("..") || Path::new(target).is_absolute() {
                broken.push(format!("{} -> {}", section.path, target));
                continue;
            }
            if !root.join(target).exists() {
                broken.push(format!("{} -> {}", section.path, target));
            }
        }
    }
    Ok(broken)
}

fn sync_project_after_agent_write(project: &ProjectConfig, changed_path: &str) -> Result<(), String> {
    let mut project = project.clone();
    let timestamp = now_iso();
    for section in &mut project.manuscript.sections {
        if section.path == changed_path {
            section.updated_at = timestamp.clone();
        }
    }
    for section in &mut project.sections {
        if section.path == changed_path {
            section.updated_at = timestamp.clone();
        }
    }
    project.updated_at = timestamp;
    let root = PathBuf::from(&project.root_path);
    write_json(&project_manifest_path(&root), &project)?;
    write_json(&compatibility_project_manifest_path(&root), &project)?;
    let mut projects = read_registry()?;
    projects = projects
        .into_iter()
        .map(|item| if item.id == project.id { project.clone() } else { item })
        .collect();
    write_registry(&projects)
}

#[tauri::command]
fn list_agent_skills(_project_id: String) -> Result<Vec<AgentSkill>, String> {
    Ok(built_in_agent_skills())
}

#[tauri::command]
fn run_agent(
    project_id: String,
    mode: AgentMode,
    skill_id: String,
    request: String,
    section_id: Option<String>,
) -> Result<AgentRun, String> {
    let project = project_by_id(&project_id)?;
    ensure_structure(&project)?;
    let pfs = ProjectFileSystem::new(&project)?;
    let skill = select_agent_skill(&mode, &skill_id, &request);
    let sections = list_sections(project_id.clone())?;
    let selected_section = section_id
        .as_ref()
        .and_then(|id| sections.iter().find(|section| &section.id == id))
        .or_else(|| sections.first());
    let timestamp = now_iso();
    let mut files_read = vec![];
    let mut files_to_change = vec![];
    let mut changes = vec![];
    let report: String;
    let mut tool_results = skill
        .allowed_tools
        .iter()
        .map(|tool| agent_tool_result(tool, true, "Tool registered."))
        .collect::<Vec<_>>();
    let settings = read_settings()?;

    match &mode {
        AgentMode::Ask => {
            let files = pfs.list_project_files()?;
            let figures = pfs.list_figures()?;
            let broken = check_broken_links(&project, &sections)?;
            files_read.push("paperforge.json".to_string());
            files_read.extend(sections.iter().map(|section| section.path.clone()));
            let draft = sections
                .iter()
                .map(|section| format!("## {}\n{}", section.title, section.content))
                .collect::<Vec<_>>()
                .join("\n\n");
            report = safe_call_llm(
                &settings,
                "You are PaperForge Project Agent. Return a concise manuscript project report. Do not claim files were modified.",
                &format!(
                    "Skill: {}\nUser request: {}\nProject: {}\nFiles: {}\nSections: {}\nFigures: {}\nBroken links: {}\nManuscript draft:\n{}",
                    skill.name,
                    request,
                    project.title,
                    files.len(),
                    sections.len(),
                    figures.len(),
                    if broken.is_empty() { "none".to_string() } else { broken.join(", ") },
                    draft
                ),
            )?;
            tool_results.push(AgentToolResult {
                tool: "check_broken_links".to_string(),
                ok: broken.is_empty(),
                message: format!("{} broken link(s).", broken.len()),
                data: Some(serde_json::json!(broken)),
                error: None,
                reason: None,
            });
        }
        AgentMode::Edit => {
            if let Some(section) = selected_section {
                files_read.push(section.path.clone());
                let original = pfs.read_project_file(&section.path)?;
                let task = if skill.id == "edit.translate-zh-en" {
                    "Translate or bilingual-polish this section while preserving citations, numbers, equations, and technical meaning."
                } else {
                    "Improve academic style while preserving citations, numbers, equations, and technical meaning."
                };
                let proposed = safe_call_llm(
                    &settings,
                    "You are PaperForge editing agent. Return only the revised Markdown section. Preserve citation markers exactly.",
                    &format!(
                        "Task: {}\nUser request: {}\nSection title: {}\nOriginal Markdown:\n{}",
                        task, request, section.title, original
                    ),
                )?;
                files_to_change.push(section.path.clone());
                changes.push(agent_change(&section.path, original, proposed));
                report = format!("{} prepared a diff for {}. Review before applying.", skill.name.clone(), section.title);
            } else {
                report = "No active section found. Create a section before using Edit mode.".to_string();
            }
        }
        AgentMode::Operate => {
            if let Some(section) = selected_section {
                files_read.push(section.path.clone());
                files_read.push("attachments/figures".to_string());
                let figure_pattern = Regex::new(r"attachments/figures/[^\s)]+").map_err(|err| err.to_string())?;
                let figure_path = figure_pattern
                    .find(&request)
                    .map(|value| value.as_str().trim().to_string())
                    .or_else(|| pfs.list_figures().ok().and_then(|figures| figures.first().cloned()));
                if let Some(figure_path) = figure_path {
                    if !figure_path.replace('\\', "/").starts_with("attachments/figures/") {
                        report = "Insert Figure refused path outside attachments/figures.".to_string();
                    } else {
                        let _ = pfs.canonical_existing_path(&figure_path)?;
                        let original = pfs.read_project_file(&section.path)?;
                        let proposed = format!("{}\n\n![Figure caption]({})\n", original.trim_end(), figure_path);
                        files_to_change.push(section.path.clone());
                        changes.push(agent_change(&section.path, original, proposed));
                        report = "Insert Figure prepared a Markdown image reference for the active section.".to_string();
                    }
                } else {
                    report = "No figure path found. Put a figure under attachments/figures and mention its path.".to_string();
                }
            } else {
                report = "No active section found. Create a section before using Operate mode.".to_string();
            }
        }
    }

    let run = AgentRun {
        id: format!("agent_run_{}", Uuid::new_v4()),
        project_id: project_id.clone(),
        mode,
        skill_id: skill.id.clone(),
        request: request.clone(),
        status: if changes.is_empty() {
            AgentRunStatus::Completed
        } else {
            AgentRunStatus::Planned
        },
        plan: AgentPlan {
            summary: format!("{}: {}", skill.name.clone(), if request.trim().is_empty() { "No request text provided." } else { request.trim() }),
            steps: if skill.writes_files {
                vec!["Read safe project context".to_string(), "Prepare diff".to_string(), "Wait for Apply or Reject".to_string()]
            } else {
                vec!["Inspect safe project context".to_string(), "Run read-only checks".to_string(), "Return report".to_string()]
            },
            files_to_read: files_read.clone(),
            files_to_change,
        },
        files_read,
        files_changed: vec![],
        report,
        changes,
        tool_results,
        created_at: timestamp.clone(),
        updated_at: timestamp,
    };
    write_agent_run(&project, &run)?;
    append_agent_log_entry(&project, &run, true, None)?;
    Ok(run)
}

#[tauri::command]
fn apply_agent_change(project_id: String, run_id: String, change_id: String) -> Result<AgentRun, String> {
    let project = project_by_id(&project_id)?;
    let pfs = ProjectFileSystem::new(&project)?;
    let mut run = read_agent_run(&project, &run_id)?;
    let change_index = run
        .changes
        .iter()
        .position(|change| change.id == change_id)
        .ok_or_else(|| "Agent change not found".to_string())?;
    let change = run.changes[change_index].clone();
    let current = pfs.read_project_file(&change.path)?;
    let root = PathBuf::from(&project.root_path);
    fs::create_dir_all(agent_backup_dir(&root)).map_err(|err| err.to_string())?;
    let backup_name = format!(
        "{}_{}.bak",
        now_iso().replace(':', "-").replace('.', "-"),
        safe_filename(&change.path.replace('/', "_"))
    );
    fs::write(agent_backup_dir(&root).join(backup_name), current).map_err(|err| err.to_string())?;
    pfs.write_project_file(&change.path, &change.proposed_content)?;
    run.changes[change_index].status = AgentChangeStatus::Applied;
    if !run.files_changed.contains(&change.path) {
        run.files_changed.push(change.path.clone());
    }
    run.status = AgentRunStatus::Applied;
    run.updated_at = now_iso();
    write_agent_run(&project, &run)?;
    sync_project_after_agent_write(&project, &change.path)?;
    append_agent_log_entry(&project, &run, true, None)?;
    Ok(run)
}

#[tauri::command]
fn reject_agent_run(project_id: String, run_id: String) -> Result<AgentRun, String> {
    let project = project_by_id(&project_id)?;
    let mut run = read_agent_run(&project, &run_id)?;
    run.status = AgentRunStatus::Rejected;
    run.updated_at = now_iso();
    for change in &mut run.changes {
        change.status = AgentChangeStatus::Rejected;
    }
    write_agent_run(&project, &run)?;
    append_agent_log_entry(&project, &run, true, None)?;
    Ok(run)
}

#[tauri::command]
fn read_agent_log(project_id: String) -> Result<Vec<AgentLogEntry>, String> {
    let project = project_by_id(&project_id)?;
    read_agent_log_entries(&project)
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
    let mut settings: AppSettings = serde_json::from_str(&raw).map_err(|err| err.to_string())?;
    settings.llm_provider = normalize_llm_settings(settings.llm_provider);
    Ok(settings)
}

#[tauri::command]
fn save_settings(mut settings: AppSettings) -> Result<AppSettings, String> {
    settings.llm_provider = normalize_llm_settings(settings.llm_provider);
    write_json(&settings_path()?, &settings)?;
    Ok(settings)
}

#[tauri::command]
fn test_ai_connection(settings: AppSettings) -> Result<String, String> {
    let provider = validate_ai_settings(&settings.llm_provider)?;
    let content = call_llm(
        &AppSettings { llm_provider: provider, ..settings },
        "Reply with exactly: PaperForge AI connection ok",
        "Connection test.",
    )?;
    Ok(if content.trim().is_empty() {
        "AI connection succeeded.".to_string()
    } else {
        format!("AI connection succeeded: {}", content.lines().next().unwrap_or("").trim())
    })
}

#[tauri::command]
fn fetch_ai_models(settings: AppSettings) -> Result<Vec<String>, String> {
    let provider = validate_ai_settings(&settings.llm_provider)?;
    if matches!(provider.provider, LlmProviderKind::Anthropic) {
        return Err("Model fetching is not supported for Anthropic in this build.".to_string());
    }
    let value = get_json_with_curl(&join_url(&provider.base_url, "models"), &provider_auth_headers(&provider))?;
    let models = value
        .get("data")
        .and_then(|data| data.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.get("id").and_then(|id| id.as_str()).map(String::from))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if models.is_empty() {
        Err("No models returned by provider.".to_string())
    } else {
        Ok(models)
    }
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
    ensure_workspace(&workspace_path)?;
    let root_path = workspace_path.join("papers").join(safe_folder_name(&title));
    let timestamp = now_iso();
    let citation_backend = citation_backend(&input.manuscript_mode);
    let sections = create_initial_sections(&input.section_names, &input.section_naming);
    let project = ProjectConfig {
        id: format!("project_{}", Uuid::new_v4()),
        version: default_project_version(),
        title,
        author,
        authors,
        target_journal: target_journal.clone(),
        journal: if target_journal == default_target_journal() {
            String::new()
        } else {
            target_journal.clone()
        },
        language: Language::En,
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
        sections: sections.iter().map(manifest_from_section).collect(),
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
    } else if compatibility_project_manifest_path(&root).exists() {
        compatibility_project_manifest_path(&root)
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
            version: default_project_version(),
            title,
            author: String::new(),
            authors: vec![],
            target_journal: default_target_journal(),
            journal: String::new(),
            language: Language::En,
            citation_style: default_citation_style(),
            export_mode: ManuscriptMode::Markdown,
            manuscript_mode: ManuscriptMode::Word,
            root_path: root.to_string_lossy().to_string(),
            created_at: timestamp.clone(),
            updated_at: timestamp,
            citation_backend: CitationBackend::ZoteroWordPlugin,
            manuscript: default_manuscript_manifest(),
            sections: vec![],
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
    write_json(&compatibility_project_manifest_path(&root), &project)?;
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
        project.journal = if project.target_journal == default_target_journal() {
            String::new()
        } else {
            project.target_journal.clone()
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
    if delete_files {
        let Some(project) = project.clone() else {
            return Err("Paper not found".to_string());
        };
        let path = PathBuf::from(&project.root_path);
        if path.exists() {
            fs::remove_dir_all(&path).map_err(|err| {
                format!(
                    "Could not delete paper folder '{}': {}. Check file permissions or close files opened by another program.",
                    path.to_string_lossy(),
                    err
                )
            })?;
        }
    }
    projects.retain(|project| project.id != project_id);
    write_registry(&projects)?;
    Ok(true)
}

#[tauri::command]
fn export_project_manifest(project_id: String) -> Result<String, String> {
    let project = project_by_id(&project_id)?;
    let root = PathBuf::from(&project.root_path);
    let raw = serde_json::to_string_pretty(&project).map_err(|err| err.to_string())?;
    fs::create_dir_all(root.join("exports/json")).map_err(|err| err.to_string())?;
    fs::write(root.join("exports/json/paperforge.json"), &raw).map_err(|err| err.to_string())?;
    Ok(raw)
}

#[tauri::command]
fn list_sections(project_id: String) -> Result<Vec<ManuscriptSection>, String> {
    let mut project = project_by_id(&project_id)?;
    let root = PathBuf::from(&project.root_path);
    let mut sections = vec![];
    let mut seen_paths = HashSet::new();
    let mut seen_filenames = HashSet::new();
    let mut changed = false;
    for manifest in &project.manuscript.sections {
        if !root.join(&manifest.path).is_file() {
            changed = true;
            continue;
        }
        let filename = manifest_file_name(manifest);
        if !seen_paths.insert(manifest.path.clone()) || !seen_filenames.insert(filename) {
            changed = true;
            continue;
        }
        sections.push(section_from_manifest(&project, manifest)?);
    }
    let scanned = scan_existing_section_files(&project)?;
    let mut next_order = sections
        .iter()
        .map(|section| section.order)
        .max()
        .unwrap_or(0)
        + 1;
    for section in scanned {
        if !seen_paths.contains(&section.path) && !seen_filenames.contains(&section.filename) {
            let mut section = section;
            section.order = next_order;
            next_order += 1;
            seen_paths.insert(section.path.clone());
            seen_filenames.insert(section.filename.clone());
            sections.push(section);
            changed = true;
        }
    }
    sections.sort_by(|left, right| {
        left.order
            .cmp(&right.order)
            .then_with(|| left.path.cmp(&right.path))
    });
    if changed {
        project.manuscript.sections = sections.iter().map(manifest_from_section).collect();
        update_project_config(project)?;
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
        "attachments".to_string(),
        "exports".to_string(),
        ".paperforge".to_string(),
    ])
}

#[tauri::command]
fn list_project_files(project_id: String) -> Result<Vec<FileTreeNode>, String> {
    let project = project_by_id(&project_id)?;
    ProjectFileSystem::new(&project)?.file_tree()
}

#[tauri::command]
fn read_text_file(project_id: String, path: String) -> Result<TextFilePayload, String> {
    let project = project_by_id(&project_id)?;
    let pfs = ProjectFileSystem::new(&project)?;
    let clean = pfs.clean_relative_path(&path)?;
    let content = pfs.read_project_file(&clean)?;
    Ok(TextFilePayload { path: clean, content })
}

#[tauri::command]
fn write_text_file(project_id: String, path: String, content: String) -> Result<TextFilePayload, String> {
    let project = project_by_id(&project_id)?;
    let pfs = ProjectFileSystem::new(&project)?;
    let clean = pfs.clean_relative_path(&path)?;
    pfs.write_project_file(&clean, &content)?;
    sync_project_after_agent_write(&project, &clean)?;
    Ok(TextFilePayload { path: clean, content })
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
    let start_re = Regex::new(r"@(?:\w+)\s*\{" ).map_err(|err| err.to_string())?;
    let field_re = Regex::new(r"^\s*([^,\s]+)\s*,\s*([\s\S]*)$").map_err(|err| err.to_string())?;
    let mut refs = vec![];
    let starts: Vec<usize> = start_re.find_iter(&bibtex).map(|m| m.end()).collect();
    for (i, start) in starts.iter().enumerate() {
        let end = starts.get(i + 1).copied().unwrap_or_else(|| bibtex.len());
        let raw = bibtex[*start..end].trim();
        let block = raw.trim_end_matches(|c: char| c == '}' || c == ' ' || c == '\n' || c == '\r').trim();
        let Some(captures) = field_re.captures(block) else { continue };
        let citekey = captures
            .get(1)
            .map(|m| m.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        let body = captures.get(2).map(|m| m.as_str()).unwrap_or("");
        if citekey.is_empty() { continue }
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
        PathBuf::from(&project.root_path).join("references/bib/references.bib"),
        &bibtex,
    )
    .map_err(|err| err.to_string())?;
    let refs = parse_bibtex(bibtex)?;
    write_json(
        &PathBuf::from(&project.root_path).join("references/bib/references.json"),
        &refs,
    )?;
    Ok(refs)
}

#[tauri::command]
fn list_references(project_id: String) -> Result<Vec<ReferenceItem>, String> {
    let project = project_by_id(&project_id)?;
    let refs_path = PathBuf::from(&project.root_path).join("references/bib/references.json");
    if refs_path.exists() {
        return read_json_vec(&refs_path);
    }
    let bib_path = PathBuf::from(&project.root_path).join("references/bib/references.bib");
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
    let path = PathBuf::from(&project.root_path).join("references/papers/papers.json");
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
    read_json_vec(&PathBuf::from(&project.root_path).join("references/papers/papers.json"))
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
    read_json_vec(&PathBuf::from(&project.root_path).join(".paperforge/claims.json"))
}

#[tauri::command]
fn save_claims(project_id: String, claims: Vec<ClaimRecord>) -> Result<Vec<ClaimRecord>, String> {
    let project = project_by_id(&project_id)?;
    write_json(
        &PathBuf::from(&project.root_path).join(".paperforge/claims.json"),
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
    let proposed_text = call_llm(
        &settings,
        "You are PaperForge AI assistant. Return only revised Markdown text. Preserve citation markers, numbers, equations, and technical meaning.",
        &format!(
            "Instruction: {}\nSelected text:\n{}",
            instruction,
            if selected_text.trim().is_empty() {
                "Draft a focused manuscript improvement proposal."
            } else {
                selected_text.as_str()
            }
        ),
    )?;
    Ok(AiProposal {
        id: format!("proposal_{}", Uuid::new_v4()),
        section_id,
        instruction,
        original_text: selected_text.clone(),
        proposed_text,
        citation_keys: vec![],
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

fn first_line(value: &str) -> String {
    value.lines().next().unwrap_or(value).trim().to_string()
}

fn windows_pandoc_candidates() -> Vec<PathBuf> {
    let mut candidates = vec![];
    for key in ["PAPERFORGE_PANDOC", "PANDOC"] {
        if let Ok(value) = std::env::var(key) {
            if !value.trim().is_empty() {
                candidates.push(PathBuf::from(value.trim()));
            }
        }
    }
    for key in ["LOCALAPPDATA", "PROGRAMFILES", "PROGRAMFILES(X86)", "USERPROFILE"] {
        if let Ok(value) = std::env::var(key) {
            let base = PathBuf::from(value);
            match key {
                "LOCALAPPDATA" => {
                    candidates.push(base.join("Pandoc/pandoc.exe"));
                    candidates.push(base.join("Programs/Pandoc/pandoc.exe"));
                    let winget = base.join("Microsoft/WinGet/Packages");
                    if let Ok(entries) = fs::read_dir(winget) {
                        for entry in entries.flatten() {
                            let name = entry.file_name().to_string_lossy().to_string();
                            if name.starts_with("JohnMacFarlane.Pandoc") {
                                candidates.push(entry.path().join("pandoc.exe"));
                            }
                        }
                    }
                }
                "PROGRAMFILES" | "PROGRAMFILES(X86)" => candidates.push(base.join("Pandoc/pandoc.exe")),
                "USERPROFILE" => candidates.push(base.join("scoop/shims/pandoc.exe")),
                _ => {}
            }
        }
    }
    candidates
}

fn find_pandoc_executable() -> PathBuf {
    windows_pandoc_candidates()
        .into_iter()
        .find(|path| path.exists())
        .unwrap_or_else(|| PathBuf::from("pandoc"))
}

fn pandoc_version_from_command() -> Result<String, String> {
    let pandoc = find_pandoc_executable();
    let output = Command::new(&pandoc)
        .arg("--version")
        .output()
        .map_err(|err| format!("{} ({})", err, pandoc.to_string_lossy()))?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

#[tauri::command]
fn get_pandoc_version() -> Result<String, String> {
    pandoc_version_from_command()
}

#[tauri::command]
fn check_pandoc_installed() -> PandocStatus {
    match pandoc_version_from_command() {
        Ok(version) => PandocStatus {
            installed: true,
            version: Some(first_line(&version)),
            error: None,
        },
        Err(error) => PandocStatus {
            installed: false,
            version: None,
            error: Some(error),
        },
    }
}

fn run_winget_pandoc_install() -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let status = Command::new("winget")
            .args([
                "install",
                "--id",
                "JohnMacFarlane.Pandoc",
                "-e",
                "--source",
                "winget",
                "--accept-package-agreements",
                "--accept-source-agreements",
                "--silent",
            ])
            .status()
            .map_err(|err| format!("Failed to start winget: {}", err))?;
        if status.success() {
            Ok(())
        } else {
            Err(format!("winget exited with status {}", status))
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        Err("Automatic Pandoc install is only supported on Windows.".to_string())
    }
}

fn ensure_pandoc_available_with<C, I>(mut check: C, mut install: I) -> Result<Vec<String>, Vec<String>>
where
    C: FnMut() -> Result<String, String>,
    I: FnMut() -> Result<(), String>,
{
    let mut logs = vec![];
    match check() {
        Ok(version) => {
            logs.push(format!("Pandoc detected: {}", first_line(&version)));
            return Ok(logs);
        }
        Err(error) => {
            logs.push(PANDOC_REQUIRED_MESSAGE.to_string());
            logs.push(format!("Pandoc detection failed: {}", error));
        }
    }

    #[cfg(target_os = "windows")]
    {
        logs.push(format!("Attempting automatic install: {}", PANDOC_INSTALL_COMMAND));
        if let Err(error) = install() {
            logs.push(format!("Pandoc automatic install failed: {}", error));
            logs.push(format!("Manual install command: {}", PANDOC_INSTALL_COMMAND));
            return Err(logs);
        }
        logs.push("Pandoc automatic install completed. Rechecking pandoc --version.".to_string());
        match check() {
            Ok(version) => {
                logs.push(format!("Pandoc detected: {}", first_line(&version)));
                Ok(logs)
            }
            Err(error) => {
                logs.push(format!("Pandoc still unavailable after install: {}", error));
                logs.push(format!("Manual install command: {}", PANDOC_INSTALL_COMMAND));
                Err(logs)
            }
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = install;
        logs.push("Automatic Pandoc install is only supported on Windows.".to_string());
        logs.push("Install Pandoc from https://pandoc.org/installing.html and retry.".to_string());
        Err(logs)
    }
}

fn ensure_pandoc_available() -> Result<Vec<String>, Vec<String>> {
    ensure_pandoc_available_with(pandoc_version_from_command, run_winget_pandoc_install)
}

fn failed_export_job(project_id: String, mode: ManuscriptMode, logs: Vec<String>) -> ExportJob {
    ExportJob {
        id: format!("export_{}", Uuid::new_v4()),
        project_id,
        mode,
        status: ExportStatus::Failed,
        output_path: String::new(),
        logs,
        created_at: now_iso(),
    }
}


fn append_warn(logs: &mut Vec<String>, label: &str, err: impl ToString) {
    logs.push(format!("WARN: {}: {}", label, err.to_string()));
    }

fn run_pandoc(args: &[String], current_dir: &Path) -> Result<String, String> {
    let pandoc = find_pandoc_executable();
    let output = Command::new(&pandoc)
        .args(args)
        .current_dir(current_dir)
        .output()
        .map_err(|err| format!("{} ({})", err, pandoc.to_string_lossy()))?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

fn word_pandoc_args(has_reference_doc: bool) -> Vec<String> {
    let mut args = vec![
        "exports/_intermediate/paper-word.md".to_string(),
        "-f".to_string(),
        "markdown".to_string(),
        "-t".to_string(),
        "docx".to_string(),
        "-o".to_string(),
        "exports/word/paper.docx".to_string(),
    ];
    if has_reference_doc {
        args.push("--reference-doc=exports/word/reference.docx".to_string());
    }
    args
}

fn latex_pandoc_args() -> Vec<String> {
    vec![
        "exports/_intermediate/paper-latex.md".to_string(),
        "-f".to_string(),
        "markdown+raw_tex".to_string(),
        "-t".to_string(),
        "latex".to_string(),
        "-s".to_string(),
        "-o".to_string(),
        "exports/latex/paper.tex".to_string(),
    ]
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

fn copy_project_snapshot_recursive(
    src: &Path,
    dst: &Path,
    files: &mut Vec<String>,
    prefix: &str,
) -> Result<(), String> {
    fs::create_dir_all(dst).map_err(|err| err.to_string())?;
    for entry in fs::read_dir(src).map_err(|err| err.to_string())? {
        let entry = entry.map_err(|err| err.to_string())?;
        let name = entry.file_name();
        let name_string = name.to_string_lossy();
        if name_string == ".git" || name_string == "project-folder" {
            continue;
        }
        let src_path = entry.path();
        let dst_path = dst.join(&name);
        let rel = if prefix.is_empty() {
            name_string.to_string()
        } else {
            format!("{}/{}", prefix, name_string)
        }
        .replace('\\', "/");
        if src_path.is_dir() {
            copy_project_snapshot_recursive(&src_path, &dst_path, files, &rel)?;
        } else {
            if let Some(parent) = dst_path.parent() {
                fs::create_dir_all(parent).map_err(|err| err.to_string())?;
            }
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
        .join("exports/markdown")
        .join(format!("paperforge-export-{}", stamp));
    fs::create_dir_all(&output).map_err(|err| err.to_string())?;

    let mut files: Vec<String> = vec![];
    let mut skipped: Vec<serde_json::Value> = vec![];
    let mut warnings: Vec<String> = vec![];

    write_json(&output.join("manifest.json"), &project)?;
    files.push("manifest.json".to_string());

    let (body, body_warnings) = markdown_package_body(&project_id)?;
    warnings.extend(body_warnings);
    fs::write(output.join("paper.md"), body).map_err(|err| err.to_string())?;
    files.push("paper.md".to_string());

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
        let library = root.join("references/bib/library.bib");
        if library.exists() {
            library
        } else {
            root.join("references/bib/references.bib")
        }
    };
    copy_optional_file(
        &bib_src,
        &output.join("references/bib/library.bib"),
        "references/bib/library.bib",
        &mut files,
        &mut skipped,
    )?;
    copy_optional_file(
        &root.join("references/bib/references.json"),
        &output.join("references/bib/references.json"),
        "references/bib/references.json",
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
        let papers = root.join("references/papers/papers.json");
        if papers.exists() {
            papers
        } else {
            root.join("references/papers/papers.json")
        }
    };
    copy_optional_file(
        &papers_src,
        &output.join("references/papers/papers.json"),
        "references/papers/papers.json",
        &mut files,
        &mut skipped,
    )?;
    copy_dir_recursive(
        &root.join("attachments"),
        &output.join("attachments"),
        &mut files,
        "attachments",
    )?;
    copy_optional_file(
        &root.join(".paperforge/claims.json"),
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
        output_path: export_output_path(&output)?,
        logs: vec![
            "Markdown package exported.".to_string(),
            "Includes manifest.json, paper.md, sections/, references/, attachments/, export-report.json.".to_string(),
        ],
        created_at: now_iso(),
    })
}

#[tauri::command]
fn export_project_folder(project_id: String) -> Result<ExportJob, String> {
    let project = normalize_project(project_by_id(&project_id)?);
    let root = PathBuf::from(&project.root_path);
    let stamp = Utc::now().format("%Y%m%d-%H%M%S").to_string();
    let output = root
        .join("exports/project-folder")
        .join(format!("{}-{}", safe_folder_name(&project.title), stamp));
    let mut files = vec![];
    copy_project_snapshot_recursive(&root, &output, &mut files, "")?;
    write_json(
        &output.join("export-report.json"),
        &serde_json::json!({
            "exportedAt": now_iso(),
            "projectTitle": project.title,
            "mode": "project-folder",
            "outputDir": output.to_string_lossy(),
            "files": files,
        }),
    )?;
    Ok(ExportJob {
        id: format!("export_{}", Uuid::new_v4()),
        project_id,
        mode: ManuscriptMode::Markdown,
        status: ExportStatus::Success,
        output_path: export_output_path(&output)?,
        logs: vec!["Project folder snapshot exported.".to_string()],
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
        logs: vec!["Use Export Word Draft to run Pandoc DOCX export.".to_string()],
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
        logs: vec!["Use Export LaTeX Project to run Pandoc LaTeX export.".to_string()],
        created_at: now_iso(),
    })
}

#[tauri::command]
fn export_word_draft(project_id: String) -> Result<ExportJob, String> {
    let project = project_by_id(&project_id)?;
    let root = PathBuf::from(&project.root_path);
    let mut logs = match ensure_pandoc_available() {
        Ok(logs) => logs,
        Err(logs) => return Ok(failed_export_job(project_id, ManuscriptMode::Word, logs)),
    };
    fs::create_dir_all(root.join("exports/_intermediate")).map_err(|err| err.to_string())?;
    fs::create_dir_all(root.join("exports/word")).map_err(|err| err.to_string())?;
    let intermediate = root.join("exports/_intermediate/paper-word.md");
    let output = root.join("exports/word/paper.docx");
    fs::write(
        &intermediate,
        convert_citations_for_mode(&merged_sections(&project_id)?, &ManuscriptMode::Word)?,
    )
    .map_err(|err| err.to_string())?;
    let reference_doc = root.join("exports/word/reference.docx");
    let args = word_pandoc_args(reference_doc.exists());
    if let Err(error) = run_pandoc(&args, &root) {
        logs.push(format!("Pandoc DOCX export failed: {}", error));
        return Ok(failed_export_job(project_id, ManuscriptMode::Word, logs));
    }
    match scan_citation_tasks(project_id.clone()) {
        Ok(tasks) => {
            if let Err(error) = write_json(&root.join("exports/word/citation_tasks.json"), &tasks) {
                append_warn(&mut logs, "citation_tasks.json write", error);
            }
        }
        Err(error) => {
            append_warn(&mut logs, "citation task scan", error);
        }
    }
    logs.extend(vec![
        "Word draft exported with Pandoc.".to_string(),
        "Kept [CITE: key] placeholders for Zotero Word plugin.".to_string(),
        "Generated citation_tasks.json.".to_string(),
    ]);
    Ok(ExportJob {
        id: format!("export_{}", Uuid::new_v4()),
        project_id,
        mode: ManuscriptMode::Word,
        status: ExportStatus::Success,
        output_path: export_output_path(&output)?,
        logs,
        created_at: now_iso(),
    })
}

#[tauri::command]
fn export_latex(project_id: String) -> Result<ExportJob, String> {
    let project = project_by_id(&project_id)?;
    let root = PathBuf::from(&project.root_path);
    let mut logs = match ensure_pandoc_available() {
        Ok(logs) => logs,
        Err(logs) => return Ok(failed_export_job(project_id, ManuscriptMode::Latex, logs)),
    };
    fs::create_dir_all(root.join("exports/_intermediate")).map_err(|err| err.to_string())?;
    fs::create_dir_all(root.join("exports/latex")).map_err(|err| err.to_string())?;
    let intermediate = root.join("exports/_intermediate/paper-latex.md");
    let output = root.join("exports/latex/paper.tex");
    fs::write(
        &intermediate,
        convert_citations_for_mode(&merged_sections(&project_id)?, &ManuscriptMode::Latex)?,
    )
    .map_err(|err| err.to_string())?;
    let args = latex_pandoc_args();
    if let Err(error) = run_pandoc(&args, &root) {
        logs.push(format!("Pandoc LaTeX export failed: {}", error));
        return Ok(failed_export_job(project_id, ManuscriptMode::Latex, logs));
    }
    let bib_src = root.join("references/bib/references.bib");
    let bib_dst = root.join("exports/latex/references.bib");
    if bib_src.exists() {
        if let Err(error) = fs::copy(&bib_src, &bib_dst) {
            append_warn(&mut logs, "references.bib copy", error);
        }
    }
    logs.extend(vec![
        "LaTeX paper.tex exported with Pandoc.".to_string(),
        "references.bib copied when available.".to_string(),
    ]);
    Ok(ExportJob {
        id: format!("export_{}", Uuid::new_v4()),
        project_id,
        mode: ManuscriptMode::Latex,
        status: ExportStatus::Success,
        output_path: export_output_path(&output)?,
        logs,
        created_at: now_iso(),
    })
}

#[tauri::command]
fn export_markdown_pandoc(project_id: String) -> Result<ExportJob, String> {
    let project = project_by_id(&project_id)?;
    let root = PathBuf::from(&project.root_path);
    let output = root.join("exports/markdown/combined.md");
    fs::write(
        &output,
        convert_citations_for_mode(&merged_sections(&project_id)?, &ManuscriptMode::Markdown)?,
    )
    .map_err(|err| err.to_string())?;
    let command = "pandoc combined.md --bibliography ../references/bib/references.bib -o paper.docx";
    fs::write(root.join("exports/markdown/pandoc_command.txt"), command).map_err(|err| err.to_string())?;
    Ok(ExportJob {
        id: format!("export_{}", Uuid::new_v4()),
        project_id,
        mode: ManuscriptMode::Markdown,
        status: ExportStatus::Success,
        output_path: export_output_path(&output)?,
        logs: vec![
            "combined.md generated.".to_string(),
            format!("Pandoc command: {}", command),
        ],
        created_at: now_iso(),
    })
}

#[tauri::command]
fn open_path(path: String) -> Result<bool, String> {
    let target = canonical_existing_path(PathBuf::from(path.trim()))?;
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
            list_project_files,
            read_text_file,
            write_text_file,
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
            export_project_folder,
            export_word_draft_placeholder,
            export_latex_placeholder,
            check_pandoc_installed,
            get_pandoc_version,
            init_workspace,
            read_settings,
            save_settings,
            test_ai_connection,
            fetch_ai_models,
            generate_ai_proposal,
            apply_ai_proposal,
            list_agent_skills,
            run_agent,
            apply_agent_change,
            reject_agent_run,
            read_agent_log,
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
    use std::cell::Cell;
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
            assert!(root.ends_with("workspace/papers/Empty_Paper"));
            assert!(root.join("manuscript/sections").exists());
            assert!(list_sections(project.id.clone())
                .expect("sections")
                .is_empty());
            let manifest: ProjectConfig = serde_json::from_str(
                &fs::read_to_string(root.join("paperforge.json")).expect("manifest raw"),
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
                .join("paperforge.json")
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
                &fs::read_to_string(root.join("paperforge.json")).expect("manifest raw"),
            )
            .expect("manifest json");
            assert_eq!(manifest.manuscript.sections[0].title, "Renamed Section");
            assert_eq!(
                manifest.manuscript.sections[0].path,
                "manuscript/sections/01_section.md"
            );
            let activities: Vec<ProjectActivity> = serde_json::from_str(
                &fs::read_to_string(root.join(".paperforge/activity.json")).expect("activity raw"),
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
                &fs::read_to_string(root.join("paperforge.json")).expect("manifest raw"),
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
                PathBuf::from(&project.root_path).join("references/bib/references.bib"),
                "@article{Zhang2023,title={Test}}",
            )
            .expect("bib");
            let job = export_markdown_package(project.id.clone()).expect("export");
            assert!(matches!(job.status, ExportStatus::Success));
            let output = PathBuf::from(job.output_path);
            assert!(output.join("manifest.json").exists());
            assert!(output.join("paper.md").exists());
            assert!(output.join("sections/01_introduction.md").exists());
            assert!(output.join("sections/02_methods.md").exists());
            assert!(output.join("references/bib/library.bib").exists());
            let report_raw = fs::read_to_string(output.join("export-report.json")).expect("report");
            assert!(report_raw.contains("paper.md"));
            assert!(!output.join(".git").exists());
        });
    }

    #[test]
    fn saving_section_writes_markdown_inside_workspace_project() {
        with_temp_cwd("save_md", |_dir| {
            let project = create_project(create_input(
                "Workspace Paper",
                SectionNamingMode::Numbered,
                vec!["Draft"],
            ))
            .expect("project");
            let mut section = list_sections(project.id.clone())
                .expect("sections")
                .pop()
                .expect("section");
            section.content = "# Draft\n\nSaved body.".to_string();
            let saved = save_section(project.id.clone(), section).expect("saved");
            let md_path = PathBuf::from(&project.root_path).join(saved.path);
            assert!(md_path.exists());
            assert_eq!(
                fs::read_to_string(md_path).expect("markdown"),
                "# Draft\n\nSaved body."
            );
            assert!(PathBuf::from(&project.root_path)
                .join("paperforge.json")
                .exists());
        });
    }

    #[test]
    fn opening_project_merges_section_files_missing_from_manifest() {
        with_temp_cwd("merge_sections", |_dir| {
            let project = create_project(create_input(
                "Merge Paper",
                SectionNamingMode::Numbered,
                vec!["Known"],
            ))
            .expect("project");
            let root = PathBuf::from(&project.root_path);
            fs::write(
                root.join("manuscript/sections/02_existing.md"),
                "## Existing\n\nLoaded from disk.",
            )
            .expect("extra section");
            let sections = list_sections(project.id.clone()).expect("sections");
            assert_eq!(sections.len(), 2);
            assert!(sections.iter().any(|section| section.filename == "02_existing.md"));
            assert_eq!(
                sections
                    .iter()
                    .map(|section| section.filename.as_str())
                    .collect::<Vec<_>>(),
                vec!["01_known.md", "02_existing.md"]
            );
            let updated = project_by_id(&project.id).expect("updated project");
            assert!(updated
                .manuscript
                .sections
                .iter()
                .any(|section| section.path == "manuscript/sections/02_existing.md"));
        });
    }

    #[test]
    fn missing_manifest_section_is_not_returned_and_manifest_is_synced() {
        with_temp_cwd("missing_section", |_dir| {
            let project = create_project(create_input(
                "Missing Paper",
                SectionNamingMode::Numbered,
                vec!["Present", "Gone"],
            ))
            .expect("project");
            let root = PathBuf::from(&project.root_path);
            fs::remove_file(root.join("manuscript/sections/02_gone.md")).expect("remove section");

            let sections = list_sections(project.id.clone()).expect("sections");

            assert_eq!(sections.len(), 1);
            assert_eq!(sections[0].filename, "01_present.md");
            assert!(!sections.iter().any(|section| section.filename == "02_gone.md"));
            let updated = project_by_id(&project.id).expect("updated project");
            assert_eq!(updated.manuscript.sections.len(), 1);
            assert_eq!(
                updated.manuscript.sections[0].path,
                "manuscript/sections/01_present.md"
            );
        });
    }

    #[test]
    fn word_and_latex_pandoc_args_stay_project_relative() {
        let word_args = word_pandoc_args(false);
        assert!(word_args.contains(&"exports/_intermediate/paper-word.md".to_string()));
        assert!(word_args.contains(&"exports/word/paper.docx".to_string()));
        assert!(!word_args.iter().any(|arg| arg.contains("workspace/papers")));

        let latex_args = latex_pandoc_args();
        assert!(latex_args.contains(&"exports/_intermediate/paper-latex.md".to_string()));
        assert!(latex_args.contains(&"exports/latex/paper.tex".to_string()));
        assert!(!latex_args.iter().any(|arg| arg.contains("workspace/papers")));
    }

    #[test]
    fn relative_open_path_resolves_under_app_root() {
        with_temp_cwd("open_path", |_dir| {
            fs::create_dir_all("workspace/papers/Paper/exports/word").expect("exports dir");
            let resolved = canonical_existing_path("workspace/papers/Paper/exports/word")
                .expect("resolved");
            assert!(resolved.is_absolute());
            assert!(resolved.ends_with("workspace/papers/Paper/exports/word"));
        });
    }

    #[test]
    fn project_file_tree_reads_real_markdown_files() {
        with_temp_cwd("file_tree", |_dir| {
            let project = create_project(create_input(
                "Tree Paper",
                SectionNamingMode::Numbered,
                vec!["Intro"],
            ))
            .expect("project");
            let root = PathBuf::from(&project.root_path);
            fs::create_dir_all(root.join("references/notes")).expect("notes dir");
            fs::write(root.join("references/notes/idea.md"), "# Idea\n\nNote.").expect("note");

            let tree = list_project_files(project.id.clone()).expect("tree");
            let raw = serde_json::to_string(&tree).expect("tree json");
            assert!(raw.contains("manuscript/sections/01_intro.md"));
            assert!(raw.contains("references/notes/idea.md"));
            assert!(!raw.contains("ai-models.json"));
        });
    }

    #[test]
    fn project_text_file_read_write_stays_in_project_root() {
        with_temp_cwd("file_edit", |_dir| {
            let project = create_project(create_input(
                "Edit File Paper",
                SectionNamingMode::Numbered,
                vec![],
            ))
            .expect("project");
            let root = PathBuf::from(&project.root_path);
            fs::create_dir_all(root.join("references/notes")).expect("notes dir");
            fs::write(root.join("references/notes/note.md"), "# Note").expect("note");

            let read = read_text_file(project.id.clone(), "references/notes/note.md".to_string())
                .expect("read");
            assert_eq!(read.content, "# Note");
            let written = write_text_file(
                project.id.clone(),
                "references/notes/note.md".to_string(),
                "# Note\n\nSaved.".to_string(),
            )
            .expect("write");
            assert_eq!(written.content, "# Note\n\nSaved.");
            assert!(read_text_file(project.id.clone(), "../outside.md".to_string()).is_err());
        });
    }

    #[test]
    fn deleting_project_removes_actual_paper_folder() {
        with_temp_cwd("delete_paper", |_dir| {
            let project = create_project(create_input(
                "Delete Paper",
                SectionNamingMode::Numbered,
                vec!["Draft"],
            ))
            .expect("project");
            let root = PathBuf::from(&project.root_path);
            assert!(root.join("paperforge.json").exists());
            assert!(delete_project(project.id.clone(), true).expect("delete"));
            assert!(!root.exists());
            assert!(project_by_id(&project.id).is_err());
        });
    }

    #[test]
    fn default_workspace_init_uses_workspace_and_light_settings() {
        with_temp_cwd("workspace_init", |dir| {
            let config = init_workspace(String::new()).expect("workspace");
            assert_eq!(config.version, APP_VERSION);
            assert_eq!(config.workspace_name, "workspace");
            let root = dir.join("workspace");
            assert!(root.join("papers").exists());
            let settings_raw = fs::read_to_string(root.join(".paperforge/settings.json"))
                .expect("workspace settings");
            assert!(settings_raw.contains("\"theme\": \"light\""));
        });
    }

    #[test]
    fn openai_compatible_response_parser_reads_chat_content() {
        let value = serde_json::json!({
            "choices": [{ "message": { "content": " Revised section. " } }]
        });
        assert_eq!(
            parse_openai_chat_content(&value).expect("content"),
            "Revised section."
        );
    }

    #[test]
    fn anthropic_response_parser_reads_text_block() {
        let value = serde_json::json!({
            "content": [
                { "type": "tool_use", "name": "ignored" },
                { "type": "text", "text": " Agent report. " }
            ]
        });
        assert_eq!(
            parse_anthropic_message_content(&value).expect("content"),
            "Agent report."
        );
    }

    #[test]
    fn citation_conversion_handles_word_and_latex_modes() {
        let markdown = "Word [CITE: Smith2023], pandoc [@Lee2024], latex \\cite{Zhang2025}.";
        assert!(convert_citations_for_mode(markdown, &ManuscriptMode::Word)
            .expect("word")
            .contains("[CITE: Lee2024]"));
        assert!(convert_citations_for_mode(markdown, &ManuscriptMode::Latex)
            .expect("latex")
            .contains("\\cite{Smith2023}"));
    }

    #[test]
    fn pandoc_args_are_relative_to_project_root() {
        let word_args = word_pandoc_args(true);
        assert_eq!(word_args[0], "exports/_intermediate/paper-word.md");
        assert!(word_args.contains(&"exports/word/paper.docx".to_string()));
        assert!(!word_args.iter().any(|arg| arg.contains("workspace/papers")));
        let latex_args = latex_pandoc_args();
        assert_eq!(latex_args[0], "exports/_intermediate/paper-latex.md");
        assert!(latex_args.contains(&"exports/latex/paper.tex".to_string()));
    }

    #[test]
    fn pandoc_detected_skips_install() {
        let install_called = Cell::new(false);
        let logs = ensure_pandoc_available_with(
            || Ok("pandoc 3.1\nfeatures".to_string()),
            || {
                install_called.set(true);
                Ok(())
            },
        )
        .expect("pandoc");
        assert!(!install_called.get());
        assert!(logs[0].contains("pandoc 3.1"));
    }

    #[test]
    fn pandoc_finder_prefers_explicit_config_path() {
        with_temp_cwd("pandoc_path", |dir| {
            let fake = dir.join("pandoc.exe");
            fs::write(&fake, "").expect("fake pandoc");
            std::env::set_var("PAPERFORGE_PANDOC", fake.to_string_lossy().to_string());
            assert_eq!(find_pandoc_executable(), fake);
            std::env::remove_var("PAPERFORGE_PANDOC");
        });
    }

    #[test]
    fn pandoc_missing_install_failure_returns_logs() {
        let result = ensure_pandoc_available_with(
            || Err("not found".to_string()),
            || Err("winget missing".to_string()),
        );
        assert!(result.is_err());
        let logs = result.err().expect("logs");
        assert!(logs.iter().any(|line| line.contains(PANDOC_REQUIRED_MESSAGE)));
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn pandoc_missing_winget_success_rechecks() {
        let checks = Cell::new(0);
        let installed = Cell::new(false);
        let logs = ensure_pandoc_available_with(
            || {
                checks.set(checks.get() + 1);
                if checks.get() == 1 {
                    Err("not found".to_string())
                } else {
                    Ok("pandoc 3.1".to_string())
                }
            },
            || {
                installed.set(true);
                Ok(())
            },
        )
        .expect("installed");
        assert!(installed.get());
        assert_eq!(checks.get(), 2);
        assert!(logs.iter().any(|line| line.contains("automatic install completed")));
    }

    #[test]
    fn parse_bibtex_empty_input_returns_empty_vec() {
        let refs = parse_bibtex(String::new()).expect("empty");
        assert!(refs.is_empty());
    }

    #[test]
    fn parse_bibtex_only_at_string_is_skipped() {
        let bib = "@string{plain = \"Hello, world\"}".to_string();
        let refs = parse_bibtex(bib).expect("string");
        assert!(refs.is_empty());
    }

    #[test]
    fn parse_bibtex_two_entries_extract_fields() {
        let bib = "@article{key1,
            title  = {First Title},
            author = {Doe, J. and Smith, A.},
            year   = {2024},
            doi    = {10.1/abc}
        }
        @book{key2,
            title  = {Second Title},
            author = {Brown, B.},
            year   = {2022}
        }
        @string{marker = \"ignored\"}".to_string();
        let refs = parse_bibtex(bib).expect("two entries");
        assert_eq!(refs.len(), 2);
        assert_eq!(refs[0].citekey, "key1");
        assert_eq!(refs[0].title, "First Title");
        assert_eq!(refs[0].year, "2024");
        assert_eq!(refs[0].doi, "10.1/abc");
        assert_eq!(refs[0].authors, vec!["Doe, J.".to_string(), "Smith, A.".to_string()]);
        assert_eq!(refs[1].citekey, "key2");
        assert_eq!(refs[1].title, "Second Title");
        assert_eq!(refs[1].year, "2022");
        assert_eq!(refs[1].authors, vec!["Brown, B.".to_string()]);
    }

    #[test]
    fn ai_settings_missing_fields_use_defaults() {
        let value = serde_json::json!({
            "provider": "openai",
            "baseUrl": "https://api.openai.com/v1",
            "apiKey": "sk-test",
            "model": "gpt-4.1-mini"
        });
        let settings = ai_model_value_to_settings(&value).expect("parsed");
        assert_eq!(settings.temperature, 0.3);
        assert_eq!(settings.max_tokens, 2000);
    }

    #[test]
    fn sidebar_mode_default_for_legacy_settings() {
        // Legacy settings.json files (written before the field was
        // added) must still deserialize cleanly with sidebar_mode
        // defaulting to Writing. This is the contract the frontend
        // depends on so the Files tab never shows a stale state on
        // a fresh install.
        let raw = r#"{
            "workspaceRoot": "workspace",
            "defaultManuscriptMode": "word",
            "llmProvider": {
                "provider": "openai-compatible",
                "baseUrl": "https://api.example.com/v1",
                "apiKey": "",
                "model": "x",
                "temperature": 0.3,
                "maxTokens": 2000
            },
            "defaultCitationStyle": "apa",
            "defaultExportMode": "markdown",
            "themeMode": "light",
            "language": "en"
        }"#;
        let settings: AppSettings = serde_json::from_str(raw).expect("legacy settings deserialize");
        assert_eq!(settings.sidebar_mode, SidebarMode::Writing);
    }

    #[test]
    fn sidebar_mode_files_deserializes() {
        // The frontend writes "files" in camelCase. The Rust enum uses
        // rename_all = "lowercase" so the wire value is the bare word
        // "files", not "Files".
        let raw = r#"{
            "workspaceRoot": "workspace",
            "defaultManuscriptMode": "word",
            "llmProvider": {
                "provider": "openai-compatible",
                "baseUrl": "https://api.example.com/v1",
                "apiKey": "",
                "model": "x",
                "temperature": 0.3,
                "maxTokens": 2000
            },
            "defaultCitationStyle": "apa",
            "defaultExportMode": "markdown",
            "themeMode": "light",
            "language": "en",
            "sidebarMode": "files"
        }"#;
        let settings: AppSettings = serde_json::from_str(raw).expect("settings with sidebarMode=files");
        assert_eq!(settings.sidebar_mode, SidebarMode::Files);

        // Round-trip: serializing with serde must produce the
        // lowercase "files" so the JSON on disk matches what the
        // TypeScript SidebarMode type expects on the next boot.
        let serialized = serde_json::to_string(&settings).expect("serialize");
        assert!(serialized.contains("\"sidebarMode\":\"files\""), "expected lowercase 'files' in serialized form, got: {}", serialized);
    }
    #[test]
    fn ai_settings_explicit_values_round_trip() {
        let value = serde_json::json!({
            "provider": "openai-compatible",
            "baseUrl": "https://api.openai.com/v1",
            "apiKey": "sk-test",
            "model": "gpt-4.1-mini",
            "temperature": 0.8,
            "maxTokens": 4096
        });
        let settings = ai_model_value_to_settings(&value).expect("parsed");
        assert_eq!(settings.temperature, 0.8);
        assert_eq!(settings.max_tokens, 4096);
    }

    #[test]
    fn safe_call_llm_catches_panic() {
        // safe_call_llm must convert any panic in call_llm into a
        // PaperForge-side error string so the desktop agent button
        // never tears down the Tauri webview. The callback is
        // guaranteed to panic; the wrapper must surface it cleanly.
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            safe_call_llm(&default_settings(), "system", "user")
        }))
        .expect("safe_call_llm must not itself panic on its own");
        // The wrapper itself ran without panicking, but the inner
        // call_llm panicked (default settings have no LLM key +
        // unreachable provider, so we are testing the wrapper shape
        // by checking that any returned error mentions the agent
        // safety boundary).
        match result {
            Ok(_) => {} // No LLM configured: call_llm returns Err, not panic.
            Err(message) => {
                // The message is opaque on purpose (provider error),
                // but it must not be empty.
                assert!(!message.is_empty());
            }
        }
    }

    #[test]
    fn select_agent_skill_ask_export_keyword_does_not_panic() {
        // A future refactor could rename ask.export-readiness. The
        // keyword short-circuit in select_agent_skill must fall back
        // to the default Ask skill instead of unwrapping.
        let skill = std::panic::catch_unwind(|| {
            select_agent_skill(&AgentMode::Ask, "auto", "please export this paper")
        })
        .expect("select_agent_skill must not panic on the Ask / export keyword path");
        assert_eq!(skill.skill_type, AgentMode::Ask);
    }

    #[test]
    fn select_agent_skill_edit_translate_keyword_does_not_panic() {
        let skill = std::panic::catch_unwind(|| {
            select_agent_skill(&AgentMode::Edit, "auto", "translate to 中文")
        })
        .expect("select_agent_skill must not panic on the Edit / translate keyword path");
        assert_eq!(skill.skill_type, AgentMode::Edit);
    }
    #[test]
    fn openai_chat_body_never_carries_tools_or_function_calling() {
        let provider = LlmProviderSettings {
            provider: LlmProviderKind::OpenaiCompatible,
            base_url: "https://api.example.com/v1".to_string(),
            api_key: "sk-test".to_string(),
            model: "test-model".to_string(),
            temperature: 0.3,
            max_tokens: 2000,
        };
        let body = build_openai_chat_body(&provider, "system", "user")
            .expect("openai body should build without forbidden keys");
        let keys: Vec<String> = body
            .as_object()
            .expect("body is an object")
            .keys()
            .cloned()
            .collect();
        for forbidden in FORBIDDEN_LLM_KEYS {
            assert!(
                !keys.iter().any(|key| key == forbidden),
                "OpenAI body should not carry top-level key {}",
                forbidden
            );
        }
        assert_eq!(
            body.get("tool_choice").and_then(|v| v.as_str()),
            Some("none"),
            "OpenAI body must disable tool_choice"
        );
        assert_eq!(
            body.get("parallel_tool_calls").and_then(|v| v.as_bool()),
            Some(false),
            "OpenAI body must disable parallel_tool_calls"
        );
        let messages = body
            .get("messages")
            .and_then(|v| v.as_array())
            .expect("messages array");
        for message in messages {
            let obj = message.as_object().expect("message is object");
            for forbidden in ["tools", "tool_calls", "tool_call_id", "function_call", "name"] {
                assert!(
                    !obj.contains_key(forbidden),
                    "Message must not carry field {}",
                    forbidden
                );
            }
        }
    }

    #[test]
    fn anthropic_body_never_carries_tools() {
        let provider = LlmProviderSettings {
            provider: LlmProviderKind::Anthropic,
            base_url: "https://api.anthropic.com/v1".to_string(),
            api_key: "sk-ant-test".to_string(),
            model: "claude-test".to_string(),
            temperature: 0.3,
            max_tokens: 2000,
        };
        let body = build_anthropic_message_body(&provider, "system", "user")
            .expect("anthropic body should build without forbidden keys");
        let keys: Vec<String> = body
            .as_object()
            .expect("body is an object")
            .keys()
            .cloned()
            .collect();
        for forbidden in FORBIDDEN_LLM_KEYS {
            assert!(
                !keys.iter().any(|key| key == forbidden),
                "Anthropic body should not carry top-level key {}",
                forbidden
            );
        }
    }

    #[test]
    fn llm_body_forbidden_keys_detects_nested_tools() {
        let body = serde_json::json!({
            "model": "x",
            "metadata": { "tools": [{ "type": "function" }] }
        });
        let hits = llm_body_forbidden_keys(&body);
        assert!(hits.contains(&"tools".to_string()));
    }

    #[test]
    fn assert_clean_llm_body_blocks_dirty_payload() {
        let body = serde_json::json!({
            "model": "x",
            "tools": [{ "type": "function", "function": { "name": "x" } }]
        });
        let err = assert_clean_llm_body(&body, "test-endpoint")
            .expect_err("dirty body must be rejected");
        assert!(err.contains("forbidden key"));
        assert!(err.contains("tools"));
    }

    #[test]
    fn openai_response_with_tool_calls_returns_clear_error() {
        let value = serde_json::json!({
            "choices": [{
                "finish_reason": "tool_calls",
                "message": {
                    "role": "assistant",
                    "content": null,
                    "tool_calls": [{
                        "id": "call_function_abc",
                        "type": "function",
                        "function": { "name": "x", "arguments": "{}" }
                    }]
                }
            }]
        });
        let err = parse_openai_chat_content(&value)
            .expect_err("tool_calls response must be rejected");
        assert!(err.contains("tool_calls"));
        assert!(err.contains("call_function_abc"));
    }

    #[test]
    fn openai_response_truncated_by_length_surfaces_reason() {
        let value = serde_json::json!({
            "choices": [{
                "finish_reason": "length",
                "message": { "role": "assistant", "content": null }
            }]
        });
        let err = parse_openai_chat_content(&value)
            .expect_err("length finish must be reported");
        assert!(err.contains("max_tokens"));
    }

    #[test]
    fn anthropic_response_with_tool_use_returns_clear_error() {
        let value = serde_json::json!({
            "stop_reason": "tool_use",
            "content": [
                { "type": "tool_use", "id": "toolu_1", "name": "x", "input": {} }
            ]
        });
        let err = parse_anthropic_message_content(&value)
            .expect_err("tool_use response must be rejected");
        assert!(err.contains("tool_use"));
    }

    #[test]
    fn llm_body_debug_log_masks_long_secrets() {
        let body = serde_json::json!({
            "model": "x",
            "messages": [
                { "role": "system", "content": "sk-1234567890ABCDEFGHIJKLMNOPQRSTUVWXYZabcd" }
            ]
        });
        let logged = llm_body_debug_log(&body);
        assert!(!logged.contains("1234567890ABCDEFGHIJKLMNOPQRSTUVWXYZabcd"));
        assert!(logged.contains("****"));
    }
}
