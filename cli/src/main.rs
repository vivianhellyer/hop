mod input;

use async_std::task;
use hop::Client;
use hop_lib::command::CommandId;
use std::{
    error::Error,
    io::{self, Write},
};

fn main() -> Result<(), Box<dyn Error>> {
    let mut client = Client::memory();
    let stdin = io::stdin();
    let mut stdin = stdin.lock();
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    let mut input = String::new();

    loop {
        write!(stdout, "> ")?;
        stdout.flush()?;
        let req = input::process_command(&mut stdin, &mut input)?;
        input.clear();

        match req.kind() {
            CommandId::Decrement => {
                let key = match req.key() {
                    Some(key) => key,
                    None => {
                        writeln!(stdout, "Key required.")?;

                        continue;
                    }
                };

                let v = task::block_on(client.decrement(key)).unwrap();

                writeln!(stdout, "{}", v)?;
            }
            CommandId::Echo => {
                if let Some(args) = req.flatten_args() {
                    let v = task::block_on(client.echo(args)).unwrap();

                    writeln!(stdout, "{}", String::from_utf8_lossy(&v))?;
                } else {
                    writeln!(stdout,)?;
                }
            }
            CommandId::Increment => {
                let key = match req.key() {
                    Some(key) => key,
                    None => {
                        writeln!(stdout, "Key required.")?;

                        continue;
                    }
                };

                let v = task::block_on(client.increment(key)).unwrap();

                writeln!(stdout, "{}", v)?;
            }
            _ => {}
        }
    }
}
