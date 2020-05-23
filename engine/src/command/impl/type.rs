use crate::{
    command::{response, Dispatch, DispatchError, DispatchResult, Request},
    Hop,
};
use alloc::vec::Vec;

pub struct Type;

impl Dispatch for Type {
    fn dispatch(hop: &Hop, req: &Request, resp: &mut Vec<u8>) -> DispatchResult<()> {
        let key = req.key().ok_or(DispatchError::KeyUnspecified)?;

        if req.key_type().is_some() {
            return Err(DispatchError::KeyTypeUnexpected);
        }

        let key = hop
            .state()
            .key_ref(key)
            .ok_or_else(|| DispatchError::KeyNonexistent)?;

        response::write_int(resp, key.kind() as u8 as i64);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Type;
    use crate::{
        command::{CommandId, Dispatch, DispatchError, Request, Response},
        state::{KeyType, Value},
        Hop,
    };
    use alloc::vec::Vec;

    #[test]
    fn test_key() {
        let hop = Hop::new();
        let mut args = Vec::new();
        args.push(b"foo".to_vec());
        args.push([1].to_vec());
        let req = Request::new(CommandId::Type, Some(args));
        let mut resp = Vec::new();

        hop.state().insert(b"foo".to_vec(), Value::Boolean(true));

        assert!(Type::dispatch(&hop, &req, &mut resp).is_ok());
        assert_eq!(resp, Response::from(KeyType::Boolean as i64).as_bytes());
    }

    #[test]
    fn test_no_key() {
        let hop = Hop::new();
        let req = Request::new(CommandId::Type, None);
        let mut resp = Vec::new();

        assert_eq!(
            DispatchError::KeyUnspecified,
            Type::dispatch(&hop, &req, &mut resp).unwrap_err()
        );
    }

    #[test]
    fn test_key_type_specified() {
        let hop = Hop::new();
        let req = Request::new(CommandId::Type, None);
        let mut resp = Vec::new();

        assert_eq!(
            DispatchError::KeyUnspecified,
            Type::dispatch(&hop, &req, &mut resp).unwrap_err()
        );
    }
}
