#[derive(Debug, Clone)]
pub struct HTTPConfig {
    pub base_url: String,
    pub bearer_token: String,
}

#[derive(Debug, Clone)]
pub struct TongueConfig {
    pub name: String,
    pub api_key: String,
    pub endpoint: String,
    pub timeout: u64,
}
