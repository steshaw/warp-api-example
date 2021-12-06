use warp::Filter;
use std::convert::Infallible;

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
        Err(err) => panic!("Argh! {}", err) // FIXME
    }
}

async fn find_is_legendary() -> Result<bool, Box<dyn std::error::Error>> {
    let url = "https://pokeapi.co/api/v2/pokemon-species/mewtwo";
    let species: serde_json::Value = reqwest::get(url).await?.json().await?;
    println!("species = {:#?}", species);
    let opt_leg = species.get("is_legendary");
    println!("opt_leg = {:#?}", opt_leg);
    match opt_leg {
        Some(serde_json::Value::Bool(is_legendary)) => Ok(*is_legendary),
        Some(_) => Err("The key \"is_legendary\" is not a bool".into()),
        None => Err("Cannot find key \"legendary\"".into()),
    }
}

async fn pokemon(name: String) -> Result<impl warp::Reply, Infallible> {
    format!("Hello Pokémon, {}\n", name);
    let l = find_is_legendary().await;
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
