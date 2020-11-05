extern crate colored;
extern crate async_std;
extern crate serde;
extern crate serde_json;
extern crate futures;

// use poker_solver::agents::{HumanAgent, RandomAgent};
// use poker_solver::game_environment::GameEnvironment;

// use rand::thread_rng;
use serde::{Serialize, Deserialize};
use async_std::{
    io::BufReader,
    net::{TcpListener, TcpStream, ToSocketAddrs},
    prelude::*,
    task,
};
use futures::channel::mpsc;
use futures::sink::SinkExt;
use std::{
    collections::hash_map::{HashMap, Entry},
    sync::Arc,
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
type Sender<T> = mpsc::UnboundedSender<T>;
type Receiver<T> = mpsc::UnboundedReceiver<T>;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum GameEvent {
    StartGame,
    EndGame,
    StartHand,
    EndHand,
}

// #[derive(Debug)]
// struct Agent {
//     reader: io::BufReader<TcpStream>,
//     writer: io::LineWriter<TcpStream>,
//     player: usize
// }

// impl Agent {
//     pub fn start_agent(player: usize, addr: SocketAddr) -> thread::JoinHandle<()> {
//         thread::spawn(move || {
//             let stream = TcpStream::connect(addr).unwrap();
//             let mut agent = Agent {
//                 writer: io::LineWriter::new(stream.try_clone().unwrap()),
//                 reader: io::BufReader::new(stream),
//                 player
//             };
//         })
//     }
// }


// main
fn run() -> Result<()> {
    task::block_on(accept_loop("127.0.0.1:8080"))
}

fn spawn_and_log_error<F>(fut: F) -> task::JoinHandle<()>
where
    F: Future<Output = Result<()>> + Send + 'static,
{
    task::spawn(async move {
        if let Err(e) = fut.await {
            eprintln!("{}", e)
        }
    })
}

async fn accept_loop(addr: impl ToSocketAddrs) -> Result<()> {
    let listener = TcpListener::bind(addr).await?;

    let (broker_sender, broker_receiver) = mpsc::unbounded(); // 1
    let _broker_handle = task::spawn(broker_loop(broker_receiver));
    let mut incoming = listener.incoming();
    while let Some(stream) = incoming.next().await {
        let stream = stream?;
        println!("Accepting from: {}", stream.peer_addr()?);
        spawn_and_log_error(connection_loop(broker_sender.clone(), stream));
    }
    Ok(())
}

async fn connection_loop(mut broker: Sender<Event>, stream: TcpStream) -> Result<()> {
    let stream = Arc::new(stream); // 2
    let reader = BufReader::new(&*stream);
    let mut lines = reader.lines();

    let name = match lines.next().await {
        None => Err("peer disconnected immediately")?,
        Some(line) => line?,
    };
    broker.send(Event::NewPeer { name: name.clone(), stream: Arc::clone(&stream) }).await // 3
        .unwrap();

    while let Some(line) = lines.next().await {
        let line = line?;
        let (dest, msg) = match line.find(':') {
            None => continue,
            Some(idx) => (&line[..idx], line[idx + 1 ..].trim()),
        };
        let dest: Vec<String> = dest.split(',').map(|name| name.trim().to_string()).collect();
        let msg: String = msg.to_string();

        broker.send(Event::Message { // 4
            from: name.clone(),
            to: dest,
            msg,
        }).await.unwrap();
    }
    Ok(())
}

async fn connection_writer_loop(
    mut messages: Receiver<String>,
    stream: Arc<TcpStream>,
) -> Result<()> {
    let mut stream = &*stream;
    while let Some(msg) = messages.next().await {
        stream.write_all(msg.as_bytes()).await?;
    }
    Ok(())
}

#[derive(Debug)]
enum Event {
    NewPeer {
        name: String,
        stream: Arc<TcpStream>,
    },
    Message {
        from: String,
        to: Vec<String>,
        msg: String,
    },
}

async fn broker_loop(mut events: Receiver<Event>) -> Result<()> {
    let mut peers: HashMap<String, Sender<String>> = HashMap::new();

    while let Some(event) = events.next().await {
        match event {
            Event::Message { from, to, msg } => {
                for addr in to {
                    if let Some(peer) = peers.get_mut(&addr) {
                        let msg = format!("from {}: {}\n", from, msg);
                        peer.send(msg).await?
                    }
                }
            }
            Event::NewPeer { name, stream} => {
                match peers.entry(name) {
                    Entry::Occupied(..) => (),
                    Entry::Vacant(entry) => {
                        let (client_sender, client_receiver) = mpsc::unbounded();
                        entry.insert(client_sender); // 4
                        spawn_and_log_error(connection_writer_loop(client_receiver, stream)); // 5
                    }
                }
            }
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    // let mut game = GameEnvironment {
    //     agents: [
    //         Box::new(HumanAgent::new()),
    //         Box::new(RandomAgent::new()),
    //     ],
    //     rng: thread_rng(),
    //     stacks: [10000; 2]
    // };
    // game.play();
    task::block_on(accept_loop("127.0.0.1:8080"))

    // create game channels

    // create agents
    // let agents = vec![
    //     Agent::start_agent(0, addr),
    //     Agent::start_agent(1, addr)
    // ];
    // join threads and stop server
    // for agent in agents {
    //     let _ = agent.join();
    // }
}
