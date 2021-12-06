use warp::Filter;

fn pokemon(name: String) -> String {
    format!("Hello Pokémon, {}\n", name)
}

#[tokio::main]
async fn main() {
    // GET /pokemon/mewtwo => 200 OK with body "Hello, mewtwo!"
    let pokemon = warp::path!("pokemon" / String)
        .map(pokemon);

    let echo = warp::path!("echo" / String)
        .map(|echo| format!("Echo {}\n", echo));

    let health = warp::path!("health").map(||"ok\n");

    let default = warp::any().map(||"hmm\n");

    warp::serve(pokemon.or(echo).or(health).or(default))
        .run(([0, 0, 0, 0], 3000))
        .await;
}
