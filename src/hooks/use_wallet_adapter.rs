use dioxus::prelude::*;
use solana_client_wasm::{solana_sdk::pubkey::Pubkey, WasmClient};

const RPC_URL: &str = "http://localhost:8899";
pub enum WalletAdapter {
    Connected { pubkey: Pubkey },
    Disconnected,
}

pub fn use_balance() -> Resource<u64> {
    let wa_signal = use_wallet_adapter();
    use_resource(move || async move {
        match *wa_signal.read() {
            WalletAdapter::Connected { pubkey } => {
                dioxus_logger::tracing::info!("pubkey: {}", pubkey);
                let rpc = WasmClient::new(RPC_URL);
                match rpc.get_balance(&pubkey).await {
                    Ok(balance) => balance,
                    Err(err) => {
                        dioxus_logger::tracing::info!("err fetching balance: {}", err);
                        0
                    }
                }
            }
            WalletAdapter::Disconnected => {
                dioxus_logger::tracing::info!("disconnected");
                0
            }
        }
    })
}

pub fn use_slot() -> Resource<Option<u64>> {
    let wa_signal = use_wallet_adapter();
    use_resource(move || async move {
        match *wa_signal.read() {
            WalletAdapter::Connected { pubkey } => {
                dioxus_logger::tracing::info!("pubkey: {}", pubkey);
                let rpc = WasmClient::new(RPC_URL);
                match rpc.get_slot().await {
                    Ok(slot) => Some(slot),
                    Err(err) => {
                        dioxus_logger::tracing::info!("err fetching slot: {}", err);
                        None
                    }
                }
            }
            WalletAdapter::Disconnected => {
                dioxus_logger::tracing::info!("disconnected");
                None
            }
        }
    })
}

pub fn use_wallet_adapter() -> Signal<WalletAdapter> {
    use_context::<Signal<WalletAdapter>>()
}
pub fn use_wallet_adapter_provider() {
    let mut signal = use_context_provider(|| Signal::new(WalletAdapter::Disconnected));
    let mut eval = eval(
        r#"
            window.addEventListener("dwa-pubkey", (event) => {
                console.log(event.detail);
                dioxus.send(event.detail.pubkey.toBuffer().toJSON().data);
            });
        "#,
    );
    spawn(async move {
        while let Ok(json_val) = eval.recv().await {
            let pubkey_result: Result<Pubkey, serde_json::Error> = serde_json::from_value(json_val);
            match pubkey_result {
                Ok(pubkey) => signal.set(WalletAdapter::Connected { pubkey }),
                Err(_) => signal.set(WalletAdapter::Disconnected),
            }
        }
    });
}
