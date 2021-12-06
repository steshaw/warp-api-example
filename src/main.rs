use warp::Filter;
use std::collections::HashMap;

fn pokemon(name: String) -> String {
    format!("Hello Pokémon, {}\n", name)
}

async fn your_ip() -> Result<String, Box<dyn std::error::Error>> {
    let resp = reqwest::get("https://httpbin.org/ip")
        .await?
        .json::<HashMap<String, String>>()
        .await?;
    Ok(format!("{:#?}", resp))
}

async fn ip() -> String {
    match your_ip().await {
        Ok(s) => s,
        Err(err) => std::panic!("{}", err), // XXX: Argh!
    }
}

#[tokio::main]
async fn main() {
    let your_ip = your_ip().await;
    println!("your_ip = {:#?}", your_ip);

    // GET /pokemon/mewtwo => 200 OK with body "Hello, mewtwo!"
    let pokemon = warp::path!("pokemon" / String)
        .map(pokemon);

    let echo = warp::path!("echo" / String)
        .map(|echo| format!("Echo {}\n", echo));

    let _ip = warp::path!("ip").map(ip); // TODO

    let health = warp::path!("health").map(||"ok\n");

    let default = warp::any().map(||"hmm\n");

    warp::serve(pokemon.or(echo).or(health).or(default))
        .run(([0, 0, 0, 0], 3000))
        .await;
}
