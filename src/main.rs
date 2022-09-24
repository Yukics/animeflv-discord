use reqwest;
use tokio;
use scraper;
use serde_json;
use std::{thread, time, env};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Payload {
    cmd: String,
    url: String,
    maxTimeout: i32
}

#[tokio::main]
async fn main ()-> Result<(), Box<dyn std::error::Error>> {

    // let proxy = env::var("PROXY").unwrap().to_string();
    let proxy = "http://localhost:8191/v1";

    // let animeflv_url = env::var("ANIMEFLV_URL").unwrap().to_string();
    let animeflv_url = "https://www3.animeflv.net";

    let payload = Payload {
        cmd: "request.get".into(),
        url: animeflv_url.into(),
        maxTimeout: 60000.into()
    };

    loop {

        // ? We create the werequest through flareresolver, cloudflare bypasser
        let resp = reqwest::Client::new()
            .post(proxy)
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
        
        let mut anime_list_array = Vec::new();
        let anime_list_selector = scraper::Selector::parse("main.Main ul.ListEpisodios li a").unwrap();

        let anime_name_selector = scraper::Selector::parse("strong.Title").unwrap();
        let anime_cap_selector = scraper::Selector::parse("span.Capi").unwrap();
        let anime_image_selector = scraper::Selector::parse("span.Image>img").unwrap();

        for element in document.select(&anime_list_selector) {
            let anime_link= element.value().attr("href").expect("Could not find href attribute");
            let anime_name_element = element.select(&anime_name_selector).next().expect("Could not find anime name.");
            let anime_cap_element = element.select(&anime_cap_selector).next().expect("Could not find cap number.");
            let anime_image = element.select(&anime_image_selector).next().expect("Could not find anime image.").value().attr("src").expect("Could not find anime image.");

            let anime_name = anime_name_element.text().collect::<String>();
            let anime_cap = anime_cap_element.text().collect::<String>();

            println!("{:?} {:?} {:?} {:?}",animeflv_url.to_owned() + anime_link, anime_name, anime_cap, animeflv_url.to_owned() + anime_image);
            anime_list_array.push(animeflv_url.to_owned() + anime_link);
            anime_list_array.push(anime_name);
            anime_list_array.push(anime_cap);
            anime_list_array.push(animeflv_url.to_owned() + anime_image);
        } 
        println!("{:?}", &anime_list_array[0]);

        // TODO if arrays are different get pos off the first element in old array 
        // TODO and print only from the new untill its there

        //* Sleep 1s+processing time
        let ten_millis = time::Duration::from_millis(10000);
        thread::sleep(ten_millis);
    }
}