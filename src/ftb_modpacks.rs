use serde::{Deserialize, Serialize};

pub static FTB_API_URL: &str = "https://meta.feed-the-beast.com/v1/modpacks";

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct FTBModpackList {
    pub success: bool,
    pub packs: Vec<Modpack>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum FTBModpackError {
    IoError,
    ApiError,
    FormatError,
}

impl FTBModpackList {
    pub async fn get_all() -> Result<Self, FTBModpackError> {
        let results = reqwest::get(FTB_API_URL)
            .await
            .map_err(|_| FTBModpackError::ApiError)?;
        
        results.json::<Self>().await.map_err(|_| FTBModpackError::FormatError)
    }
    
    pub fn from_file(path: &str) -> Result<Self, FTBModpackError> {
        let raw = std::fs::read_to_string(path).map_err(|_| FTBModpackError::IoError)?;
        serde_json::from_str(&raw).map_err(|_|FTBModpackError::FormatError)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Modpack {
    pub id: i64,
    pub slug: String,    
    pub name: String,    
    pub synopsis: String,    
    pub r#type: String,    
    pub versions: Vec<ModpackVersion>,
    pub art: ModpackArt,
    pub stats: ModpackStats,   
    pub featured: bool,
    pub tags: Vec<String>,
    pub released: i64,
    pub updated: i64,
     
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ModpackVersion {
    pub id: i64,    
    pub name: String,    
    pub r#type: String,    
    pub minecraft: String,    
    pub loader: String,    
    pub loader_type: String,    
    pub memory: Memory,    
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Memory {
    pub min: i64,
    pub recommended: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ModpackArt {
    pub background: Option<String>,
    pub logo: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ModpackStats {
    pub plays: i64,
    pub installs: i64,
    pub plays_14d: i64,
}