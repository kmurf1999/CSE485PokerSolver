use crate::{mpsc,Event, Games};
use futures::channel::mpsc;
use futures::stream::StreamExt;
use warp::ws::Message;
use futures::SinkExt;

use warp::filters::ws::Message;

pub async fn agent_loop(
    mut receiver: mpsc::UnboundedReceiver<std::result::Result<Message, warp::Error>>,
    game_id: String,
    games: Games,
) {
    loop {
        match receiver.next().await {
            Some(message) => {
                println!("msg: {:?}", message);
            }
            None => {
                break;
            }
        }
    }
}
