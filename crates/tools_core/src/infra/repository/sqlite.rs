use std::{fs, path::PathBuf};

use async_trait::async_trait;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::repository::{Repository, Storable};

#[derive(Debug, thiserror::Error)]
pub enum SqliteRepositoryError {
    #[error(transparent)]
    RusqliteError(#[from] rusqlite::Error),
    #[error("Task failed to execute: {0}")]
    JoinError(#[from] tokio::task::JoinError),
    #[error("Table `{0}` does not exist")]
    TableNotFound(String),
    #[error("parsing error: {0}")]
    ParseError(String),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
}

pub struct SqliteRepository {
    db_path: String,
    table_name: String,
    sql_get_by_id: String,
    sql_get_by_sku: String,
    sql_add: String,
    sql_delete: String,
    sql_list: String,
}

impl SqliteRepository {
    pub async fn new(db_path: &str, table_name: String) -> Self {
        let sql_get_by_id = format!("SELECT id, sku, metadata FROM {} WHERE id = ?1", table_name);
        let sql_get_by_sku = format!("SELECT id, sku, metadata FROM {} WHERE sku = ?1", table_name);
        let sql_add = format!("INSERT INTO {} (id, sku, metadata) VALUES (?1, ?2, ?3)", table_name);
        let sql_delete = format!("DELETE FROM {} WHERE id = ?1", table_name);
        let sql_list = format!("SELECT id, sku, metadata FROM {}", table_name);
        let rs = SqliteRepository {
            db_path: db_path.to_string(),
            table_name,
            sql_get_by_id,
            sql_get_by_sku,
            sql_add,
            sql_delete,
            sql_list,
        };
        rs
    }

    pub async fn create_table(&self) -> Result<(), SqliteRepositoryError> {
        let db_path = PathBuf::from(self.db_path.clone());
        let table_name = self.table_name.clone();

        tokio::task::spawn_blocking(move || {
            if let Some(parent_dir) = db_path.as_path().parent() {
                fs::create_dir_all(parent_dir)?
            }
            let conn = Connection::open(db_path)?;
            let sql = format!(
                "CREATE TABLE IF NOT EXISTS {} (id TEXT PRIMARY KEY, sku TEXT NOT NULL UNIQUE, metadata TEXT NOT NULL)",
                table_name
            );
            conn.execute(&sql, [])?;
            Ok(())
        })
        .await?
    }
}

#[async_trait]
impl Repository<SqliteRepositoryItem> for SqliteRepository {
    type RepositoryError = SqliteRepositoryError;

    async fn get_by_id(&self, id: Uuid) -> Result<Option<SqliteRepositoryItem>, Self::RepositoryError> {
        let db_path = self.db_path.clone();
        let sql = self.sql_get_by_id.clone();

        tokio::task::spawn_blocking(move || {
            let conn = Connection::open(db_path)?;
            let mut stmt = conn.prepare(&sql)?;
            let mut rows = stmt.query_map([id.to_string()], |row| {
                Ok(SqliteRepositoryItem {
                    id: Uuid::parse_str(&row.get::<_, String>(0)?).map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?,
                    sku: row.get(1)?,
                    metadata: row.get(2)?,
                })
            })?;

            match rows.next() {
                Some(Ok(item)) => Ok(Some(item)),
                Some(Err(e)) => Err(e.into()),
                None => Ok(None),
            }
        }).await?
    }

    async fn get_by_sku(&self, sku: &<SqliteRepositoryItem as Storable>::Sku) -> Result<Option<SqliteRepositoryItem>, Self::RepositoryError> {
        let db_path = self.db_path.clone();
        let sql = self.sql_get_by_sku.clone();
        let sku = sku.clone();

        tokio::task::spawn_blocking(move || {
            let conn = Connection::open(db_path)?;
            let mut stmt = conn.prepare(&sql)?;

            let mut rows = stmt.query_map([sku], |row| {
                Ok(SqliteRepositoryItem {
                    id: Uuid::parse_str(&row.get::<_, String>(0)?).map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?,
                    sku: row.get(1)?,
                    metadata: row.get(2)?,
                })
            })?;

            match rows.next() {
                Some(Ok(item)) => Ok(Some(item)),
                Some(Err(e)) => Err(e.into()),
                None => Ok(None),
            }
        }).await?
    }

    async fn add(&self, item: SqliteRepositoryItem) -> Result<(), Self::RepositoryError> {
        let db_path = self.db_path.clone();
        let sql = self.sql_add.clone();

        tokio::task::spawn_blocking(move || {
            let conn = Connection::open(db_path)?;
            let mut stmt = conn.prepare(&sql)?;
            stmt.execute((&item.id.to_string(), &item.sku, &item.metadata))?;
            Ok(())
        }).await?
    }

    async fn delete(&self, id: Uuid) -> Result<(), Self::RepositoryError> {
        let db_path = self.db_path.clone();
        let sql = self.sql_delete.clone();

        tokio::task::spawn_blocking(move || {
            let conn = Connection::open(db_path)?;
            let mut stmt = conn.prepare(&sql)?;
            stmt.execute([id.to_string()])?;
            Ok(())
        }).await?
    }

    async fn list(&self) -> Result<Vec<SqliteRepositoryItem>, Self::RepositoryError> {
        let db_path = self.db_path.clone();
        let sql = self.sql_list.clone();

        tokio::task::spawn_blocking(move || {
            let conn = Connection::open(db_path)?;
            let mut stmt = conn.prepare(&sql)?;

            let rows = stmt.query_map([], |row| {
                Ok(SqliteRepositoryItem {
                    id: Uuid::parse_str(&row.get::<_, String>(0)?).map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?,
                    sku: row.get(1)?,
                    metadata: row.get(2)?,
                })
            })?;

            let mut rs = Vec::new();
            for x in rows {
                rs.push(x?);
            }
            Ok(rs)
        }).await?
    }
}

#[derive(Serialize, Deserialize)]
pub struct SqliteRepositoryItem {
    id: Uuid,
    sku: String,
    metadata: String,
}

impl SqliteRepositoryItem {
    pub fn new(sku: String, metadata: String) -> Self {
        let id = Uuid::new_v4();
        Self { id, sku, metadata }
    }
}

impl Storable for SqliteRepositoryItem {
    type Sku = String;
    type Metadata = String;

    fn id(&self) -> Uuid {
        self.id
    }

    fn sku(&self) -> &Self::Sku {
        &self.sku
    }

    fn metadata(&self) -> &Self::Metadata {
        &self.metadata
    }
}
