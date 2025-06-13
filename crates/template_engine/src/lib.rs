// Placeholder for template_engine library crate

pub fn fetch_template_files(_source_repo: &str) -> Result<Vec<(String, Vec<u8>)>, String> {
    // For MVP, just return a single README.md file
    Ok(vec![(
        "README.md".to_string(),
        b"# Repo created by RepoRoller\n".to_vec(),
    )])
}

/// Trait for fetching template files.
pub trait TemplateFetcher: Send + Sync {
    fn fetch_template_files(&self, source: &str) -> Result<Vec<(String, Vec<u8>)>, String>;
}

/// Default implementation for fetching template files from a source repo.
pub struct DefaultTemplateFetcher;

impl TemplateFetcher for DefaultTemplateFetcher {
    fn fetch_template_files(&self, source: &str) -> Result<Vec<(String, Vec<u8>)>, String> {
        crate::fetch_template_files(source)
    }
}
