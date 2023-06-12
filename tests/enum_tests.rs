// some tests for structs
use type2network::{error::Error, FromNetworkOrder, ToNetworkOrder};
use type2network_derive::{FromNetwork, ToNetwork};

#[test]
fn enum_simple() {
    #[derive(ToNetwork)]
    enum Bool {
        True,
        False,
    }

    // impl Bool {
    //     pub fn to_n(&self) {
    //         match self {
    //             Bool::True =>
    //         }
    //     }
    // }
}

#[test]
fn enum_message() {
    #[derive(ToNetwork)]
    enum Message {
        Quit,
        Move { x: i32, y: i32 },
        Write(String),
        ChangeColor(i32, i32, i32),
    }
}
