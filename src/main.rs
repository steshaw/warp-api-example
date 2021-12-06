use warp::Filter;

fn pokemon(name: String) -> String {
    format!("Hello Pokémon, {}\n", name)
}

async fn argh() -> Result<(), reqwest::Error> {
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
    println!("argh = {:#?}", echo_json);
    Ok(())
}

async fn your_ip() -> Result<String, Box<dyn std::error::Error>> {
    let resp: serde_json::Value = reqwest::get("https://httpbin.org/ip").await?.json().await?;
    println!("resp = {:#?}", resp);
    let opt_ip = resp.get("origin");
    println!("opt_ip = {:#?}", opt_ip);
    match opt_ip {
        Some(serde_json::Value::String(ip)) => Ok(ip.to_string()),
        Some(_) => Err("The key \"origin\" is not a string".into()),
        None => Err("Cannot find key \"origin\"".into()),
    }
}

async fn ip() -> String {
    match your_ip().await {
        Ok(s) => s,
        Err(err) => std::panic!("{}", err), // XXX: Argh!
    }
}

#[tokio::main]
async fn main() {
    match your_ip().await {
        Ok(ip) => println!("your_ip = {:#?}", ip),
        Err(err) => println!("Error getting ip: {}", err),
    }

    let r = argh().await;
    println!("argh = {:#?}", r);

    // GET /pokemon/mewtwo => 200 OK with body "Hello, mewtwo!"
    let pokemon = warp::path!("pokemon" / String).map(pokemon);

    let echo = warp::path!("echo" / String).map(|echo| format!("Echo {}\n", echo));

    let _ip = warp::path!("ip").map(ip); // TODO

    let health = warp::path!("health").map(|| "ok\n");

    let default = warp::any().map(|| "hmm\n");

    warp::serve(pokemon.or(echo).or(health).or(default))
        .run(([0, 0, 0, 0], 3000))
        .await;
}
