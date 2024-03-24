use dioxus::prelude::*;
#[cfg(feature = "web")]
use solana_client_wasm::solana_sdk::native_token::LAMPORTS_PER_SOL;
#[cfg(feature = "desktop")]
use solana_sdk::native_token::LAMPORTS_PER_SOL;

use crate::{
    components::{try_start_mining, IsToolbarOpen, MinerStatus, MinerStatusMessage},
    gateway::AsyncResult,
    hooks::{use_gateway, use_sol_balance},
    miner::Miner,
};

const MIN_BALANCE: u64 = LAMPORTS_PER_SOL.saturating_div(100);

#[component]
pub fn MinerToolbarActivating(cx: Scope, miner: UseState<Miner>) -> Element {
    let gateway = use_gateway(cx);
    let sufficient_balance = use_state(cx, || true);
    let sol_balance = use_sol_balance(cx);
    let is_toolbar_open = use_shared_state::<IsToolbarOpen>(cx).unwrap();
    let miner_status = use_shared_state::<MinerStatus>(cx).unwrap();
    let miner_status_message = use_shared_state::<MinerStatusMessage>(cx).unwrap();

    use_effect(cx, &sol_balance.clone(), |_| {
        match sol_balance {
            AsyncResult::Ok(balance) => sufficient_balance.set(balance.0.ge(&MIN_BALANCE)),
            _ => sufficient_balance.set(false),
        }
        async move {}
    });

    use_future(cx, &sufficient_balance.clone(), |_| {
        let miner = miner.clone();
        let miner_status = miner_status.clone();
        let miner_status_message = miner_status_message.clone();
        let sufficient_balance = *sufficient_balance.get();
        let gateway = gateway.clone();
        async move {
            if sufficient_balance {
                match try_start_mining(&gateway, miner.get(), &miner_status_message).await {
                    Ok(did_start) => {
                        if did_start {
                            *miner_status.write() = MinerStatus::Active;
                        } else {
                            // TODO Insufficient balance... Set appropriate error
                            log::error!("Insufficient balance to start mining");
                            *miner_status.write() = MinerStatus::NetworkError;
                        };
                    }
                    Err(err) => {
                        // TODO Present error to user
                        log::error!("Failed to start mining: {:?}", err);
                        *miner_status.write() = MinerStatus::NetworkError;
                    }
                }
            }
        }
    });

    if is_toolbar_open.read().0 {
        render! {
            div {
                class: "flex flex-col grow gap-2 px-4 py-6 sm:px-8 sm:py-8",
                h2 {
                    class: "text-3xl md:text-4xl lg:text-5xl font-bold",
                    "Starting"
                }
                match *miner_status_message.read() {
                    MinerStatusMessage::CreatingTokenAccount => {
                        render! {
                            p {
                                class: "text-lg",
                                "Creating token account..."
                            }
                        }
                    }
                    MinerStatusMessage::GeneratingChallenge => {
                        render! {
                            p {
                                class: "text-lg",
                                "Generating challenge..."
                            }
                        }
                    }
                    _ => None
                }
            }
        }
    } else {
        render! {
            div {
                p {
                    class: "font-medium my-auto",
                    "Starting"
                }
            }
        }
    }
}
