use crate::AppState;
use okf::linter::{Mode, Severity};
use serde::{Deserialize, Serialize};
use std::path::{Component, Path};
use std::str::FromStr;
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenBundleArgs {
    pub path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenBundleResult {
    pub status: String,
    pub doc_count: usize,
}

#[tauri::command]
pub async fn open_bundle(
    state: State<'_, AppState>,
    args: OpenBundleArgs,
) -> Result<OpenBundleResult, String> {
    let path = args.path.clone();
    let bundle_path = Path::new(&path);

    let engine = haven_index::IndexEngine::open(bundle_path.join(".haven").join("index.db"))
        .await
        .map_err(|e| e.to_string())?;
    engine
        .rebuild_from_bundle(bundle_path)
        .await
        .map_err(|e| e.to_string())?;

    let mut bundle = state.bundle_path.lock().await;
    *bundle = Some(path);

    let mut idx = state.index_engine.lock().await;
    *idx = Some(engine);

    Ok(OpenBundleResult {
        status: "opened".to_string(),
        doc_count: 0,
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateDocumentArgs {
    pub path: String,
    pub title: String,
    pub content: String,
    pub doc_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateDocumentResult {
    pub path: String,
    pub commit: Option<String>,
}

#[tauri::command]
pub async fn create_document(
    state: State<'_, AppState>,
    args: CreateDocumentArgs,
) -> Result<CreateDocumentResult, String> {
    let bundle_path = state
        .bundle_path
        .lock()
        .await
        .clone()
        .ok_or("No bundle open")?;
    validate_bundle_relative_path(&args.path)?;

    let doc_type = okf::DocType::from_str(&args.doc_type).map_err(|e| e.to_string())?;

    let now = chrono::Utc::now().to_rfc3339();
    let doc = okf::Document {
        frontmatter: okf::Frontmatter {
            doc_type,
            title: Some(args.title.clone()),
            description: None,
            resource: None,
            tags: None,
            timestamp: Some(now),
            okf_version: Some(okf::OKF_VERSION.to_string()),
            extra: serde_yaml::Value::Mapping(serde_yaml::Mapping::new()),
        },
        body: args.content.clone(),
    };
    let lint_messages = okf::Linter::new(Mode::Strict).lint_with_path(&doc, &args.path);
    if let Some(message) = lint_messages
        .iter()
        .find(|message| message.severity == Severity::Error)
    {
        return Err(message.message.clone());
    }
    let full_content = okf::to_markdown(&doc);

    let git = haven_git::GitHandle::open(&bundle_path)
        .or_else(|_| haven_git::GitHandle::init(&bundle_path))
        .map_err(|e| e.to_string())?;

    let author = haven_git::Author::human("Haven User", "user@haven.local");
    let commit_oid = git
        .commit_file(
            &args.path,
            &full_content,
            &author,
            &format!("Add {}", args.path),
        )
        .await
        .map_err(|e| e.to_string())?;

    let engine = state.index_engine.lock().await.clone();
    if let Some(engine) = engine {
        engine
            .index_document(&args.path, &args.title, doc_type.as_str(), &args.content)
            .await
            .map_err(|e| e.to_string())?;
        engine
            .rebuild_from_bundle(Path::new(&bundle_path))
            .await
            .map_err(|e| e.to_string())?;
    }

    Ok(CreateDocumentResult {
        path: args.path,
        commit: Some(commit_oid.to_string()),
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReadDocumentArgs {
    pub path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReadDocumentResult {
    pub path: String,
    pub raw: String,
}

#[tauri::command]
pub async fn read_document(
    state: State<'_, AppState>,
    args: ReadDocumentArgs,
) -> Result<ReadDocumentResult, String> {
    let bundle_path = state
        .bundle_path
        .lock()
        .await
        .clone()
        .ok_or("No bundle open")?;
    validate_bundle_relative_path(&args.path)?;

    let abs_path = Path::new(&bundle_path).join(&args.path);
    let raw = std::fs::read_to_string(&abs_path).map_err(|e| e.to_string())?;

    Ok(ReadDocumentResult {
        path: args.path,
        raw,
    })
}

fn validate_bundle_relative_path(path: &str) -> Result<(), String> {
    let path = Path::new(path);
    if path.is_absolute() {
        return Err("Document path must be bundle-relative".to_string());
    }
    if path.components().any(|component| {
        matches!(
            component,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        )
    }) {
        return Err("Document path cannot escape the bundle".to_string());
    }
    if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
        return Err("Document path must end in .md".to_string());
    }
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchArgs {
    pub query: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResultItem {
    pub path: String,
    pub title: String,
    pub snippet: String,
    pub score: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResultData {
    pub results: Vec<SearchResultItem>,
}

#[tauri::command]
pub async fn search_documents(
    state: State<'_, AppState>,
    args: SearchArgs,
) -> Result<SearchResultData, String> {
    let engine = state
        .index_engine
        .lock()
        .await
        .clone()
        .ok_or("No bundle open")?;

    let results = engine
        .search(&args.query)
        .await
        .map_err(|e| e.to_string())?;

    Ok(SearchResultData {
        results: results
            .into_iter()
            .map(|r| SearchResultItem {
                path: r.path,
                title: r.title,
                snippet: r.snippet,
                score: r.score,
            })
            .collect(),
    })
}
