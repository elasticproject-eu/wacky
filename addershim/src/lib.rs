#[allow(warnings)]
mod bindings;

use crate::bindings::exports::shim::addershim::add::Guest;
use bindings::docs::adder::add::{add as add_before_shim  ,subtract as subtract_before_shim  ,addresult as addresult_before_shim  ,addlist as addlist_before_shim  ,addoption as addoption_before_shim  ,sumtuple as sumtuple_before_shim};

struct Component;

impl Guest for Component {

    fn add(x: u32,y: u32) -> u32 {
        add_before_shim(x,y)
    }

 

    fn subtract(pe: u32,ee: u32) -> u8 {
        subtract_before_shim(pe,ee)
    }

 

    fn addresult(yx: u32,y: u32) -> Result<u32, f32> {
        addresult_before_shim(yx,y)
    }

 

    fn addlist(m: u16) -> Vec<u32> {
        addlist_before_shim(m)
    }

 

    fn addoption(lo: Option<u8>) -> Option<u32> {
        addoption_before_shim(lo)
    }

 

    fn sumtuple(pair: (u8,f64)) -> char {
        sumtuple_before_shim(pair)
    }

}

bindings::export!(Component with_types_in bindings);