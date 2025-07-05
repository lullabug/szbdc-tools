use std::{error::Error, hash::Hash};

use async_trait::async_trait;
use uuid::Uuid;

pub trait Storable {
    type Sku: Eq + Hash + Clone + Send + Sync;
    type Metadata: Clone + Send + Sync;

    fn id(&self) -> Uuid;
    fn sku(&self) -> &Self::Sku;
    fn metadata(&self) -> &Self::Metadata;
}

#[derive(Clone, Debug)]
pub struct Item<S, M>
where
    S: Eq + Hash + Clone + Send + Sync,
    M: Clone + Send + Sync,
{
    id: Uuid,
    sku: S,
    metadata: M,
}

impl<S, M> Item<S, M>
where
    S: Eq + Hash + Clone + Send + Sync,
    M: Clone + Send + Sync,
{
    pub fn new(sku: S, metadata: M) -> Self {
        let id = Uuid::new_v4();
        Self { id, sku, metadata }
    }
}

impl<S, M> Storable for Item<S, M>
where
    S: Eq + Hash + Clone + Send + Sync,
    M: Clone + Send + Sync,
{
    type Sku = S;
    type Metadata = M;

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

#[async_trait]
pub trait Repository<T: Storable> {
    type RepositoryError: Error + Send + Sync + 'static;

    async fn get_by_id(&self, id: Uuid) -> Result<Option<T>, Self::RepositoryError>;
    async fn get_by_sku(&self, sku: &T::Sku) -> Result<Option<T>, Self::RepositoryError>;
    async fn add(&self, item: T) -> Result<(), Self::RepositoryError>;
    async fn delete(&self, id: Uuid) -> Result<(), Self::RepositoryError>;
    async fn list(&self) -> Result<Vec<T>, Self::RepositoryError>;
}

#[derive(Debug, Clone)]
pub struct ItemRegisterEvent<S, M>
where
    S: Eq + Hash + Clone + Send + Sync,
    M: Clone + Send + Sync,
{
    pub sku: S,
    pub metadata: M,
    pub registered_at: chrono::DateTime<chrono::Utc>,
    pub source: ItemRegisterSource,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemRegisterSource {
    Manual,
    QrCode,
    Api,
    Other(String),
}
