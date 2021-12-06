use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use warp::Filter;

async fn placeholder() -> Result<(), reqwest::Error> {
    let echo_json: serde_json::Value = reqwest::Client::new()
        .post("https://jsonplaceholder.typicode.com/posts")
        .json(&serde_json::json!({
            "title": "Reqwest.rs",
            "body": "https://docs.rs/reqwest",
            "userId": 1
        }))
        .send()
        .await?
        .json()
        .await?;
    println!("placeholder = {:#?}", echo_json);
    Ok(())
}

async fn your_ip() -> Result<String, Box<dyn std::error::Error>> {
    let resp: serde_json::Value = reqwest::get("https://httpbin.org/ip").await?.json().await?;
    println!("resp = {:#?}", resp);
    let opt_ip = resp.get("origin");
    println!("opt_ip = {:#?}", opt_ip);
    match opt_ip {
        Some(serde_json::Value::String(ip)) => Ok(ip.to_string() + "\n"),
        Some(_) => Err("The key \"origin\" is not a string".into()),
        None => Err("Cannot find key \"origin\"".into()),
    }
}

async fn ip() -> Result<String, Infallible> {
    match your_ip().await {
        Ok(s) => Ok(s),
        Err(err) => panic!("Argh! {}", err), // FIXME
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Habitat {
    name: String,
    url: String, // FIXME: URL type?
}

#[derive(Debug, Serialize, Deserialize)]
struct Species {
    name: String,
    habitat: Habitat,
    is_legendary: bool,
}

async fn find_is_legendary(name: String) -> Result<Species, Box<dyn std::error::Error>> {
    // FIXME: Contruct url properly
    let url = format!("https://pokeapi.co/api/v2/pokemon-species/{}", name);
    let species_value: serde_json::Value = reqwest::get(&url).await?.json().await?;
    println!("species_value = {:#?}", species_value);
    let species: Species = reqwest::get(&url).await?.json::<Species>().await?;
    println!("species = {:#?}", species);
    Ok(species)
}

async fn pokemon(name: String) -> Result<impl warp::Reply, Infallible> {
    format!("Hello Pokémon, {}\n", name);
    let l = find_is_legendary(name.to_string()).await; // XXX: Remove to_string
    let r = format!("name = {}, is_legendary = {:#?}", name, l);
    Ok(r)
}

#[tokio::main]
async fn main() {
    let r = placeholder().await;
    println!("placeholder = {:#?}", r);

    // GET /pokemon/mewtwo => 200 OK with body "Hello, mewtwo!"
    let pokemon = warp::path!("pokemon" / String).and_then(pokemon);

    let echo = warp::path!("echo" / String).map(|echo| format!("Echo {}\n", echo));

    let ip = warp::path!("ip").and_then(ip); // TODO

    let health = warp::path!("health").map(|| "ok\n");

    let default = warp::any().map(|| "hmm\n");

    //warp::serve(pokemon.and_then(echo.or(health).or(default))
    warp::serve(pokemon.or(echo).or(ip).or(health).or(default))
        .run(([0, 0, 0, 0], 3000))
        .await;
}
