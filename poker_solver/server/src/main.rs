use warp::Filter;
use server::{with_games, with_clients, with_lobby, handler, Clients, Games, Lobby};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let games: Games = Default::default();
    let clients: Clients = Default::default();
    let lobby: Lobby = Default::default();

    let cors = warp::cors()
        .allow_methods(vec!["GET", "POST"])
        .allow_header("content-type")
        .allow_header("authorization")
        .allow_any_origin()
        .build();

    // route to join game
    let join_game = warp::path("join")
        .and(warp::post())
        .and(with_lobby(lobby.clone()))
        // .and(with_games(games.clone()))
        .and(with_clients(clients.clone()))
        .and_then(handler::join_handler);

    // main ws route
    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(warp::path::param())
        .and(with_games(games.clone()))
        .and(with_clients(clients.clone()))
        .and_then(handler::ws_handler);

    let routes = join_game
        .or(ws_route)
        .with(cors);

    tokio::spawn(async move {
        handler::lobby_handler(lobby.clone(), games.clone(), clients.clone()).await;
    });

    warp::serve(routes).run(([127, 0, 0, 1], 3001)).await;
}