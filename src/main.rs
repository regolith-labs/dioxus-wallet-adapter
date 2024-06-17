#![allow(non_snake_case)]

use dioxus::prelude::*;
use tracing::Level;

fn main() {
    // Init logger
    dioxus_logger::init(Level::INFO).expect("failed to init logger");
    launch(App);
}

#[component]
fn App() -> Element {
    rsx!(
        Nav {}
        React {}
    )
}

#[component]
fn Nav() -> Element {
    rsx!(nav { id: "plsss" })
}

#[component]
fn React() -> Element {
    // Create a future that will resolve once the javascript has been successfully executed.
    let future = use_resource(move || async move {
        // The `eval` is available in the prelude - and simply takes a block of JS.
        // Dioxus' eval is interesting since it allows sending messages to and from the JS code using the `await dioxus.recv()`
        // builtin function. This allows you to create a two-way communication channel between Rust and JS.

        let mut eval_1 = eval(
            r#"
                let msg = await dioxus.recv();
                console.log(msg);
                const rrr = window.PlsRender;
                console.log(rrr);
                rrr();
                return
            "#,
        );

        // Send a message to the JS code.
        eval_1.send("Hi from Rust!".into()).unwrap();

        // Our line on the JS side will log the message and then return "hello world".
        let res = eval_1.recv().await.unwrap();

        // This will print "Hi from JS!" and "Hi from Rust!".
        let _res_1 = eval_1.await;

        res
    });

    future.value().as_ref();
    rsx!(div {})
}
