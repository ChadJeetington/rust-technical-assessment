use super::*;
use git2::{Repository, FetchOptions, RemoteCallbacks};

use std::path::PathBuf;
use tokio::fs;
use tracing::{info, warn};
use walkdir::WalkDir;

pub struct UniswapDocSource {
    /// Base directory for storing cloned repositories
    base_dir: PathBuf,
    /// Repository configurations
    repos: Vec<UniswapRepoConfig>,
    /// Git credentials if needed
    credentials: Option<GitCredentials>,
}

struct UniswapRepoConfig {
    url: String,
    branch: String,
    doc_paths: Vec<String>,
    version: String,
}

struct GitCredentials {
    username: String,
    token: String,
}

impl UniswapDocSource {
    pub fn new(base_dir: PathBuf) -> Self {
        // Create base directory if it doesn't exist
        if !base_dir.exists() {
            std::fs::create_dir_all(&base_dir).expect("Failed to create base directory");
        }

        let repos = vec![
            // Start with just v2-core to test
            UniswapRepoConfig {
                url: "https://github.com/Uniswap/v2-core".to_string(),
                branch: "master".to_string(), // v2-core uses master branch
                doc_paths: vec!["contracts/".to_string()],
                version: "v2".to_string(),
            },
            // Temporarily commenting out other repos until we get v2-core working
            /*
            UniswapRepoConfig {
                url: "https://github.com/Uniswap/v2-periphery".to_string(),
                branch: "master".to_string(),
                doc_paths: vec!["contracts/".to_string()],
                version: "v2".to_string(),
            },
            UniswapRepoConfig {
                url: "https://github.com/Uniswap/v3-core".to_string(),
                branch: "main".to_string(),
                doc_paths: vec!["contracts/".to_string()],
                version: "v3".to_string(),
            },
            UniswapRepoConfig {
                url: "https://github.com/Uniswap/v3-periphery".to_string(),
                branch: "main".to_string(),
                doc_paths: vec!["contracts/".to_string()],
                version: "v3".to_string(),
            },
            UniswapRepoConfig {
                url: "https://github.com/Uniswap/docs".to_string(),
                branch: "main".to_string(),
                doc_paths: vec!["docs/".to_string()],
                version: "latest".to_string(),
            },
            */
        ];
        
        Self {
            base_dir,
            repos,
            credentials: None,
        }
    }
    
    pub fn with_credentials(mut self, username: String, token: String) -> Self {
        self.credentials = Some(GitCredentials { username, token });
        self
    }
    
    async fn clone_or_pull_repo(&self, config: &UniswapRepoConfig) -> Result<PathBuf, IngestionError> {
        let repo_name = config.url.split('/').last()
            .ok_or_else(|| IngestionError::FetchError("Invalid repo URL".to_string()))?;
        let repo_path = self.base_dir.join(repo_name);

        // Create callbacks that can be cloned
        let make_callbacks = || -> RemoteCallbacks<'static> {
            let mut callbacks = RemoteCallbacks::new();
            if let Some(creds) = &self.credentials {
                let username = creds.username.clone();
                let token = creds.token.clone();
                callbacks.credentials(move |_, _, _| {
                    git2::Cred::userpass_plaintext(&username, &token)
                });
            }
            callbacks
        };
        
        if repo_path.exists() {
            // Pull updates
            let repo = Repository::open(&repo_path)
                .map_err(|e| IngestionError::FetchError(format!("Failed to open repo: {}", e)))?;
            
            let mut remote = repo.find_remote("origin")
                .map_err(|e| IngestionError::FetchError(format!("Failed to find remote: {}", e)))?;
            
            let mut fetch_options = FetchOptions::new();
            fetch_options.remote_callbacks(make_callbacks());
            
            remote.fetch(&[&config.branch], Some(&mut fetch_options), None)
                .map_err(|e| IngestionError::FetchError(format!("Failed to fetch: {}", e)))?;
        } else {
            // Clone repository
            info!("   🔄 Cloning repository {} to {}", config.url, repo_path.display());
            
            let mut clone_options = FetchOptions::new();
            clone_options.remote_callbacks(make_callbacks());
            
            let mut builder = git2::build::RepoBuilder::new();
            // Don't specify branch during clone
            builder.fetch_options(clone_options);

            match builder.clone(&config.url, &repo_path) {
                Ok(_) => {
                    info!("   ✅ Successfully cloned repository");
                    
                    // Open the repository
                    let repo = Repository::open(&repo_path)
                        .map_err(|e| IngestionError::FetchError(format!("Failed to open repo after clone: {}", e)))?;
                    
                    // Get the default branch
                    let head = repo.head()
                        .map_err(|e| IngestionError::FetchError(format!("Failed to get HEAD: {}", e)))?;
                    
                    info!("   📝 Repository cloned with branch: {}", head.shorthand().unwrap_or("unknown"));
                },
                Err(e) => {
                    warn!("   ⚠️ Failed to clone repository: {}", e);
                    if e.code() == git2::ErrorCode::Exists {
                        info!("   📂 Repository already exists, trying to open and pull...");
                        let repo = Repository::open(&repo_path)
                            .map_err(|e| IngestionError::FetchError(format!("Failed to open existing repo: {}", e)))?;
                        
                        // Try to pull latest changes
                        let mut remote = repo.find_remote("origin")
                            .map_err(|e| IngestionError::FetchError(format!("Failed to find remote: {}", e)))?;
                        
                        let mut pull_options = FetchOptions::new();
                        pull_options.remote_callbacks(make_callbacks());
                        
                        info!("   🔄 Pulling latest changes...");
                        remote.fetch(&["refs/heads/*:refs/heads/*"], Some(&mut pull_options), None)
                            .map_err(|e| IngestionError::FetchError(format!("Failed to fetch: {}", e)))?;
                        
                        info!("   ✅ Successfully updated repository");
                    } else {
                        return Err(IngestionError::FetchError(format!("Failed to clone: {}", e)));
                    }
                }
            };
        }
        
        Ok(repo_path)
    }
}

#[async_trait]
impl DocumentSource for UniswapDocSource {
    async fn fetch_documents(&self) -> Result<Vec<RawDocument>, IngestionError> {
        let mut documents = Vec::new();
        
        info!("🔍 Starting to fetch Uniswap documentation...");
        for config in &self.repos {
            info!("📦 Processing repository: {}", config.url);
            let repo_path = self.clone_or_pull_repo(config).await?;
            
            for doc_path in &config.doc_paths {
                let full_path = repo_path.join(doc_path);
                
                for entry in WalkDir::new(&full_path)
                    .into_iter()
                    .filter_map(Result::ok)
                    .filter(|e| e.file_type().is_file())
                {
                    let path = entry.path();
                    let doc_type = match path.extension().and_then(|ext| ext.to_str()) {
                        Some("sol") => DocumentType::Solidity,
                        Some("md") | Some("mdx") => DocumentType::Markdown,
                        Some("json") => DocumentType::JSON,
                        _ => continue,
                    };
                    let doc_type_str = doc_type.to_string();
                    
                    info!("   Processing file: {}", path.display());
                    
                    let content = match fs::read(path).await {
                        Ok(content) => content,
                        Err(e) => {
                            warn!("   ⚠️ Failed to read file {}: {}", path.display(), e);
                            continue;
                        }
                    };
                    
                    let relative_path = path.strip_prefix(&repo_path)
                        .map_err(|e| IngestionError::ProcessingError(e.to_string()))?;
                    
                    let metadata = DocumentMetadata {
                        title: path.file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("Unknown")
                            .to_string(),
                        doc_type,
                        version: config.version.clone(),
                        created_at: Utc::now(),
                        updated_at: Utc::now(),
                        source: DocumentSourceMetadata {
                            source_type: "git".to_string(),
                            location: config.url.clone(),
                            version: Some(config.branch.clone()),
                        },
                        tags: vec![
                            config.version.clone(),
                            doc_type_str,
                            relative_path.to_string_lossy().to_string(),
                        ],
                    };
                    
                    documents.push(RawDocument::new(content, metadata));
                }
            }
        }
        
        Ok(documents)
    }
    
    async fn has_updates(&self) -> Result<bool, IngestionError> {
        for config in &self.repos {
            let repo_path = self.base_dir.join(
                config.url.split('/').last()
                    .ok_or_else(|| IngestionError::FetchError("Invalid repo URL".to_string()))?
            );
            
            if !repo_path.exists() {
                return Ok(true);
            }
            
            let repo = Repository::open(&repo_path)
                .map_err(|e| IngestionError::FetchError(format!("Failed to open repo: {}", e)))?;
            
            let mut remote = repo.find_remote("origin")
                .map_err(|e| IngestionError::FetchError(format!("Failed to find remote: {}", e)))?;
            
            let mut fetch_options = FetchOptions::new();
            fetch_options.remote_callbacks({
                let mut callbacks = RemoteCallbacks::new();
                if let Some(creds) = &self.credentials {
                    let username = creds.username.clone();
                    let token = creds.token.clone();
                    callbacks.credentials(move |_, _, _| {
                        git2::Cred::userpass_plaintext(&username, &token)
                    });
                }
                callbacks
            });
            
            remote.fetch(&[&config.branch], Some(&mut fetch_options), None)
                .map_err(|e| IngestionError::FetchError(format!("Failed to fetch: {}", e)))?;
            
            let head = repo.head()
                .map_err(|e| IngestionError::FetchError(format!("Failed to get HEAD: {}", e)))?;
            
            let local_commit = head.target()
                .ok_or_else(|| IngestionError::FetchError("No HEAD commit".to_string()))?;
            
            let remote_branch = repo.find_branch(&config.branch, git2::BranchType::Remote)
                .map_err(|e| IngestionError::FetchError(format!("Failed to find remote branch: {}", e)))?;
            
            let remote_commit = remote_branch.get().target()
                .ok_or_else(|| IngestionError::FetchError("No remote commit".to_string()))?;
            
            if local_commit != remote_commit {
                return Ok(true);
            }
        }
        
        Ok(false)
    }
    
    fn get_metadata(&self) -> DocumentSourceMetadata {
        DocumentSourceMetadata {
            source_type: "uniswap".to_string(),
            location: "https://github.com/Uniswap".to_string(),
            version: Some("latest".to_string()),
        }
    }
}
