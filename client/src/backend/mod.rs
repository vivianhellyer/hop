pub mod memory;
pub mod server;

pub use self::{memory::MemoryBackend, server::ServerBackend};

use crate::model::StatsData;
use async_trait::async_trait;
use hop_engine::state::KeyType;

#[async_trait]
pub trait Backend {
    type Error;

    async fn decrement(&self, key: &[u8], key_type: Option<KeyType>) -> Result<i64, Self::Error>;

    async fn delete(&self, key: &[u8]) -> Result<Vec<u8>, Self::Error>;

    async fn echo(&self, content: &[u8]) -> Result<Vec<u8>, Self::Error>;

    async fn exists<T: IntoIterator<Item = U> + Send, U: AsRef<[u8]> + Send>(
        &self,
        keys: T,
    ) -> Result<bool, Self::Error>;

    async fn increment(&self, key: &[u8], key_type: Option<KeyType>) -> Result<i64, Self::Error>;

    async fn rename(&self, from: &[u8], to: &[u8]) -> Result<Vec<u8>, Self::Error>;

    async fn stats(&self) -> Result<StatsData, Self::Error>;
}
