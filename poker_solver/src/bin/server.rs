use tokio::net::{TcpListener, TcpStream};
use tokio::stream::{Stream, StreamExt};
use tokio::sync::{mpsc, Mutex};
use tokio_util::codec::Framed;

use futures::SinkExt;
use std::collections::HashMap;
use std::error::Error;
use std::io;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use poker_solver::codec::{PokerCodec, PokerCodecError};
use poker_solver::event::PokerEvent;

type Tx = mpsc::UnboundedSender<PokerEvent>;
type Rx = mpsc::UnboundedReceiver<PokerEvent>;

/// Shared state
struct Shared {
    peers: HashMap<SocketAddr, Tx>
}

impl Shared {
    fn new() -> Self {
        Shared {
            peers: HashMap::new()
        }
    }
}

struct Peer {
    rx: Rx,
    codec: Framed<TcpStream, PokerCodec>
}

#[derive(Debug)]
enum Message {
    FromClient(PokerEvent),
    FromServer(PokerEvent)
}

impl Stream for Peer {
    type Item = Result<Message, PokerCodecError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // Receive a message from server to send to client
        if let Poll::Ready(Some(event)) = Pin::new(&mut self.rx).poll_next(cx) {
            return Poll::Ready(Some(Ok(Message::FromServer(event))));
        }

        // Secondly poll the `Framed` stream.
        let result: Option<_> = futures::ready!(Pin::new(&mut self.codec).poll_next(cx));

        Poll::Ready(match result {
            // We've received a message from the client
            Some(Ok(event)) => Some(Ok(Message::FromClient(event))),

            // An error occurred.
            Some(Err(e)) => Some(Err(e)),

            // The stream has been exhausted.
            None => None,
        })
    }
}

impl Peer {
    async fn new(state: Arc<Mutex<Shared>>, codec: Framed<TcpStream, PokerCodec>) -> io::Result<Peer> {
        let addr = codec.get_ref().peer_addr()?;
        let (tx, rx) = mpsc::unbounded_channel();
        state.lock().await.peers.insert(addr, tx);
        Ok(Peer {
            rx,
            codec
        })
    }
}

async fn handle_client(state: Arc<Mutex<Shared>>, tx: Tx, stream: TcpStream, addr: SocketAddr) -> Result<(), Box<dyn Error>> {
    // connect client and add peer
    let codec = Framed::new(stream, PokerCodec::new());
    let mut peer = Peer::new(state.clone(), codec).await?;
    // handle message passing
    println!("Client {} has connected", addr);

    while let Some(result) = peer.next().await {
        match result {
            Ok(Message::FromClient(msg)) => {
                // relay message to game server
                tx.send(msg.clone())?;
            },
            Ok(Message::FromServer(msg)) => {
                // relay message to client
                peer.codec.send(msg).await?;
            },
            Err(e) => {
                println!("recv err: {}", e);
            }
        }
    }
    // disconnect client
    {
        println!("Client {} has disconnected", addr);
        let mut state = state.lock().await;
        state.peers.remove(&addr);
    }
    Ok(())
}

async fn game_loop(state: Arc<Mutex<Shared>>, mut rx: Rx) -> Result<(), Box<dyn Error>> {
    while let Some(event) = rx.next().await {
        // demo
        // let mut state = state.lock().await;
        // let tx = state.peers.get(&event.from).unwrap();
        // tx.send(event.clone())?;
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("0.0.0.0:3333").await?;
    let (tx, rx) = mpsc::unbounded_channel();
    let state = Arc::new(Mutex::new(Shared::new()));
    println!("server listening on 0.0.0.0:3333");
    let _state = state.clone();
    // Create game handler
    tokio::spawn(async move {
        // handle game loop
        if let Err(e) = game_loop(_state, rx).await {
            println!("game loop returned error: {}", e);
        }
    });
    // Spawn client handlers
    loop {
        let (stream, addr) = listener.accept().await?;
        let state = state.clone();
        let tx = tx.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_client(state, tx, stream, addr).await {
                println!("error occured: {}", e);
            }
        });
    }
}