use super::{super::ContextConclusion, Request};
use crate::{command::CommandId, pool::Pool, state::KeyType};
use alloc::vec::Vec;
use core::convert::{TryFrom, TryInto};

type Conclusion = ContextConclusion<Request>;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum ParseError {
    CommandIdInvalid = 0,
    KeyTypeInvalid = 1,
}

impl TryFrom<u8> for ParseError {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::CommandIdInvalid,
            1 => Self::KeyTypeInvalid,
            _ => return Err(()),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum Stage {
    Init,
    Kind {
        cmd_type: CommandId,
        key_type: Option<KeyType>,
    },
    ArgumentParsing {
        argument_count: u8,
        cmd_type: CommandId,
        key_type: Option<KeyType>,
    },
}

impl Default for Stage {
    fn default() -> Self {
        Stage::Init
    }
}

#[derive(Debug)]
pub struct Context {
    argument_pool: Pool<Vec<u8>>,
    buf_args: Option<Vec<Vec<u8>>>,
    idx: usize,
    stage: Stage,
}

impl Context {
    const ARG_LEN_BYTES: usize = 4;

    pub fn new() -> Self {
        Default::default()
    }

    pub fn feed(&mut self, buf: &[u8]) -> Result<Option<Request>, ParseError> {
        loop {
            // We need to do this check on the first iteration to make sure we
            // were actually given *any* data, and after each iteration to make
            // sure that there's more data to process.
            if buf.get(self.idx..).is_none() {
                return Ok(None);
            }

            let conclusion = match self.stage {
                Stage::Init => self.stage_init(buf)?,
                Stage::Kind { cmd_type, key_type } => self.stage_kind(buf, key_type, cmd_type)?,
                Stage::ArgumentParsing {
                    argument_count,
                    cmd_type,
                    key_type,
                } => self.stage_argument_parsing(buf, cmd_type, key_type, argument_count)?,
            };

            match conclusion {
                Conclusion::Finished(command_info) => return Ok(Some(command_info)),
                Conclusion::Incomplete => return Ok(None),
                Conclusion::Next => continue,
            }
        }
    }

    pub fn reset(&mut self, mut args: Vec<Vec<u8>>) {
        self.reset_light();
        self.idx = 0;

        for mut vec in args.drain(..) {
            vec.clear();

            self.argument_pool.push(vec);
        }

        self.buf_args.replace(args);
    }

    fn stage_init(&mut self, buf: &[u8]) -> Result<Conclusion, ParseError> {
        let byte = match buf.first() {
            Some(byte) => *byte,
            None => return Ok(Conclusion::Incomplete),
        };

        // If the first bit is flipped, then this byte is denoting the type of
        // key to work with. This means that byte idx 2 is the argument length.
        //
        // If the first bit is 0, then this byte is the argument length, and the
        // type of key to work with is not a requirement.
        let key_type = if byte >> 7 == 1 {
            let key_type_id = byte >> 1;

            Some(KeyType::try_from(key_type_id).map_err(|_| ParseError::KeyTypeInvalid)?)
        } else {
            None
        };

        let cmd_type = CommandId::try_from(byte).map_err(|_| ParseError::CommandIdInvalid)?;

        // If the command type is simple and has no arguments or keys, then
        // we can just return a successful command here.
        if cmd_type.is_simple() {
            self.reset_light();

            return Ok(Conclusion::Finished(Request {
                args: None,
                key_type: None,
                kind: cmd_type,
            }));
        }

        self.stage = Stage::Kind { cmd_type, key_type };
        self.idx = self.idx.wrapping_add(1);

        Ok(Conclusion::Next)
    }

    fn stage_kind(
        &mut self,
        buf: &[u8],
        key_type: Option<KeyType>,
        cmd_type: CommandId,
    ) -> Result<Conclusion, ParseError> {
        let argument_count = match buf.get(self.idx) {
            Some(argument_count) => *argument_count,
            None => return Ok(Conclusion::Incomplete),
        };

        self.stage = Stage::ArgumentParsing {
            argument_count,
            cmd_type,
            key_type,
        };
        self.idx = self.idx.saturating_add(1);

        Ok(Conclusion::Next)
    }

    fn stage_argument_parsing(
        &mut self,
        buf: &[u8],
        cmd_type: CommandId,
        key_type: Option<KeyType>,
        argument_count: u8,
    ) -> Result<Conclusion, ParseError> {
        let len_bytes = match buf.get(self.idx..self.idx + Self::ARG_LEN_BYTES) {
            Some(bytes) => bytes.try_into().unwrap(),
            None => return Ok(Conclusion::Incomplete),
        };

        let arg_len = u32::from_be_bytes(len_bytes) as usize;

        match buf.get(self.idx..self.idx + arg_len) {
            Some(arg) => {
                let mut pooled_arg = self.argument_pool.pull();
                pooled_arg.extend_from_slice(arg);
                self.push_arg(pooled_arg);
            }
            None => return Ok(Conclusion::Incomplete),
        };

        self.idx += 4 + arg_len;

        if self.arg_count() == argument_count as usize {
            let args = self.buf_args.take();

            Ok(Conclusion::Finished(Request {
                args,
                key_type,
                kind: cmd_type,
            }))
        } else {
            Ok(Conclusion::Next)
        }
    }

    fn arg_count(&mut self) -> usize {
        if let Some(args) = self.buf_args.as_ref() {
            args.len()
        } else {
            #[cfg(feature = "log")]
            log::warn!("Got into a weird state! Args don't exist to count, fixing");

            self.buf_args.replace(Vec::new());

            0
        }
    }

    fn push_arg(&mut self, arg: Vec<u8>) {
        match self.buf_args.as_mut() {
            Some(args) => args.push(arg),
            None => {
                #[cfg(feature = "log")]
                log::warn!("Got into a weird state! Args don't exist to push to, fixing");

                let mut args = Vec::new();
                args.push(arg);

                self.buf_args.replace(args);
            }
        }
    }

    fn reset_light(&mut self) {
        self.stage = Stage::default();
    }
}

impl Default for Context {
    fn default() -> Self {
        Self {
            argument_pool: Pool::new(Vec::new),
            buf_args: Some(Vec::new()),
            idx: 0,
            stage: Stage::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        super::{super::error::Result, CommandId},
        Context, ParseError,
    };
    use core::convert::TryFrom;

    #[test]
    fn test_increment_foo() -> Result<()> {
        // - the first byte is the command type's ID (up to ID 255)
        // - the second byte, if the command type has arguments, is the number of
        //   arguments (up to 255)
        // - for each argument, the first 4 bytes is the length of the argument
        //   in bytes (so, each argument can theoretically be up to 4032MiB in
        //   size)
        // - the rest of the bytes, up to the defined length of the argument,
        //   is the argument
        let mut cmd = [
            // command type 0 is "increment int"
            0, // there is 1 argument
            1, // the argument has a length of 3 bytes
            0, 0, 0, 3, // the argument is 'foo'
            b'f', b'o', b'o',
        ]
        .to_vec();

        // the context might not be given all of the data upfront (eg large
        // streams of data, just because of how tcp works, etc.), so often you
        // need to feed multiple rounds of data into it to get a complete
        // command request
        let mut ctx = Context::new();
        // but here we're feeding in all the data in one go
        let cmd = ctx
            .feed(&mut cmd)
            .expect("parses correctly")
            .expect("returns a command");

        assert_eq!(cmd.kind, CommandId::Increment);

        Ok(())
    }

    #[test]
    fn test_parse_error_try_from_u8() {
        assert_eq!(
            ParseError::try_from(0).unwrap(),
            ParseError::CommandIdInvalid
        );
        assert_eq!(ParseError::try_from(1).unwrap(), ParseError::KeyTypeInvalid);
    }
}
