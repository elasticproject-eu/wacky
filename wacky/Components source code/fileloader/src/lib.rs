#[allow(warnings)]
mod bindings;

use crate::bindings::exports::component::fileloader::fileaccess::Guest;
use bindings::component::trustedwriter::writer::writing_back;
use wasi::filesystem::preopens;
use wasi::filesystem::types::{DescriptorFlags, OpenFlags, PathFlags};

struct Component;

impl Guest for Component {
    fn filnprint() {
        let provided_dir = preopens::get_directories();
        let (descrip, _) = provided_dir.first().ok_or(()).unwrap();
        let fdflags = DescriptorFlags::READ;
        let pflags = PathFlags::empty();
        let oflags = OpenFlags::empty();
        let file_name = "information.txt";

        let file = match descrip.open_at(pflags, file_name, oflags, fdflags) {
            Ok(file) => Some(file),
            Err(e) => {
                println!("Error: {:?}", e);
                None
            }
        };

        let mut the_end = false;
        let mut data_buff: Vec<u8> = Vec::new();
        let mut start = 0;

        if let Some(file) = file {
            while !the_end {
                let (data, stop) = file.read(25, start).unwrap();
                start += data.len() as u64;
                data_buff.extend_from_slice(&data);
                the_end = stop;
            }
        }
        let _ = writing_back(&data_buff);
    }
}

bindings::export!(Component with_types_in bindings);
