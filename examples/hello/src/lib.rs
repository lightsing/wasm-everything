#[macro_use]
extern crate log;

use cstr::cstr;
use we_rt::{init_logger, invoke, callback, HostCallback};
use serde::{Deserialize, Serialize};
use std::ffi::{CStr, c_void};
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
extern "C" fn hello(cb: i64, user_data: i64) {
    LOG_INIT.call_once(init_logger);
    let rt = we_rt::Runtime::new();

    rt.spawn(async move {
        info!("log inside wasm");
        let test_string = String::from("hello world");
        callback(test_string.as_bytes(), cb, user_data);

        let result: Result<Response, _> = invoke("hello", "add_one", Arg { foo: 1 }).await;
        info!("{:?}", result.unwrap());
    })
}
