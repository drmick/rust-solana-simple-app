use crate::external::CMCService;
use index_client as ic;
use std::time::Duration;
use tokio::time::sleep;
mod external;

#[tokio::main]
async fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() != 3 {
        eprintln!("Invalid arguments. Required path to program keypair and CMC API KEY",);
        std::process::exit(-1);
    }

    let keypair_path = &args[1];
    let cmc_api_key: &String = &args[2];

    let cmc_service = CMCService {
        api_key: cmc_api_key.to_string(),
    };

    loop {
        let price = cmc_service.get_btc_price().await;
        send_price(keypair_path, price);
        sleep(Duration::from_secs(3600)).await;
    }
}

fn send_price(keypair_path: &String, price: f32) {
    let connection = ic::sol_client::establish_connection().unwrap();

    // required balance to complete a transaction
    let balance_requirement = ic::sol_client::get_balance_requirement(&connection).unwrap();

    let player = ic::utils::get_player().unwrap();

    // current balance
    let player_balance = ic::sol_client::get_player_balance(&player, &connection).unwrap();

    if player_balance < balance_requirement {
        let request = balance_requirement - player_balance;
        ic::sol_client::request_airdrop(&player, &connection, request).unwrap();
    }

    let program = ic::sol_client::get_program(keypair_path, &connection).unwrap();

    ic::sol_client::create_account(&player, &program, &connection).unwrap();

    ic::sol_client::send_btc_price(&player, &program, &connection, price).unwrap();
    println!("Btc price {} successfully sent", price)
}
