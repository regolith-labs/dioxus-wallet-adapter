use base64::Engine;
use dioxus::prelude::*;
use solana_client_wasm::{
    solana_sdk::{pubkey::Pubkey, signature::Signature, transaction::Transaction},
    ClientError, WasmClient,
};
use solana_extra_wasm::transaction_status::TransactionConfirmationStatus;
use web_time::Duration;

use crate::RPC_URL;

pub enum WalletAdapter {
    Connected { pubkey: Pubkey },
    Disconnected,
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
                dioxus.send(event.detail.pubkey);
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

pub fn use_balance() -> Resource<u64> {
    let wallet_adapter = use_wallet_adapter();
    use_resource(move || async move {
        match *wallet_adapter.read() {
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

pub fn invoke_signature(tx: Transaction, mut signal: Signal<InvokeSignatureStatus>) {
    signal.set(InvokeSignatureStatus::Waiting);
    let mut eval = eval(
        r#"
        let msg = await dioxus.recv();
        let signed = await window.DwaTxSigner({b64: msg});
        dioxus.send(signed);
        "#,
    );
    match bincode::serialize(&tx) {
        Ok(vec) => {
            let b64 = base64::engine::general_purpose::STANDARD.encode(vec);
            let res = eval.send(serde_json::Value::String(b64));
            match res {
                Ok(()) => {
                    spawn(async move {
                        let res = eval.recv().await;
                        match res {
                            Ok(serde_json::Value::String(string)) => {
                                let rpc = WasmClient::new(RPC_URL);
                                let decode_res = base64::engine::general_purpose::STANDARD
                                    .decode(string)
                                    .ok()
                                    .and_then(|buffer| bincode::deserialize(&buffer).ok());
                                let rpc_res = match decode_res {
                                    Some(tx) => {
                                        dioxus_logger::tracing::info!("Sending: {:?}", tx);
                                        let x = rpc.send_transaction(&tx).await;
                                        dioxus_logger::tracing::info!("Sent: {:?}", x);
                                        x.ok()
                                    }
                                    None => {
                                        dioxus_logger::tracing::info!("error decoding tx");
                                        None
                                    }
                                };
                                dioxus_logger::tracing::info!("Dec: {:?}", rpc_res);
                                match rpc_res {
                                    Some(sig) => {
                                        dioxus_logger::tracing::info!("sig: {}", sig);
                                        let confirmed = confirm_signature(sig).await;
                                        if confirmed.is_ok() {
                                            signal.set(InvokeSignatureStatus::Done(sig));
                                        } else {
                                            signal.set(InvokeSignatureStatus::Timeout)
                                        }
                                    }
                                    None => {
                                        dioxus_logger::tracing::info!("error sending tx");
                                        signal.set(InvokeSignatureStatus::DoneWithError)
                                    }
                                }
                            }
                            _ => {
                                dioxus_logger::tracing::info!("err recv val");
                                signal.set(InvokeSignatureStatus::DoneWithError)
                            }
                        };
                    });
                }
                Err(_err) => {
                    dioxus_logger::tracing::info!("err sending val");
                    signal.set(InvokeSignatureStatus::DoneWithError)
                }
            }
        }
        Err(err) => {
            dioxus_logger::tracing::info!("err serializing tx: {}", err);
            signal.set(InvokeSignatureStatus::DoneWithError)
        }
    };
}

pub async fn confirm_signature(sig: Signature) -> Result<InvokeSignatureStatus, ClientError> {
    // Confirm tx
    const CONFIRM_RETRIES: usize = 20;
    const CONFIRM_DELAY: u64 = 500;
    let rpc = WasmClient::new(RPC_URL);
    for _ in 0..CONFIRM_RETRIES {
        // Delay before confirming
        async_std::task::sleep(Duration::from_millis(CONFIRM_DELAY)).await;

        // Fetch transaction status
        match rpc.get_signature_statuses(&[sig]).await {
            Ok(signature_statuses) => {
                for signature_status in signature_statuses {
                    if let Some(signature_status) = signature_status.as_ref() {
                        if signature_status.confirmation_status.is_some() {
                            if let Some(current_commitment) =
                                signature_status.confirmation_status.as_ref()
                            {
                                match current_commitment {
                                    TransactionConfirmationStatus::Processed => {}
                                    TransactionConfirmationStatus::Confirmed
                                    | TransactionConfirmationStatus::Finalized => {
                                        dioxus_logger::tracing::info!("Confirmed: true");
                                        return Ok(InvokeSignatureStatus::Done(sig));
                                    }
                                }
                            }
                        } else {
                            dioxus_logger::tracing::info!("No status");
                        }
                    }
                }
            }

            // Handle confirmation errors
            Err(err) => {
                dioxus_logger::tracing::error!("Error confirming: {:?}", err);
            }
        }
    }

    Ok(InvokeSignatureStatus::Timeout)
}

#[derive(PartialEq)]
pub enum InvokeSignatureStatus {
    Start,
    Waiting,
    DoneWithError,
    Timeout,
    Done(Signature),
}
