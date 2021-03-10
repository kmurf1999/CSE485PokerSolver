use warp::Filter;
use server::{with_games, with_clients, handler, Clients, Games};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let games: Games = Default::default();
    let clients: Clients = Default::default();

    let cors = warp::cors()
        .allow_methods(vec!["GET", "POST"])
        .allow_header("content-type")
        .allow_header("authorization")
        .allow_any_origin()
        .build();

    // route to create a game and return game ID
    let create_game = warp::path("create")
        .and(warp::post())
        .and(with_games(games.clone()))
        .and(with_clients(clients.clone()))
        .and_then(handler::create_game_handler);

    // route to join game
    let join_game = warp::path("join")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_games(games.clone()))
        .and(with_clients(clients.clone()))
        .and_then(handler::join_game_handler);

    // main ws route
    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(warp::path::param())
        .and(with_games(games.clone()))
        .and(with_clients(clients.clone()))
        .and_then(handler::ws_handler);

    let routes = create_game
        .or(join_game)
        .or(ws_route)
        .with(cors);

    warp::serve(routes).run(([127, 0, 0, 1], 3001)).await;
}