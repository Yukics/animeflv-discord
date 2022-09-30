use reqwest;
use tokio;
use scraper;
use serde_json;
use serde_json::json;
use unicode_segmentation::UnicodeSegmentation;
use time::Duration;
use std::{thread, time, env};

// Async manager/helper
#[tokio::main]
async fn main ()-> Result<(), Box<dyn std::error::Error>> {

    // ? Get constants and parse env vars
    let proxy = env::var("PROXY").unwrap().to_string();
    // let proxy = "http://localhost:8191/v1";

    let animeflv_url = env::var("ANIMEFLV_URL").unwrap().to_string();
    // let animeflv_url = "https://www3.animeflv.net";

    let millis = parse_time(env::var("CHECK_INTERVAL").unwrap().to_string());

    let payload = json!({
        "cmd": "request.get",
        "url": &animeflv_url,
        "maxTimeout": 60000
    });

    let mut last_anime_list_array: Vec<String> = Vec::new();

    loop {
        // ? We create the webrequest through flareresolver, cloudflare bypasser
        let resp = reqwest::Client::new()
            .post(&proxy)
            .header("ContentType", "application/json")
            .json(&payload)
            .send()
            .await?;

        let resp_parsed = resp.text().await?;
        let resp_json: serde_json::Value = serde_json::from_str(&resp_parsed)?;
        let html = (&resp_json["solution"]["response"]).to_string();

        // ? The "replace" is due to json string bad formatting Ex: <strong class="\\\"Title\\\"">, 
        // ? then parses the html string to scrap it Ex: Element(<strong class="Title">), 
        let document = scraper::Html::parse_document(&html.replace("\\", ""));
        
        let mut anime_list_array: Vec<String> = Vec::new();
        let anime_list_selector = scraper::Selector::parse("main.Main ul.ListEpisodios li a").unwrap();

        let anime_name_selector = scraper::Selector::parse("strong.Title").unwrap();
        let anime_cap_selector = scraper::Selector::parse("span.Capi").unwrap();
        let anime_image_selector = scraper::Selector::parse("span.Image>img").unwrap();

        // ? Get list of animes on landpage
        for element in document.select(&anime_list_selector) {
            let anime_link= element.value().attr("href").expect("Could not find href attribute");
            let anime_name_element = element.select(&anime_name_selector).next().expect("Could not find anime name.");
            let anime_cap_element = element.select(&anime_cap_selector).next().expect("Could not find cap number.");
            let anime_image = element.select(&anime_image_selector).next().expect("Could not find anime image.").value().attr("src").expect("Could not find anime image.");

            let anime_name = anime_name_element.text().collect::<String>();
            let anime_cap = anime_cap_element.text().collect::<String>();

            anime_list_array.push(anime_name);
            anime_list_array.push(animeflv_url.to_owned() + anime_link);
            anime_list_array.push(anime_cap);
            anime_list_array.push(animeflv_url.to_owned() + anime_image);
        }   

        // ? If the animeflv landpage changes it sends as many messages as new animes have been uploaded 
        if last_anime_list_array != anime_list_array && last_anime_list_array.len() > 0{
            let last = &last_anime_list_array;
            let new = &anime_list_array;
            tokio::task::spawn(send_discord(last.to_vec(),new.to_vec()));
        } 
        // ? Unocmment only for debugging
        // else {
        //     println!("Nothing new");
        // }

        last_anime_list_array = anime_list_array;

        thread::sleep(millis);
    }
}

async fn send_discord(last: Vec<String>, new: Vec<String>)-> (){
    println!("Sending message");

    let discord_wh = env::var("DISCORD_WEBHOOK").unwrap().to_string();
    // let discord_wh = "https://discord.com/api/webhooks/1023348456426835989/ihPFQ5U6M1w-yQvG-ajsn9wxshB_gc_yqcrALxkDt9m48_r7uVquo_SFVyf0Hx9p_cOA";
    let index = new.iter().position(|r| r == &last[0]);
    let max_range = (index.unwrap() + 1)/4;
    
    for new_anime in 0..max_range{
        
        // ? Builds message json
        let new_message = json!({
            "embeds": [
                {
                "author": {
                    "name": "AnimeFLV",
                    "url": "https://www3.animeflv.net/",
                    "icon_url": "https://descargas.ams3.digitaloceanspaces.com/images/8212/animeflv-free_android_6647_1.png"
                },
                "title": new[0+(new_anime*4)],
                "url": new[1+(new_anime*4)],
                "color": 15258703,
                "fields": [
                    {
                    "name": "Cap",
                    "value": new[2+(new_anime*4)],
                    "inline": true
                    }
                ],
                "image": {
                        "url": new[3+(new_anime*4)]
                    }
                }
            ]
        });

        // ? Sends message to discord webhook
        let resp = reqwest::Client::new()
        .post(&discord_wh)
        .header("ContentType", "application/json")
        .json(&new_message)
        .send()
        .await.unwrap();

        // ? Checks response
        if resp.status().is_success() {
            println!("OK: Message sent {:?} {:?}", &new[0+(new_anime*4)], &new[2+(new_anime*4)]);
        } else if resp.status().is_server_error() {
            println!("ERROR: Discord server might not be available right now so the new anime notification is lost");
        } else {
            println!("Something else happened. Status: {:?}", resp.status());
        }
    }
}

// ? This function parses 10s, 5m, 1h format to miliseconds
fn parse_time(time: String)-> Duration{

    let mut milisecs: u64 = 1000;

    // ? Get only numerical part, all but last character
    let index = time.graphemes(true).count() - 1; 
    let quantity = (&time[..index]).parse::<u64>().unwrap();

    // ? Get last character
    let format = time.chars().rev().nth(0).unwrap();

    // ? Default time will be 10s
    match format {
        's' => milisecs = &milisecs * &quantity,
        'm' => milisecs = &milisecs * 60 * &quantity,
        'h' => milisecs = &milisecs * 360 * &quantity,
         _  => milisecs = &milisecs * 10
    }

    return time::Duration::from_millis(milisecs);
}