#![allow(non_snake_case)]

use dioxus::prelude::*;
use tracing::Level;

use crate::hooks::use_wallet_adapter;

mod hooks;

fn main() {
    // Init logger
    dioxus_logger::init(Level::INFO).expect("failed to init logger");
    launch(App);
}

#[component]
fn App() -> Element {
    use_wallet_adapter::use_wallet_adapter_provider();
    rsx! {
        MountWalletAdapter {}
        RenderBalance {}
        RenderSlot {}
    }
}

#[component]
fn MountWalletAdapter() -> Element {
    let _ = use_future(move || async move {
        let eval = eval(
            r#"
                let mount = window.MountWalletAdapter;
                console.log(mount);
                mount();
                return
            "#,
        );
        let _ = eval.await;
    });
    rsx!(nav {
        id: "dioxus-wallet-adapter"
    })
}

#[component]
fn RenderSlot() -> Element {
    let slot = use_wallet_adapter::use_slot();
    let e = match *slot.read() {
        Some(Some(s)) => {
            rsx! {
                div {
                    "slot: {s}"
                }
            }
        }
        Some(None) => {
            rsx! {
                div {
                    "no slot found"
                }
            }
        }
        None => {
            rsx! {
                div {
                    "waiting for slot"
                }
            }
        }
    };
    e
}

#[component]
fn RenderBalance() -> Element {
    let balance = use_wallet_adapter::use_balance();
    let e = match *balance.read() {
        Some(bal) => {
            rsx! {
                div {
                    "balance: {bal}"
                }
            }
        }
        None => {
            rsx! {
                div {
                    "waiting for balance"
                }
            }
        }
    };
    e
}
