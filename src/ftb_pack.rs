use std::path::{Path, PathBuf};
use serde::{Serialize, Deserialize};

pub static API_URL: &str = "https://api.modpacks.ch/public/modpack/";

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Pack {
    pub files: Vec<File>,
    
    pub specs: Specs,
    pub targets: Vec<Target>,

    pub installs: i64,
    pub refreshed: i64,
    pub changelog: String,
    pub parent: i64,
    pub notification: String,
    pub links: Vec<String>,
    pub status: String,
    pub id: i64,
    pub name: String,
    pub r#type: String,
    pub updated: i64,
    pub private: bool,
}

impl Pack {
    pub async fn get_from_id(pack_id: i64, version_id: i64) -> Result<Self, ()> {
        let result = reqwest::get(format!("{API_URL}{}/{}", pack_id, version_id))
            .await
            .map_err(|_|())?;
        
        result.json::<Self>().await.map_err(|_| ())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Specs {
    pub id: i64,
    pub minimum: i64,
    pub recommended: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Target {
    pub version: String,
    pub id: i64,
    pub name: String,
    pub r#type: String,
    pub updated: i64,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct File {
    pub version: String,
    pub path: Box<Path>,
    pub url: Option<String>,
    pub mirrors: Option<Vec<String>>,
    pub sha1: String,
    pub size: i64,
    pub tags: Vec<String>,
    pub clientonly: bool,
    pub serveronly: bool,
    pub optional: bool,
    pub id: u64,
    pub name: String,
    pub r#type: String,
    pub updated: i64,
    pub curseforge: Option<CurseForge>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CurseForge {
    pub project: i64,
    pub file: i64,    
}

pub async fn get_overrides(base: PathBuf, file: &File) {
    let file_name = file.name.to_owned();
    let Some(url) = file.url.to_owned() else {
        return;
    };

    let mut path = base.clone();
    path.push("overrides");
    path.push(file.path.clone());
    let _ = std::fs::create_dir_all(path.clone());
    path.push(file_name);
    
    
    if url.trim().eq("") {
        return;
    }
    
    let response = reqwest::get(url)
        .await
        .unwrap();
    
    let bytes = response.bytes().await.unwrap();

    let _ = std::fs::write(path.clone(), &bytes);
}