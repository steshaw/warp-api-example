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
struct Language {
    name: String,
    url: String, // FIXME: URL type?
}

#[derive(Debug, Serialize, Deserialize)]
struct FlavorText {
    flavor_text: String,
    language: Language,
}

#[derive(Debug, Serialize, Deserialize)]
struct Species {
    name: String,
    habitat: Habitat,
    is_legendary: bool,
    flavor_text_entries: Vec<FlavorText>,
}

async fn find_species(name: String) -> Result<Species, reqwest::Error> {
    // FIXME: Contruct url properly
    let url = format!("https://pokeapi.co/api/v2/pokemon-species/{}", name);
    let species_value: serde_json::Value = reqwest::get(&url).await?.json().await?;
    println!("species_value = {:#?}", species_value);
    let flavor_text_entries = species_value.get("flavor_text_entries");
    println!("flavor_text_entries = {:#?}", flavor_text_entries);
    let species: Species = reqwest::get(&url).await?.json::<Species>().await?;
    println!("species = {:#?}", species);
    Ok(species)
}

#[derive(Debug, Serialize, Deserialize)]
struct PokemonInfo {
    name: String,
    habitat: String,
    is_legendary: bool,
    description: String,
}

impl warp::Reply for PokemonInfo {
    fn into_response(self) -> warp::reply::Response {
        let json = warp::reply::json(&self);
        warp::reply::with_status(json, warp::http::StatusCode::OK).into_response()
    }
}

// XXX: Can this be an `into()`?
fn to_rejection(e: reqwest::Error) -> warp::Rejection {
    // TODO: map the `e` here.
    warp::reject::not_found()
}

fn get_description(flavor_text_entries: Vec<FlavorText>) -> Option<String> {
    flavor_text_entries
        .iter()
        .find(|e| e.language.name == "en")
        .map(|e| e.flavor_text.to_string()) // TODO: Remove to_string
}

async fn pokemon(name: String) -> Result<impl warp::Reply, warp::Rejection> {
    // XXX: Remove the to_string()
    find_species(name.to_string())
        .await
        .map_err(to_rejection)
        .and_then(|species| {
            let opt_desc = get_description(species.flavor_text_entries);
            match opt_desc {
                Some(desc) => Ok(PokemonInfo {
                    name: species.name,
                    habitat: species.habitat.name,
                    is_legendary: species.is_legendary,
                    description: desc,
                }),
                None => Err(warp::reject::not_found()),
            }
        })
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

    let _default = warp::any().map(|| "hmm\n");

    warp::serve(
        pokemon.or(echo).or(ip).or(health), //.or(default) // Disable for now otherwise not_found errors are picked up by `default`
    )
    .run(([0, 0, 0, 0], 3000))
    .await;
}
