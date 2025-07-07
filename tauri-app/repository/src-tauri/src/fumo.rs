use serde::{Deserialize, Serialize};
use tools_core::{domain::repository::{Repository, Storable}, infra::repository::sqlite::{SqliteRepository as Repo, SqliteRepositoryItem as SqlItem}};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct Item {
    pub uid: String,
    pub sku: String,
    pub metadata: String,
}

impl From<SqlItem> for Item {
    fn from(item: SqlItem) -> Self {
        Item {
            uid: item.id().to_string(),
            sku: item.sku().to_string(),
            metadata: item.metadata().to_string(),
        }
    }
}

#[tauri::command]
pub async fn fumo_load() -> Result<Vec<Item>, String> {
    let repo = Repo::new("./db/fumo.sqlite", "szbdc20250809".to_string()).await;
    repo.create_table().await.map_err(|e| e.to_string())?;
    Ok(repo.list().await.map_err(|e| e.to_string())?
        .into_iter()
        .map(|x| x.into())
        .collect())
}

#[tauri::command]
pub async fn fumo_get_by_uid(uid: String) -> Result<Option<Item>, String> {
    let repo = Repo::new("./db/fumo.sqlite", "szbdc20250809".to_string()).await;
    let uid = Uuid::parse_str(&uid).map_err(|e| e.to_string())?;
    let rs = repo.get_by_id(uid).await.map_err(|e| e.to_string())?
        .map(|x| x.into());
    Ok(rs)
}

#[tauri::command]
pub async fn fumo_get_by_sku(sku: String) -> Result<Option<Item>, String> {
    let repo = Repo::new("./db/fumo.sqlite", "szbdc20250809".to_string()).await;
    let rs = repo.get_by_sku(&sku).await.map_err(|e| e.to_string())?
        .map(|x| x.into());
    Ok(rs)
}

#[tauri::command]
pub async fn fumo_add(sku: String, metadata: String) -> Result<(), String> {
    let repo = Repo::new("./db/fumo.sqlite", "szbdc20250809".to_string()).await;
    let item = SqlItem::new(sku, metadata);
    repo.add(item).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn fumo_remove(uid: String) -> Result<(), String> {
    let repo = Repo::new("./db/fumo.sqlite", "szbdc20250809".to_string()).await;
    let uid = Uuid::parse_str(&uid).map_err(|e| e.to_string())?;
    repo.delete(uid).await.map_err(|e| e.to_string())
}