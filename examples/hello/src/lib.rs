#[macro_use]
extern crate log;

use cstr::cstr;
use we_rt::{init_logger, invoke, callback};
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

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Response {
    bar: i32,
}

#[no_mangle]
extern "C" fn hello(cb: i64, user_data: i64) {
    LOG_INIT.call_once(init_logger);
    let rt = we_rt::Runtime::new();

    rt.spawn(async move {
        info!("log inside wasm");
        let response = Response { bar: 1 };
        callback(&bincode::serialize(&response).unwrap(), cb, user_data);

        let result: Result<Response, _> = invoke("hello", "add_one", Arg { foo: 1 }).await;
        info!("{:?}", result.unwrap());
    })
}
