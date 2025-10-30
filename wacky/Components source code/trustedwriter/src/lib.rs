#[allow(warnings)]
mod bindings;

use crate::bindings::exports::component::trustedwriter::writer::Guest;
use wasi::cli::{stderr, stdout};
use wasi::io::streams::StreamError;

struct Component;

impl Guest for Component {
    fn writing_back(info: Vec<u8>) {
        let out_stream = stdout::get_stdout();
        let outs_ref = &out_stream;
        let info_ref = &info;
        let writing_errors = stderr::get_stderr();
        match outs_ref.blocking_write_and_flush(info_ref) {
            Ok(_) => {}
            Err(error) => match error {
                StreamError::Closed => panic!("Output stream is closed.\n"),
                StreamError::LastOperationFailed(specific_error) => {
                    let error_message = specific_error.to_debug_string();
                    writing_errors
                        .blocking_write_and_flush(error_message.as_bytes())
                        .unwrap();
                    panic!("Error dedicated");
                }
            },
        }
    }
}

bindings::export!(Component with_types_in bindings);
