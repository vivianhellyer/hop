use super::{
    super::{Dispatch, DispatchError, DispatchResult, Request},
    increment_by::IncrementBy,
};
use crate::Hop;
use alloc::vec::Vec;

pub struct Increment;

impl Dispatch for Increment {
    fn dispatch(hop: &Hop, req: &Request) -> DispatchResult<Vec<u8>> {
        let key = req.key().ok_or(DispatchError::KeyRetrieval)?;

        IncrementBy::increment(hop, key, req.key_type(), 1)
    }
}