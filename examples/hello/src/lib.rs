use rt_interface::invoke;
use serde::{Serialize, Deserialize};

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
    let test_string = String::from("hello world");
    let result: Result<Response, _> = invoke(
        "hello",
        "add_one",
        vec![Arg { foo: 1 }]
    );
    result.unwrap();
}