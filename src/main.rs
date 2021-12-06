use serde::{Deserialize, Serialize};
use warp::Filter;

#[derive(Debug, Serialize, Deserialize)]
struct Habitat {
    name: String,
    url: url::Url,
}

#[derive(Debug, Serialize, Deserialize)]
struct Language {
    name: String,
    url: url::Url,
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
    // FIXME: Remove unwraps.
    let url = url::Url::parse("https://pokeapi.co/api/v2/pokemon-species/").unwrap();
    println!("url 1 = {}", url);
    let url = url.join(name.as_str()).unwrap();
    println!("url 2 = {}", url);
    let url = url.to_string();
    println!("url 3 = {}", url);
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

fn translation_url(translation: &str, desc: &str) -> String {
    //let _url = format!("https://api.funtranslations.com/translate/{}.json?text=Master%20Obiwan%20has%20lost%20a%20planet.", translation);
    let base = format!(
        "https://api.funtranslations.com/translate/{}.json",
        translation
    );
    // Fun Translation doesn't seem to like newline characters, so let's just replace them with
    // spaces for now (because we really want to have fun).
    // Alternatively, we could split into lines, translate each?, then join back together.
    let d = str::replace(desc, "\n", " ");
    url::Url::parse_with_params(base.as_str(), &[("text", d)])
        .expect("Must be valid")
        .to_string()
}

#[derive(Debug, Serialize, Deserialize)]
struct TranslationTotal {
    total: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct TranslationContents {
    translated: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct TranslationResult {
    success: TranslationTotal,
    contents: TranslationContents,
}
/*
{
  "success": {
    "total": 1
  },
  "contents": {
    "translated": "Lost a planet,  master obiwan has.",
    "text": "Master Obiwan has lost a planet.",
    "translation": "yoda"
  }
}

Object({
    "error": Object({
        "code": Number(
            429,
        ),
        "message": String(
            "Too Many Requests: Rate limit of 5 requests per hour exceeded. Please wait for 59 minutes and 4 seconds.",
        ),
    }),
})
*/
async fn translate(translation: &str, desc: &String) -> Result<TranslationResult, reqwest::Error> {
    if true {
        let url = translation_url(translation, desc);
        println!("translation_url = {:#?}", url);
        let v: serde_json::Value = reqwest::get(&url).await?.json().await?;
        println!("translation = {:#?}", v);
        let tr: TranslationResult = reqwest::get(&url).await?.json().await?;
        println!("tr = {:#?}", tr);
        Ok(tr)
    } else {
        // We are rate-limited, so use fake response.
        let r = TranslationResult {
            success: TranslationTotal { total: 1 },
            contents: TranslationContents {
                translated: "Bogus translated text".to_string(),
            },
        };
        Ok(r)
    }
}

async fn try_translation(info: PokemonInfo) -> Result<PokemonInfo, reqwest::Error> {
    let translation = if info.habitat == "cave" || info.is_legendary {
        "yoda"
    } else {
        "shakespeare"
    };
    let x = translate(translation, &info.description).await;
    match x {
        Ok(tr) => {
            if tr.success.total == 1 {
                // The Yoda translation results in double spacing. Replace with single spacing.
                let d = str::replace(tr.contents.translated.as_str(), "  ", " ");
                Ok(PokemonInfo {
                    name: info.name,
                    habitat: info.habitat,
                    is_legendary: info.is_legendary,
                    description: d,
                })
            } else {
                println!("Translation success total was not 1");
                Ok(info)
            }
        }
        Err(err) => {
            println!("There was an error: {}", err);
            Ok(info)}
        ,
    }
}

async fn pokemon(name: String) -> Result<impl warp::Reply, warp::Rejection> {
    // XXX: Remove the to_string()
    let resp: Result<PokemonInfo, warp::Rejection> = find_species(name.to_string())
        .await
        .map_err(to_rejection)
        .and_then(|species| {
            let opt_desc = get_description(species.flavor_text_entries);
            match opt_desc {
                Some(desc) => {
                    let info = PokemonInfo {
                        name: species.name,
                        habitat: species.habitat.name,
                        is_legendary: species.is_legendary,
                        description: desc,
                    };
                    Ok(info)
                }
                None => Err(warp::reject::not_found()),
            }
        });
    match resp {
        Ok(info) => try_translation(info).await.map_err(to_rejection),
        e => e,
    }
}

#[tokio::main]
async fn main() {
    if false {
        // TODO: Would make a nice integration test of the Translation API.
        // Avoid testing the Translation API as we get rate limited pretty quickly.
        let desc = "You are great!".to_string();
        let t1 = translate("yoda", &desc).await;
        println!("yoda = {:#?}", t1);
        let t2 = translate("shakespeare", &desc).await;
        println!("shakespeare = {:#?}", t2);
    }

    // GET /pokemon/mewtwo => 200 OK with body "Hello, mewtwo!"
    let pokemon = warp::path!("pokemon" / String).and_then(pokemon);

    let health = warp::path!("health").map(|| "ok\n");

    warp::serve(
        pokemon.or(health),
    )
    .run(([0, 0, 0, 0], 3000))
    .await;
}
