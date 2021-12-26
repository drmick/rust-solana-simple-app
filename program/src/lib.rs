use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{account_info::{AccountInfo, next_account_info}, entrypoint, msg, program_error::ProgramError, pubkey::Pubkey};

#[derive(BorshSerialize, BorshDeserialize, Debug, Default)]
pub struct AppState {
    pub prices: [StoredPrice; 5],
    pub average_price: f32,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Default, Copy, Clone)]
pub struct StoredPrice {
    pub price: f32,
    pub is_some: bool,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct IncomingPrice {
    pub price: f32,
}

#[cfg(not(feature = "exclude_entrypoint"))]
entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> entrypoint::ProgramResult {
    // Get the account that stores prices count information.
    let accounts_iter = &mut accounts.iter();

    let account = next_account_info(accounts_iter)?;

    if account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }
    let incoming_price: IncomingPrice = IncomingPrice::try_from_slice(instruction_data)?;

    let mut app_state: AppState = AppState::try_from_slice(&account.data.borrow())?;
    app_state.prices.rotate_right(1);
    app_state.prices[0] = StoredPrice { price: incoming_price.price, is_some: true };
    app_state.average_price = average(&app_state.prices);

    if app_state.average_price.is_nan() {
        //prices not found
        return Err(ProgramError::Custom(1))
    }
    msg!("{:?}", app_state);
    app_state.serialize(&mut &mut account.data.borrow_mut()[..])?;
    Ok(())
}

fn average(arr: &[StoredPrice]) -> f32 {
    arr.iter().filter(|it| it.is_some).map(|it| it.price).
        sum::<f32>() as f32 / arr.iter().filter(|it| it.is_some).count() as f32
}

#[cfg(test)]
mod test {
    use super::*;
    use solana_program::clock::Epoch;

    #[test]
    fn test_average() {
        let program_id = Pubkey::default();
        let key = Pubkey::default();
        let mut lamports = 0;
        let mut data = AppState::default().try_to_vec().unwrap();
        let owner = Pubkey::default();
        let account = AccountInfo::new(
            &key,
            false,
            true,
            &mut lamports,
            &mut data,
            &owner,
            false,
            Epoch::default(),
        );

        let accounts = vec![account];

        let instruction_data = IncomingPrice{price: 10.0_f32}.try_to_vec().unwrap();

        process_instruction(&program_id, &accounts, &instruction_data).unwrap();
        assert_eq!(
            AppState::try_from_slice(&accounts[0].data.borrow())
                .unwrap().average_price,
            10.0_f32,
        );
        let instruction_data = IncomingPrice{price: 20.0_f32}.try_to_vec().unwrap();

        process_instruction(&program_id, &accounts, &instruction_data).unwrap();
        assert_eq!(
            AppState::try_from_slice(&accounts[0].data.borrow())
                .unwrap().average_price,
            15.0_f32,
        );
    }
}
