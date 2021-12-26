use std::collections::HashMap;
use serde::Deserialize;

pub struct CMCService {
    pub api_key: String,
}

#[derive(Deserialize, Debug)]
struct CMCResponse {
    data: Vec<CMCResponseData>,
}

#[derive(Deserialize, Debug)]
struct CMCResponseData {
    symbol: String,
    quote: HashMap<String, CMCResponseDataPrice>,
}

#[derive(Deserialize, Debug)]
struct CMCResponseDataPrice {
    price: f32,
}

impl CMCService {
    pub async fn get_btc_price(&self) -> f32 {
        let client = reqwest::Client::builder().build().unwrap();
        let resp = client.get("https://pro-api.coinmarketcap.com/v1/cryptocurrency/listings/latest")
            .header("X-CMC_PRO_API_KEY".to_string(), &self.api_key).send().await.unwrap();
        let resp = resp
            .json::<CMCResponse>()
            .await.unwrap();

        let price: f32 = resp.data.iter().filter(|it|
            it.symbol == "BTC".to_string()).map(|it| it.quote.get("USD").unwrap().price).next().unwrap();

        price
    }
}