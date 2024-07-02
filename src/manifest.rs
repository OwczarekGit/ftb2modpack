use std::{
    io::{BufWriter, Write},
    path::PathBuf,
};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    pub minecraft: Minecraft,
    pub manifest_type: String,
    pub manifest_version: i64,
    pub name: String,
    pub version: String,
    pub author: String,
    pub files: Vec<File>,
    pub overrides: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Minecraft {
    pub version: String,
    pub mod_loaders: Vec<ModLoaders>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ModLoaders {
    pub id: String,
    pub primary: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct File {
    #[serde(rename(serialize = "projectID"))]
    pub project_id: i64,
    #[serde(rename(serialize = "fileID"))]
    pub file_id: i64,
    pub required: bool,
}

impl TryFrom<crate::ftb_pack::Pack> for Manifest {
    type Error = ();

    fn try_from(v: crate::ftb_pack::Pack) -> Result<Self, Self::Error> {
        let files = v
            .files
            .iter()
            .filter(|file| file.r#type.eq("mod") && file.curseforge.is_some())
            .map(|file| {
                let cf = file.curseforge.as_ref().unwrap();
                File {
                    project_id: cf.project,
                    file_id: cf.file,
                    required: true,
                }
            })
            .collect::<Vec<_>>();

        let version = v
            .targets
            .iter()
            .find(|target| target.r#type.eq("game") && target.name.eq("minecraft"))
            .map(|target| target.version.clone())
            .unwrap_or("unknown".to_string());

        let mod_loaders = v
            .targets
            .iter()
            .filter(|target| target.r#type.eq("modloader"))
            .map(|target| ModLoaders {
                id: format!("{}-{}", target.name, target.version),
                primary: true,
            })
            .collect::<Vec<_>>();

        Ok(Self {
            files,
            author: "FTB2Pack".to_string(),
            manifest_type: "minecraftModpack".to_string(),
            manifest_version: 1,
            name: "Modpack".to_string(),
            version: v.name,
            minecraft: Minecraft {
                version,
                mod_loaders,
            },
            overrides: "overrides".to_string(),
        })
    }
}

pub fn save_manifest(base: PathBuf, manifest: Manifest) {
    let mut path = base.clone();
    path.push("manifest.json");
    let file = std::fs::File::create(path).expect("File to be created");
    let mut writer = BufWriter::new(file);
    let _ = serde_json::to_writer_pretty(&mut writer, &manifest);
    let _ = writer.flush();
}
