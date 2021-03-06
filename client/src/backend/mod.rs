pub mod memory;

#[cfg(all(not(target_arch = "wasm32"), feature = "tokio"))]
pub mod server;

pub use self::memory::MemoryBackend;

#[cfg(all(not(target_arch = "wasm32"), feature = "tokio"))]
pub use self::server::ServerBackend;

use crate::model::StatsData;
use alloc::{boxed::Box, vec::Vec};
use async_trait::async_trait;
use hop_engine::state::{KeyType, Value};

#[async_trait]
pub trait Backend: Send + Sync {
    type Error;

    async fn append<T: Into<Value> + Send>(
        &self,
        key: &[u8],
        value: T,
    ) -> Result<Value, Self::Error>
    where
        Self: Sized;

    async fn decrement_by<T: Into<Value> + Send>(
        &self,
        key: &[u8],
        value: T,
    ) -> Result<Value, Self::Error>
    where
        Self: Sized;

    async fn decrement(&self, key: &[u8], key_type: Option<KeyType>) -> Result<Value, Self::Error>
    where
        Self: Sized;

    async fn delete(&self, key: &[u8]) -> Result<Vec<u8>, Self::Error>
    where
        Self: Sized;

    async fn echo(&self, content: &[u8]) -> Result<Vec<Vec<u8>>, Self::Error>
    where
        Self: Sized;

    async fn exists<T: IntoIterator<Item = U> + Send, U: AsRef<[u8]> + Send>(
        &self,
        keys: T,
    ) -> Result<bool, Self::Error>
    where
        Self: Sized;

    async fn get(&self, key: &[u8]) -> Result<Value, Self::Error>
    where
        Self: Sized;

    async fn increment_by<T: Into<Value> + Send>(
        &self,
        key: &[u8],
        value: T,
    ) -> Result<Value, Self::Error>
    where
        Self: Sized;

    async fn increment(&self, key: &[u8], key_type: Option<KeyType>) -> Result<Value, Self::Error>
    where
        Self: Sized;

    async fn is<T: IntoIterator<Item = U> + Send, U: AsRef<[u8]> + Send>(
        &self,
        key_type: KeyType,
        keys: T,
    ) -> Result<bool, Self::Error>
    where
        Self: Sized;

    async fn key_type(&self, key: &[u8]) -> Result<KeyType, Self::Error>
    where
        Self: Sized;

    async fn keys(&self, key: &[u8]) -> Result<Vec<Vec<u8>>, Self::Error>
    where
        Self: Sized;

    async fn length(&self, key: &[u8], key_type: Option<KeyType>) -> Result<i64, Self::Error>
    where
        Self: Sized;

    async fn rename(&self, from: &[u8], to: &[u8]) -> Result<Vec<u8>, Self::Error>
    where
        Self: Sized;

    async fn set<T: Into<Value> + Send>(&self, key: &[u8], value: T) -> Result<Value, Self::Error>
    where
        Self: Sized;

    async fn stats(&self) -> Result<StatsData, Self::Error>
    where
        Self: Sized;
}
