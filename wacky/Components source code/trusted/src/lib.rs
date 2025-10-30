#[allow(warnings)]
mod bindings;

use crate::bindings::exports::wasi::cli::run::Guest;
use bindings::component::fileloader::fileaccess::filnprint;
use bindings::component::trustedwriter::writer::writing_back;
use wasi::cli::stderr;
use wasi::cli::stdin::{self, InputStream};
use wasi::io::streams::StreamError;

struct Component;

impl Guest for Component {
    /// Say hello!
    fn run() -> Result<(), ()> {
        println!();
        println!("Untrusted File Component");
        filnprint();
        println!();
        println!();
        let in_stream = stdin::get_stdin();
        let buff_len = 2048;
        let read_input = reading(&in_stream, buff_len);
        println!();
        println!();
        writing_back(&read_input);
        println!();
        println!();
        Ok(())
    }
}

fn reading(x: &InputStream, y: u64) -> Vec<u8> {
    println!("Trusted Terminal Component");
    println!(">>Enter Content to be echoed through terminal:");
    let reading_errors = stderr::get_stderr();
    match x.blocking_read(y) {
        Ok(read_data) => read_data,
        Err(error) => match error {
            StreamError::Closed => panic!("Input stream is closed.\n"),
            StreamError::LastOperationFailed(specific_error) => {
                let error_message = specific_error.to_debug_string();
                reading_errors
                    .blocking_write_and_flush(error_message.as_bytes())
                    .unwrap();
                panic!("Error dedicated");
            }
        },
    }
}
bindings::export!(Component with_types_in bindings);
