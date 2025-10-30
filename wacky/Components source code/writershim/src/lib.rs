#[allow(warnings)]
mod bindings;

//use crate::bindings::exports::shim::writershim::writer::Guest;
use crate::bindings::exports::docs::writetwo::writer::Guest;
use bindings::docs::writetwo::writer::writing_back as write_before_shim;
//use wasi::io::streams::StreamError;
//use bindings::wasi::io::streams::{OutputStream, StreamError};
//use wasi::cli::{stderr, stdout};

struct Component;

impl Guest for Component {
    fn writing_back(info: Vec<u8>) {
        let mut changed_vec = b"\x1b[33m".to_vec();
        changed_vec.extend(info);
        let reset_vec = b"\x1b[0m".to_vec();
        changed_vec.extend(reset_vec);
        write_before_shim(&changed_vec);
    }
}

bindings::export!(Component with_types_in bindings);
