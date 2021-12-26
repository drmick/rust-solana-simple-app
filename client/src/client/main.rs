use index_client as ic;

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() != 2 {
        eprintln!("invalid arguments. Required path to program keypair and CMC API KEY");
        std::process::exit(-1);
    }
    let keypair_path = &args[1];

    let connection = ic::sol_client::establish_connection().unwrap();

    let player = ic::utils::get_player().unwrap();

    let program = ic::sol_client::get_program(keypair_path, &connection).unwrap();

    println!(
        "{} is average BTC price in USD.",
        ic::sol_client::get_average_price(&player, &program, &connection).unwrap()
    )
}
