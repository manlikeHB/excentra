use std::{collections::HashSet, sync::Arc};

use axum::{
    extract::{
        State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::IntoResponse,
};
use futures_util::{
    SinkExt, StreamExt,
    stream::{SplitSink, SplitStream},
};
use tokio::sync::{
    Mutex, broadcast,
    mpsc::{self},
};
use uuid::Uuid;

use crate::{
    api::types::AppState,
    auth::verify_token,
    ws::messages::{Channel, InboundMessage, OutboundMessage, WsEvent},
};

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let (sender, receiver) = socket.split();

    // subscription for THIS / CURRENT connection
    let subscriptions: Arc<Mutex<HashSet<String>>> = Arc::new(Mutex::new(HashSet::new()));
    // auth state
    let user_id: Arc<Mutex<Option<Uuid>>> = Arc::new(Mutex::new(None));

    // communicate between the read and write task
    let (error_tx, error_rx) = mpsc::channel::<OutboundMessage>(32);

    // broadcast receiver
    let rx = state.ws_sender.subscribe();

    let subs_read = subscriptions.clone();
    let subs_write = subscriptions.clone();
    let user_id_read = user_id.clone();
    // let user_id_write = user_id.clone();

    let read_task =
        tokio::spawn(
            async move { read_task(receiver, subs_read, user_id_read, state, error_tx).await },
        );

    let write_task =
        tokio::spawn(async move { write_task(sender, rx, subs_write, error_rx).await });

    tokio::select! {
     _ = read_task => {}
     _ = write_task => {}
    }
}

async fn read_task(
    mut receiver: SplitStream<WebSocket>,
    subscriptions: Arc<Mutex<HashSet<String>>>,
    user_id: Arc<Mutex<Option<Uuid>>>,
    state: Arc<AppState>,
    error_tx: mpsc::Sender<OutboundMessage>,
) {
    // continuously read message from websocket receiver
    while let Some(msg) = receiver.next().await {
        match msg {
            // Deserialize text to expected Inbound message
            Ok(Message::Text(text)) => match serde_json::from_str::<InboundMessage>(&text) {
                // handle various inbound message types
                Ok(msg) => match msg {
                    InboundMessage::Auth { token } => {
                        match verify_token(&token, &state.jwt_secret) {
                            Ok(c) => {
                                // add retrieved user id to user_id
                                *user_id.lock().await = Some(c.user_id());
                                // send auth message after successful verification
                                // this will be received in the write task and sent out to client
                                let _ = error_tx.send(OutboundMessage::Authenticated).await;
                            }
                            Err(_) => {
                                let _ = error_tx
                                    .send(OutboundMessage::Error {
                                        message: "Invalid or expired token".to_string(),
                                    })
                                    .await;
                            }
                        }
                    }
                    // subscribe to a channel
                    InboundMessage::Subscribe { channel } => match Channel::from_str(&channel) {
                        Ok(Channel::Orders(channel_user_id)) => {
                            // orders is a private channel, so the user_id being sent need to check if authorized
                            if is_authorized_for_private(channel_user_id, &user_id, &error_tx).await
                            {
                                let channel_str = Channel::Orders(channel_user_id).to_string();
                                subscriptions.lock().await.insert(channel_str.clone());
                                let _ = error_tx
                                    .send(OutboundMessage::Subscribed {
                                        channel: channel_str,
                                    })
                                    .await;
                            }
                        }
                        Ok(ch) => {
                            // other public channels get added to the channel subs
                            let channel_str = ch.to_string();
                            subscriptions.lock().await.insert(channel_str.clone());
                            let _ = error_tx
                                .send(OutboundMessage::Subscribed {
                                    channel: channel_str,
                                })
                                .await;
                        }
                        Err(msg) => {
                            let _ = error_tx.send(OutboundMessage::Error { message: msg }).await;
                        }
                    },
                    // unsubscribe from a channel
                    InboundMessage::Unsubscribe { channel } => match Channel::from_str(&channel) {
                        Ok(Channel::Orders(channel_user_id)) => {
                            if is_authorized_for_private(channel_user_id, &user_id, &error_tx).await
                            {
                                subscriptions
                                    .lock()
                                    .await
                                    .remove(&Channel::Orders(channel_user_id).to_string());
                            }
                        }
                        Ok(ch) => {
                            let channel_str = ch.to_string();
                            subscriptions.lock().await.remove(&channel_str.clone());
                            let _ = error_tx
                                .send(OutboundMessage::Unsubscribed {
                                    channel: channel_str,
                                })
                                .await;
                        }
                        Err(msg) => {
                            let _ = error_tx.send(OutboundMessage::Error { message: msg }).await;
                        }
                    },
                },
                Err(_) => {
                    let _ = error_tx
                        .send(OutboundMessage::Error {
                            message: "Invalid message format".to_string(),
                        })
                        .await;
                }
            },
            Ok(Message::Close(_)) => break,
            Err(_) => break,
            _ => {}
        }
    }
}

async fn is_authorized_for_private(
    channel_user_id: Uuid,
    user_id: &Arc<Mutex<Option<Uuid>>>,
    error_tx: &mpsc::Sender<OutboundMessage>,
) -> bool {
    match *user_id.lock().await {
        None => {
            let _ = error_tx
                .send(OutboundMessage::Error {
                    message: "Authentication required for private channels".to_string(),
                })
                .await;
            false
        }
        Some(uid) if uid != channel_user_id => {
            let _ = error_tx
                .send(OutboundMessage::Error {
                    message: "Cannot access another user's channel".to_string(),
                })
                .await;
            false
        }
        Some(_) => true,
    }
}

async fn write_task(
    mut sender: SplitSink<WebSocket, Message>,
    mut rx: broadcast::Receiver<WsEvent>,
    subscriptions: Arc<Mutex<HashSet<String>>>,
    mut error_rx: mpsc::Receiver<OutboundMessage>,
) {
    loop {
        tokio::select! {
            // expect msg from error_tx in read task
            Some(msg) = error_rx.recv() => {
                match serde_json::to_string(&msg) {
                    Ok(json) => {
                        let _ = sender.send(Message::Text(json.into())).await;
                    }
                    Err(_) => {}
                }
            }
            // expected events from broadcast
            result = rx.recv() => {
                match result {
                    Ok(event) => {
                        let subs = subscriptions.lock().await;

                        let should_send = match &event {
                            WsEvent::OrderBookUpdate{symbol, ..} => {
                                subs.contains(&format!("orderbook:{}", symbol))
                            }
                            WsEvent::TradeEvent{symbol, ..} => {
                                subs.contains(&format!("trades:{}", symbol))
                            }
                            WsEvent::OrderStatusUpdate{user_id: event_user_id, ..} => {
                                subs.contains(&format!("orders:{}", event_user_id))
                            }
                            WsEvent::TickerUpdate{symbol, ..} => {
                                subs.contains(&format!("ticker:{}", symbol))
                            }
                        };

                        drop(subs);

                        if should_send {
                            let msg = OutboundMessage::Event { data: event };
                            match serde_json::to_string(&msg) {
                                Ok(json) => {
                                    let _ = sender.send(Message::Text(json.into())).await;
                                }
                                Err(_) => {}
                            }
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        // TODO: tracing::warn!
                        eprintln!("WebSocket client lagged, missed {} events", n);
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                }

            }
            else => break
        }
    }
}
