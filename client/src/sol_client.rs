use borsh::{BorshDeserialize, BorshSerialize};
use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::instruction::{AccountMeta, Instruction};
use solana_sdk::message::Message;
use solana_sdk::signature::Signer;
use solana_sdk::signer::keypair::{Keypair, read_keypair_file};
use solana_sdk::transaction::Transaction;

use crate::{Error, Result};
use crate::utils;

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Price {
    value: f32,
}

/// Establishes a RPC connection with the solana cluster configured by
/// `solana config set --url <URL>`. Information about what cluster
/// has been configured is gleened from the solana config file
/// `~/.config/solana/cli/config.yml`.
pub fn establish_connection() -> Result<RpcClient> {
    let rpc_url = utils::get_rpc_url()?;
    Ok(RpcClient::new_with_commitment(
        rpc_url,
        CommitmentConfig::confirmed(),
    ))
}

/// Determines the amount of lamports that will be required to execute
/// this smart contract. The minimum balance is calculated assuming
/// that the user would like to make their account rent exempt.
/// TODO (Need to refactoring)
#[allow(deprecated)]
pub fn get_balance_requirement(connection: &RpcClient) -> Result<u64> {
    let account_fee =
        connection.get_minimum_balance_for_rent_exemption(utils::get_data_size()?)?;

    let (_, fee_calculator) = connection.get_recent_blockhash()?;
    let transaction_fee = fee_calculator.lamports_per_signature * 100;

    Ok(transaction_fee + account_fee)
}

/// Gets the balance of PLAYER in lamports via a RPC call over
pub fn get_player_balance(player: &Keypair, connection: &RpcClient) -> Result<u64> {
    Ok(connection.get_balance(&player.pubkey())?)
}

/// Requests that AMOUNT lamports are transfered to PLAYER via a RPC
/// call over CONNECTION.
pub fn request_airdrop(player: &Keypair, connection: &RpcClient, amount: u64) -> Result<()> {
    let sig = connection.request_airdrop(&player.pubkey(), amount)?;
    loop {
        let confirmed = connection.confirm_transaction(&sig)?;
        if confirmed {
            break;
        }
    }
    Ok(())
}

/// Loads keypair information from the file located at KEYPAIR_PATH
/// and then verifies that the loaded keypair information corresponds
/// to an executable account via CONNECTION. Failure to read the
/// keypair or the loaded keypair corresponding to an executable
/// account will result in an error being returned.
pub fn get_program(keypair_path: &str, connection: &RpcClient) -> Result<Keypair> {
    let program_keypair = read_keypair_file(keypair_path).map_err(|e| {
        Error::InvalidConfig(format!(
            "failed to read program keypair file ({}): ({})",
            keypair_path, e
        ))
    })?;

    let program_info = connection.get_account(&program_keypair.pubkey())?;
    if !program_info.executable {
        return Err(Error::InvalidConfig(format!(
            "program with keypair ({}) is not executable",
            keypair_path
        )));
    }
    Ok(program_keypair)
}

pub fn create_account(
    player: &Keypair,
    program: &Keypair,
    connection: &RpcClient,
) -> Result<()> {
    let index_pubkey = utils::get_public_key(&player.pubkey(), &program.pubkey())?;

    if let Err(_) = connection.get_account(&index_pubkey) {
        let lamport_requirement =
            connection.get_minimum_balance_for_rent_exemption(utils::get_data_size()?)?;

        let instruction = solana_sdk::system_instruction::create_account_with_seed(
            &player.pubkey(),
            &index_pubkey,
            &player.pubkey(),
            utils::get_index_seed(),
            lamport_requirement,
            utils::get_data_size()? as u64,
            &program.pubkey(),
        );
        let message = Message::new(&[instruction], Some(&player.pubkey()));
        let transaction =
            Transaction::new(&[player], message, connection.get_latest_blockhash()?);

        connection.send_and_confirm_transaction(&transaction)?;
    }

    Ok(())
}

/// Sends an instruction from PLAYER to PROGRAM via CONNECTION. The
/// instruction contains no data but does contain the address of our
/// previously generated index account. The program will use that
/// passed in address to add price after verifying
/// that it owns the account that we have passed in.
pub fn send_btc_price(player: &Keypair, program: &Keypair, connection: &RpcClient, price: f32) -> Result<()> {
    let index_pubkey = utils::get_public_key(&player.pubkey(), &program.pubkey())?;
    let price = Price { value: price };
    let price = &price.try_to_vec().unwrap();

    let instruction = Instruction::new_with_bytes(
        program.pubkey(),
        price,
        vec![AccountMeta::new(index_pubkey, false)],
    );
    let message = Message::new(&[instruction], Some(&player.pubkey()));
    let transaction = Transaction::new(&[player],
                                       message,
                                       connection.get_latest_blockhash()?);
    connection.send_and_confirm_transaction(&transaction)?;
    Ok(())
}

pub fn get_average_price(player: &Keypair, program: &Keypair, connection: &RpcClient) -> Result<f32> {
    let index_pubkey = utils::get_public_key(&player.pubkey(), &program.pubkey())?;
    let index_account = connection.get_account(&index_pubkey)?;
    Ok(utils::get_average_price(&index_account.data)?)
}
