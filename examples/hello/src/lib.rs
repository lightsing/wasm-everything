#[macro_use]
extern crate log;

use cstr::cstr;
use we_rt::{init_logger, invoke};
use serde::{Deserialize, Serialize};
use std::ffi::CStr;
use std::sync::Once;

#[no_mangle]
pub static NAME: &CStr = cstr!(b"hello");

static LOG_INIT: Once = Once::new();

#[derive(Serialize, Clone, Debug)]
struct Arg {
    foo: i32,
}

#[derive(Deserialize, Clone, Debug)]
struct Response {
    bar: i32,
}

#[no_mangle]
extern "C" fn hello() {
    LOG_INIT.call_once(init_logger);
    let rt = we_rt::Runtime::new();

    rt.spawn(async move {
        info!("log inside wasm");
        let _test_string = String::from("hello world");
        let result: Result<Response, _> = invoke("hello", "add_one", Arg { foo: 1 }).await;
        info!("{:?}", result.unwrap());
    })
}
