use crate::{
    command::{response, Dispatch, DispatchError, DispatchResult, Request},
    state::{KeyType, Value},
    Hop,
};
use alloc::{borrow::ToOwned, vec::Vec};

pub struct Set;

impl Set {
    fn boolean(hop: &Hop, req: &Request, resp: &mut Vec<u8>, key: &[u8]) -> DispatchResult<()> {
        let arg = req.typed_arg(1).ok_or(DispatchError::ArgumentRetrieval)?;
        hop.state().remove(key);
        let mut key = hop.state().key_or_insert_with(key, Value::boolean);
        let boolean = key
            .as_boolean_mut()
            .ok_or(DispatchError::KeyTypeDifferent)?;

        *boolean = arg;

        response::write_bool(resp, arg);

        Ok(())
    }

    fn bytes(hop: &Hop, req: &Request, resp: &mut Vec<u8>, key: &[u8]) -> DispatchResult<()> {
        let arg = req
            .typed_arg::<&[u8]>(1)
            .ok_or(DispatchError::ArgumentRetrieval)?;
        hop.state().remove(key);
        let mut key = hop.state().key_or_insert_with(key, Value::bytes);
        let bytes = key.as_bytes_mut().ok_or(DispatchError::KeyTypeDifferent)?;

        *bytes = arg.to_vec();

        response::write_bytes(resp, arg);

        Ok(())
    }

    fn float(hop: &Hop, req: &Request, resp: &mut Vec<u8>, key: &[u8]) -> DispatchResult<()> {
        let arg = req.typed_arg(1).ok_or(DispatchError::ArgumentRetrieval)?;
        hop.state().remove(key);
        let mut key = hop.state().key_or_insert_with(key, Value::float);
        let float = key.as_float_mut().ok_or(DispatchError::KeyTypeDifferent)?;

        *float = arg;

        response::write_float(resp, arg);

        Ok(())
    }

    fn integer(hop: &Hop, req: &Request, resp: &mut Vec<u8>, key: &[u8]) -> DispatchResult<()> {
        let arg = req.typed_arg(1).ok_or(DispatchError::ArgumentRetrieval)?;
        hop.state().remove(key);
        let mut key = hop.state().key_or_insert_with(key, Value::integer);
        let int = key
            .as_integer_mut()
            .ok_or(DispatchError::KeyTypeDifferent)?;

        *int = arg;

        response::write_int(resp, arg);

        Ok(())
    }

    fn list(hop: &Hop, req: &Request, resp: &mut Vec<u8>, key: &[u8]) -> DispatchResult<()> {
        let args = req.args(1..).ok_or(DispatchError::ArgumentRetrieval)?;
        hop.state().remove(key);
        let mut key = hop.state().key_or_insert_with(key, Value::list);
        let list = key.as_list_mut().ok_or(DispatchError::KeyTypeDifferent)?;

        *list = args.map(ToOwned::to_owned).collect();
        let args = req.args(1..).ok_or(DispatchError::ArgumentRetrieval)?;

        response::write_list(resp, args);

        Ok(())
    }

    fn map(hop: &Hop, req: &Request, resp: &mut Vec<u8>, key: &[u8]) -> DispatchResult<()> {
        let args = req.typed_args().ok_or(DispatchError::ArgumentRetrieval)?;
        hop.state().remove(key);
        let mut key = hop.state().key_or_insert_with(key, Value::map);
        let map = key.as_map_mut().ok_or(DispatchError::KeyTypeDifferent)?;

        response::write_map(resp, &args);

        *map = args;

        Ok(())
    }

    fn set(hop: &Hop, req: &Request, resp: &mut Vec<u8>, key: &[u8]) -> DispatchResult<()> {
        let args = req.typed_args().ok_or(DispatchError::ArgumentRetrieval)?;
        hop.state().remove(key);
        let mut key = hop.state().key_or_insert_with(key, Value::set);
        let set = key.as_set_mut().ok_or(DispatchError::KeyTypeDifferent)?;

        response::write_set(resp, &args);

        *set = args;

        Ok(())
    }

    fn string(hop: &Hop, req: &Request, resp: &mut Vec<u8>, key: &[u8]) -> DispatchResult<()> {
        let arg = req
            .typed_arg::<&str>(1)
            .ok_or(DispatchError::ArgumentRetrieval)?;
        hop.state().remove(key);
        let mut key = hop.state().key_or_insert_with(key, Value::string);
        let string = key.as_string_mut().ok_or(DispatchError::KeyTypeDifferent)?;

        *string = arg.to_owned();

        response::write_str(resp, arg);

        Ok(())
    }
}

impl Dispatch for Set {
    fn dispatch(hop: &Hop, req: &Request, resp: &mut Vec<u8>) -> DispatchResult<()> {
        let key = req.key().ok_or(DispatchError::KeyUnspecified)?;

        // All types require at least one argument, so let's do that check here.
        if req.arg(1).is_none() {
            return Err(DispatchError::ArgumentRetrieval);
        }

        let key_type = req
            .key_type()
            .or_else(|| hop.state().key_type(key))
            .unwrap_or(KeyType::Bytes);

        match key_type {
            KeyType::Bytes => Self::bytes(hop, req, resp, key),
            KeyType::Boolean => Self::boolean(hop, req, resp, key),
            KeyType::Float => Self::float(hop, req, resp, key),
            KeyType::Integer => Self::integer(hop, req, resp, key),
            KeyType::List => Self::list(hop, req, resp, key),
            KeyType::Map => Self::map(hop, req, resp, key),
            KeyType::Set => Self::set(hop, req, resp, key),
            KeyType::String => Self::string(hop, req, resp, key),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Set;
    use crate::{
        command::{request::RequestBuilder, CommandId, Dispatch, DispatchError, Response},
        state::{KeyType, Value},
        Hop,
    };
    use alloc::vec::Vec;

    #[test]
    fn test_types_no_arg() {
        let hop = Hop::new();
        let mut resp = Vec::new();

        let types = [
            KeyType::Boolean,
            KeyType::Bytes,
            KeyType::Float,
            KeyType::Integer,
            KeyType::List,
            KeyType::Map,
            KeyType::Set,
            KeyType::String,
        ];

        for key_type in &types {
            let mut builder = RequestBuilder::new_with_key_type(CommandId::Set, *key_type);
            assert!(builder.bytes(b"foo".as_ref()).is_ok());
            let req = builder.into_request();

            assert_eq!(
                Set::dispatch(&hop, &req, &mut resp).unwrap_err(),
                DispatchError::ArgumentRetrieval,
            );

            resp.clear();
        }
    }

    #[test]
    fn test_bool() {
        let mut builder = RequestBuilder::new_with_key_type(CommandId::Set, KeyType::Boolean);
        assert!(builder.bytes(b"foo".as_ref()).is_ok());
        assert!(builder.bytes([1].as_ref()).is_ok());
        let req = builder.into_request();

        let hop = Hop::new();

        let mut resp = Vec::new();

        assert!(Set::dispatch(&hop, &req, &mut resp).is_ok());
        assert_eq!(resp, Response::from(true).as_bytes());
        assert_eq!(
            Some(&true),
            hop.state()
                .key_ref(b"foo")
                .as_deref()
                .and_then(Value::as_boolean_ref)
        );
    }

    #[test]
    fn test_bytes() {
        let mut builder = RequestBuilder::new_with_key_type(CommandId::Set, KeyType::Bytes);
        assert!(builder.bytes(b"foo".as_ref()).is_ok());
        assert!(builder.bytes(b"bar baz".to_vec()).is_ok());
        let req = builder.into_request();

        let hop = Hop::new();

        let mut resp = Vec::new();

        assert!(Set::dispatch(&hop, &req, &mut resp).is_ok());
        assert_eq!(resp, Response::from(b"bar baz".to_vec()).as_bytes());
        assert_eq!(
            Some(b"bar baz".as_ref()),
            hop.state()
                .key_ref(b"foo")
                .as_deref()
                .and_then(Value::as_bytes_ref)
        );
    }

    #[test]
    fn test_float() {
        let mut builder = RequestBuilder::new_with_key_type(CommandId::Set, KeyType::Float);
        assert!(builder.bytes(b"foo".as_ref()).is_ok());
        assert!(builder.bytes(2f64.to_be_bytes().to_vec()).is_ok());
        let req = builder.into_request();

        let hop = Hop::new();

        let mut resp = Vec::new();

        assert!(Set::dispatch(&hop, &req, &mut resp).is_ok());
        assert_eq!(resp, Response::from(2f64).as_bytes());
        assert_eq!(
            Some(&2f64),
            hop.state()
                .key_ref(b"foo")
                .as_deref()
                .and_then(Value::as_float_ref)
        );
    }

    #[test]
    fn test_int() {
        let mut builder = RequestBuilder::new_with_key_type(CommandId::Set, KeyType::Integer);
        assert!(builder.bytes(b"foo".as_ref()).is_ok());
        assert!(builder.bytes(2i64.to_be_bytes().to_vec()).is_ok());
        let req = builder.into_request();

        let hop = Hop::new();

        let mut resp = Vec::new();

        assert!(Set::dispatch(&hop, &req, &mut resp).is_ok());
        assert_eq!(resp, Response::from(2i64).as_bytes());
        assert_eq!(
            Some(&2),
            hop.state()
                .key_ref(b"foo")
                .as_deref()
                .and_then(Value::as_integer_ref),
        );
    }

    #[test]
    fn test_list_three_entries() {
        let mut builder = RequestBuilder::new_with_key_type(CommandId::Set, KeyType::List);
        assert!(builder.bytes(b"foo".as_ref()).is_ok());
        assert!(builder.bytes(b"value1".to_vec()).is_ok());
        assert!(builder.bytes(b"value2".to_vec()).is_ok());
        assert!(builder.bytes(b"value2".to_vec()).is_ok());
        let req = builder.into_request();

        let hop = Hop::new();

        let mut resp = Vec::new();

        assert!(Set::dispatch(&hop, &req, &mut resp).is_ok());
        assert_eq!(
            Some(3),
            hop.state()
                .key_ref(b"foo")
                .as_deref()
                .and_then(Value::as_list_ref)
                .map(|list| list.len()),
        );
    }

    #[test]
    fn test_map_two_entries() {
        let mut builder = RequestBuilder::new_with_key_type(CommandId::Set, KeyType::Map);
        assert!(builder.bytes(b"foo".as_ref()).is_ok());
        assert!(builder.bytes(b"key1".to_vec()).is_ok());
        assert!(builder.bytes(b"value1".to_vec()).is_ok());
        assert!(builder.bytes(b"key2".to_vec()).is_ok());
        assert!(builder.bytes(b"value2".to_vec()).is_ok());
        let req = builder.into_request();

        let hop = Hop::new();

        let mut resp = Vec::new();

        assert!(Set::dispatch(&hop, &req, &mut resp).is_ok());
        assert_eq!(
            Some(2),
            hop.state()
                .key_ref(b"foo")
                .as_deref()
                .and_then(Value::as_map_ref)
                .map(|map| map.len()),
        );
    }

    #[test]
    fn test_set_two_entries() {
        let mut builder = RequestBuilder::new_with_key_type(CommandId::Set, KeyType::Set);
        assert!(builder.bytes(b"foo".as_ref()).is_ok());
        assert!(builder.bytes(b"value1".to_vec()).is_ok());
        assert!(builder.bytes(b"value2".to_vec()).is_ok());
        let req = builder.into_request();

        let hop = Hop::new();

        let mut resp = Vec::new();

        assert!(Set::dispatch(&hop, &req, &mut resp).is_ok());
        assert_eq!(
            Some(2),
            hop.state()
                .key_ref(b"foo")
                .as_deref()
                .and_then(Value::as_set_ref)
                .map(|set| set.len()),
        );
    }

    #[test]
    fn test_str() {
        let mut builder = RequestBuilder::new_with_key_type(CommandId::Set, KeyType::String);
        assert!(builder.bytes(b"foo".as_ref()).is_ok());
        assert!(builder.bytes(b"bar".to_vec()).is_ok());
        let req = builder.into_request();

        let hop = Hop::new();

        let mut resp = Vec::new();

        assert!(Set::dispatch(&hop, &req, &mut resp).is_ok());
        assert_eq!(resp, Response::from("bar".to_owned()).as_bytes());
        assert_eq!(
            Some("bar"),
            hop.state()
                .key_ref(b"foo")
                .as_deref()
                .and_then(Value::as_string_ref)
        );
    }
}
