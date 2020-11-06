use std::time::Duration;
use tokio::net::{TcpListener, TcpStream};
use tokio::stream::{Stream, StreamExt};
use tokio::sync::{mpsc, Mutex};
use tokio::task;
use tokio::time::timeout;
use tokio_util::codec::Framed;

use futures::SinkExt;
use std::error::Error;
use std::io;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use poker_solver::action::Action;
use poker_solver::agents::{HumanAgent, RandomAgent};
use poker_solver::card::score_hand;
use poker_solver::codec::{PokerCodec, PokerCodecError};
use poker_solver::event::{PokerEvent, PokerEventType};
use poker_solver::round::BettingRound;
use poker_solver::state::GameState;

/// Maximum of seconds to wait for a player action
static TIMEOUT: u64 = 30;

type Tx = mpsc::UnboundedSender<PokerEvent>;
type Rx = mpsc::UnboundedReceiver<PokerEvent>;

/// Shared state
struct Shared {
    peers: Vec<(SocketAddr, Tx)>,
}

impl Shared {
    fn new() -> Self {
        Shared { peers: Vec::new() }
    }
}

struct Peer {
    rx: Rx,
    codec: Framed<TcpStream, PokerCodec>,
}

#[derive(Debug)]
enum Message {
    FromClient(PokerEvent),
    FromServer(PokerEvent),
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
    async fn new(
        state: Arc<Mutex<Shared>>,
        codec: Framed<TcpStream, PokerCodec>,
    ) -> io::Result<Peer> {
        let addr = codec.get_ref().peer_addr()?;
        let (tx, rx) = mpsc::unbounded_channel();
        state.lock().await.peers.push((addr, tx));
        Ok(Peer { rx, codec })
    }
}

async fn handle_client(
    state: Arc<Mutex<Shared>>,
    tx: Tx,
    stream: TcpStream,
    addr: SocketAddr,
) -> Result<(), Box<dyn Error>> {
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
            }
            Ok(Message::FromServer(msg)) => {
                // relay message to client
                peer.codec.send(msg).await?;
            }
            Err(e) => {
                println!("recv err: {}", e);
            }
        }
    }
    // disconnect client
    {
        println!("Client {} has disconnected", addr);
        // TODO remove from state.peers
        // let mut state = state.lock().await;
        // state.peers.remove(&addr);
    }
    Ok(())
}

struct Server {
    addr: SocketAddr,
    state: Arc<Mutex<Shared>>,
    stacks: [u32; 2],
    rx: Rx,
}

impl Server {
    fn new(state: Arc<Mutex<Shared>>, rx: Rx, addr: SocketAddr) -> Self {
        Server {
            state,
            addr,
            rx,
            stacks: [1000, 1000],
        }
    }
    async fn start(
        state: Arc<Mutex<Shared>>,
        rx: Rx,
        addr: SocketAddr,
    ) -> Result<(), Box<dyn Error>> {
        let mut server = Server::new(state, rx, addr);
        server.run_game().await
    }
    async fn run_hand(&mut self) -> Result<(), Box<dyn Error>> {
        let mut hand_state = GameState::init_with_blinds(self.stacks, [10, 5], None);
        // tell players hand is starting
        self.send(
            0,
            PokerEvent {
                from: self.addr,
                event: PokerEventType::HandStart {
                    stacks: self.stacks,
                    blinds: [10, 5],
                    position: 0,
                },
            },
        )
        .await;
        self.send(
            1,
            PokerEvent {
                from: self.addr,
                event: PokerEventType::HandStart {
                    stacks: self.stacks,
                    blinds: [10, 5],
                    position: 1,
                },
            },
        )
        .await;
        // deal hands
        self.deal_cards(&mut hand_state).await;
        while !hand_state.is_game_over() {
            // tell current player to make a move
            let current_player = hand_state.current_player_idx();
            self.send(
                current_player.into(),
                PokerEvent {
                    from: self.addr,
                    event: PokerEventType::RequestAction,
                },
            )
            .await;
            // recv action from that player
            // or if action cannot be recieved, apply default action (CHECK / FOLD)
            // give a player 30 seconds to decide
            let action: Action = match timeout(
                Duration::from_secs(TIMEOUT),
                self.recv_action(&mut hand_state, current_player.into()),
            )
            .await
            {
                Ok(Some(action)) => action,
                Ok(None) => hand_state.default_action(),
                Err(_) => hand_state.default_action(),
            };
            hand_state = hand_state.apply_action(action);
            // alert players that action has been taken by a player
            self.sendall(PokerEvent {
                from: self.addr,
                event: PokerEventType::AlertAction {
                    player: current_player,
                    action,
                    stacks: hand_state.stacks(),
                    wagers: hand_state.wagers(),
                    pot: hand_state.pot(),
                },
            })
            .await;
            // deal more cards if necessary
            if hand_state.bets_settled() {
                if hand_state.is_game_over() {
                    break;
                }
                hand_state = hand_state.next_round();
                self.deal_cards(&mut hand_state).await;
            }
        }
        // evaluate winner of hand
        self.stacks = hand_state.stacks();
        if let Some(loser) = hand_state.player_folded() {
            let winner = 1 - loser;
            // add chips to winner
            self.stacks[usize::from(winner)] += hand_state.pot();
            self.sendall(PokerEvent {
                from: self.addr,
                event: PokerEventType::HandOver {
                    winner: 1 - loser,
                    stacks: self.stacks,
                    pot: hand_state.pot(),
                },
            })
            .await;
        } else {
            while hand_state.round() != BettingRound::RIVER {
                self.deal_cards(&mut hand_state).await;
            }
            // find out who won
            let pot = hand_state.pot();
            let score0 = score_hand(hand_state.board(), hand_state.player(0).cards());
            let score1 = score_hand(hand_state.board(), hand_state.player(1).cards());
            let winner: u8;
            if score0 > score1 {
                winner = 0;
                self.stacks[usize::from(winner)] += pot;
            } else if score1 > score0 {
                winner = 1;
                self.stacks[usize::from(winner)] += pot;
            } else {
                winner = 2;
                self.stacks[0] += pot / 2;
                self.stacks[1] += pot / 2;
            }
            self.sendall(PokerEvent {
                from: self.addr,
                event: PokerEventType::HandOver {
                    winner,
                    stacks: self.stacks,
                    pot: hand_state.pot(),
                },
            })
            .await;
        }
        Ok(())
    }
    async fn run_game(&mut self) -> Result<(), Box<dyn Error>> {
        // wait for 2 players to connect
        while self.state.lock().await.peers.len() != 2 {
            task::yield_now().await;
        }
        println!("Two players have connected, starting game");
        // Run hands until game is done
        while self.stacks.iter().position(|&s| s == 0).is_none() {
            self.run_hand().await?;
            // TODO
            // switch stacks and peers
            // This is to switch blinds and who goes first
            // Find a better way to do this
            self.stacks.reverse();
            self.state.lock().await.peers.reverse();
        }
        Ok(())
    }
    /// Receive an action from a player at index
    /// will loop until a valid action has been received from that player
    async fn recv_action(&mut self, hand_state: &mut GameState, from: usize) -> Option<Action> {
        assert!(from < 3);
        let addr = self.state.lock().await.peers[from].0;
        while let Some(msg) = self.rx.next().await {
            // make sure address is valid
            if msg.from != addr {
                continue;
            }
            if let PokerEventType::SendAction { action } = msg.event {
                if hand_state.is_action_valid(action) {
                    return Some(action);
                }
                // tell them they need to try again
                self.send(
                    from,
                    PokerEvent {
                        from: self.addr,
                        event: PokerEventType::RequestAction,
                    },
                )
                .await;
            }
        }
        None
    }
    /// Generate random cards based on the current betting round
    /// and alert players of their new cards
    async fn deal_cards(&mut self, hand_state: &mut GameState) {
        hand_state.deal_cards();
        match hand_state.round() {
            BettingRound::PREFLOP => {
                self.send(
                    0,
                    PokerEvent {
                        from: self.addr,
                        event: PokerEventType::DealCards {
                            round: hand_state.round(),
                            cards: hand_state.current_player().cards().to_vec(),
                            n_cards: 2,
                        },
                    },
                )
                .await;
                self.send(
                    1,
                    PokerEvent {
                        from: self.addr,
                        event: PokerEventType::DealCards {
                            round: hand_state.round(),
                            cards: hand_state.other_player().cards().to_vec(),
                            n_cards: 2,
                        },
                    },
                )
                .await;
            }
            _ => {
                self.sendall(PokerEvent {
                    from: "0.0.0.0:3333".parse().unwrap(),
                    event: PokerEventType::DealCards {
                        round: hand_state.round(),
                        cards: hand_state.board().to_vec(),
                        n_cards: 5,
                    },
                })
                .await;
            }
        }
    }
    /// Send a message to a player at index
    async fn send(&mut self, index: usize, message: PokerEvent) {
        assert!(index < 3);
        let state = self.state.lock().await;
        let _ = state.peers[index].1.send(message);
    }
    /// Send a message to all players
    async fn sendall(&mut self, message: PokerEvent) {
        let mut state = self.state.lock().await;
        for peer in state.peers.iter_mut() {
            let _ = peer.1.send(message.clone());
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("0.0.0.0:3333").await?;
    let server_addr = listener.local_addr().unwrap();
    let (tx, rx) = mpsc::unbounded_channel();
    let state = Arc::new(Mutex::new(Shared::new()));
    println!("server listening on 0.0.0.0:3333");
    let _state = state.clone();
    // Create game handler
    tokio::spawn(async move {
        // handle game loop
        if let Err(e) = Server::start(_state, rx, server_addr).await {
            println!("game loop returned error: {}", e);
        }
    });
    // Connect Game agents
    tokio::spawn(async move {
        if let Err(e) = RandomAgent::start(server_addr).await {
            println!("agent returned error: {}", e);
        }
    });
    tokio::spawn(async move {
        if let Err(e) = HumanAgent::start(server_addr).await {
            println!("agent returned error: {}", e);
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
