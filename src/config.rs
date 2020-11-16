use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub default_path: String,
    pub projects: Option<Vec<ProjectConfig>>,
}

#[derive(Deserialize)]
pub struct ProjectConfig {
    pub name: Option<String>,
    pub path: Option<String>,
    pub url: String,
    pub description: Option<String>,
}
