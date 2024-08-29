#![allow(non_snake_case)]

use dioxus::prelude::*;
use hooks::use_wallet_adapter::use_balance;
use solana_client_wasm::{
    solana_sdk::{native_token::lamports_to_sol, transaction::Transaction},
    WasmClient,
};
use tracing::Level;

use crate::hooks::use_wallet_adapter::{
    invoke_signature, use_wallet_adapter, use_wallet_adapter_provider, InvokeSignatureStatus,
    WalletAdapter,
};

mod hooks;

pub const RPC_URL: &str =
    "https://mainnet.helius-rpc.com/?api-key=1de92644-323b-4900-9041-13c02730955c";

fn main() {
    // Init logger
    dioxus_logger::init(Level::INFO).expect("failed to init logger");
    launch(App);
}

fn App() -> Element {
    use_wallet_adapter_provider();
    rsx! {
        MountWalletAdapter {}
        RenderBalance {}
        SignMemo {}
    }
}

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

fn RenderBalance() -> Element {
    let balance = use_balance();
    let e = match *balance.read() {
        Some(bal) => {
            rsx! {
                div {
                    "Balance: {lamports_to_sol(bal)} SOL"
                }
            }
        }
        None => {
            rsx! {
                div {
                    "Loading balance"
                }
            }
        }
    };
    e
}

fn SignMemo() -> Element {
    let status = use_signal(|| InvokeSignatureStatus::Start);
    let wallet_adapter = use_wallet_adapter();

    let tx = use_resource(move || async move {
        match *wallet_adapter.read() {
            WalletAdapter::Disconnected => None,
            WalletAdapter::Connected { pubkey } => {
                let rpc = WasmClient::new(RPC_URL);
                let ix = solana_extra_wasm::program::spl_memo::build_memo(
                    "Hello, world".as_bytes(),
                    &[&pubkey],
                );
                let mut tx = Transaction::new_with_payer(&vec![ix], Some(&pubkey));
                tx.message.recent_blockhash = rpc.get_latest_blockhash().await.unwrap();
                Some(tx)
            }
        }
    });

    rsx! {
        if let Some(Some(tx)) = tx.cloned() {
            match *status.read() {
                InvokeSignatureStatus::Start => rsx! {
                    button {
                        onclick: move |_| {
                            invoke_signature(tx.clone(), status);
                        },
                        "Submit memo transaction"
                    }
                },
                InvokeSignatureStatus::Waiting => rsx! { p { "Submitting..." } },
                InvokeSignatureStatus::DoneWithError => rsx! { p { "Error" } },
                InvokeSignatureStatus::Timeout => rsx! { p { "Timeout" } },
                InvokeSignatureStatus::Done(sig) => rsx! { p { "{sig}" } },
            }
        } else {
            p {
                "Loading tx"
            }
        }
    }
}
