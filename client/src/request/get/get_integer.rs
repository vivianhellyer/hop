use super::super::MaybeInFlightFuture;
use crate::Backend;
use alloc::{boxed::Box, sync::Arc};
use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use hop_engine::state::Value;

/// A configured `get` command that will resolve to an integer when `await`ed.
///
/// This is returned by [`GetUnconfigured::int`].
///
/// [`GetUnconfigured::int`]: struct.GetUnconfigured.html#method.int
pub struct GetInteger<'a, B: Backend, K: AsRef<[u8]> + 'a + Send + Unpin> {
    backend: Option<Arc<B>>,
    fut: MaybeInFlightFuture<'a, i64, B::Error>,
    key: Option<K>,
}

impl<'a, B: Backend, K: AsRef<[u8]> + 'a + Send + Unpin> GetInteger<'a, B, K> {
    pub(crate) fn new(backend: Arc<B>, key: K) -> Self {
        Self {
            backend: Some(backend),
            fut: None,
            key: Some(key),
        }
    }
}

impl<'a, B: Backend + Send + Sync + 'static, K: AsRef<[u8]> + Send + Unpin> Future
    for GetInteger<'a, B, K>
{
    type Output = Result<i64, B::Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.fut.is_none() {
            let backend = self.backend.take().expect("backend only taken once");
            let key = self.key.take().expect("key only taken once");

            self.fut.replace(Box::pin(async move {
                let key = key.as_ref();
                let value = backend.get(key).await?;

                match value {
                    Value::Integer(int) => Ok(int),
                    _ => unreachable!(),
                }
            }));
        }

        self.fut.as_mut().expect("future exists").as_mut().poll(cx)
    }
}

#[cfg(test)]
mod tests {
    use super::GetInteger;
    use crate::backend::MemoryBackend;
    use alloc::vec::Vec;
    use static_assertions::assert_impl_all;

    assert_impl_all!(GetInteger<MemoryBackend, Vec<u8>>: Send);
}
