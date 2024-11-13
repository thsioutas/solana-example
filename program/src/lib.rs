use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program::invoke,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::Sysvar,
};
use std::io::Write;

entrypoint!(process_instruction);

#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub enum SolanaExampleInstruction {
    Transfer { amount: u64 },
    Reset,
}

/// Persistent state structure to track total transferred SOL.
#[derive(BorshDeserialize, BorshSerialize, Debug)]
struct TransferState {
    total_transferred: u64,
}

fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    use SolanaExampleInstruction::*;
    let transfer_instruction = SolanaExampleInstruction::try_from_slice(instruction_data)?;
    msg!("Instruction = {:?}", transfer_instruction);
    match transfer_instruction {
        Transfer { amount } => sol_transfer(accounts, amount, program_id),
        Reset => reset_total_transferred(accounts),
    }
}

/// Ensures that the state account is created and initialized if it doesn't already exist.
fn ensure_state_account<'a>(
    payer: &AccountInfo<'a>,
    state_account: &AccountInfo<'a>,
    program_id: &Pubkey,
    system_program: &AccountInfo<'a>,
) -> ProgramResult {
    if state_account.lamports() == 0 {
        let account_space = std::mem::size_of::<TransferState>();
        // Calculate the minimum lamports required for rent exemption
        let rent_exempt_balance = Rent::get()?.minimum_balance(account_space);

        // Create the state account with the minimum required lamports
        msg!("Creating state account for tracking total transferred SOL");
        invoke(
            &system_instruction::create_account(
                payer.key,
                state_account.key,
                rent_exempt_balance,
                account_space as u64,
                program_id,
            ),
            &[payer.clone(), state_account.clone(), system_program.clone()],
        )?;

        // Initialize the state account data
        let state = TransferState {
            total_transferred: 0,
        };

        let mut serialized_data: Vec<u8> = Vec::new();
        state.serialize(&mut serialized_data).unwrap();
        let mut state_data = state_account.try_borrow_mut_data()?;
        (&mut state_data[..serialized_data.len()]).write_all(&serialized_data)?;

        // TODO: Why doesn't this work?
        // state.serialize(&mut *state_account.try_borrow_mut_data()?)?;

        msg!("State account created and initialized");
    } else {
        msg!("State account already exists");
    }
    Ok(())
}

fn ensure_recipient_account<'a>(
    payer: &AccountInfo<'a>,
    recipient: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
) -> ProgramResult {
    if recipient.lamports() == 0 {
        msg!("Creating recipient account as it does not exist");
        // Size of data: 0 because it's a system account, which holds only lamports
        let account_space = 0;
        // Calculate the minimum lamports required for rent exemption
        let rent_exempt_balance = Rent::get()?.minimum_balance(account_space);

        // Create the recipient account as a system account
        invoke(
            &system_instruction::create_account(
                payer.key,
                recipient.key,
                rent_exempt_balance,
                account_space as u64,
                system_program.key,
            ),
            &[payer.clone(), recipient.clone(), system_program.clone()],
        )?;
        msg!("Recipient account created");
    } else {
        msg!("Recipeint account already exists");
    }
    Ok(())
}

fn sol_transfer(accounts: &[AccountInfo], amount: u64, program_id: &Pubkey) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let payer = next_account_info(accounts_iter)?;
    let recipient = next_account_info(accounts_iter)?;
    let state_account = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;

    // Ensure the recipient account exists
    ensure_recipient_account(payer, recipient, system_program)?;

    // Ensure the state account is created and initialized
    ensure_state_account(payer, state_account, program_id, system_program)?;

    // Check if the payer has enough funds for the transfer
    if payer.lamports() < amount {
        msg!("Insufficient funds");
        return Err(ProgramError::InsufficientFunds);
    }

    // Check if the source account (payer) is a signer
    if !payer.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    msg!(
        "Transferring {} lamports from {} to {}",
        amount,
        payer.key,
        recipient.key
    );

    invoke(
        &system_instruction::transfer(payer.key, recipient.key, amount),
        &[payer.clone(), recipient.clone(), system_program.clone()],
    )?;

    msg!("Transfer successful");
    let mut state = TransferState::try_from_slice(&state_account.try_borrow_mut_data()?)?;
    state.total_transferred += amount;
    state.serialize(&mut *state_account.data.borrow_mut())?;

    msg!(
        "State updated successfully, total_transferred: {}",
        state.total_transferred
    );
    Ok(())
}

/// Resets the total transferred amount; only the admin can do this.
fn reset_total_transferred(accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    // Retrieve the necessary accounts
    let admin = next_account_info(accounts_iter)?;
    let state_account = next_account_info(accounts_iter)?;

    // Check that the admin is a signer
    if !admin.is_signer {
        msg!("Admin must be a signer to reset the state");
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Reset the total transferred state
    let mut state_data = state_account.try_borrow_mut_data()?;
    let mut state = TransferState::try_from_slice(&state_data)?;
    state.total_transferred = 0;
    state.serialize(&mut *state_data)?;

    msg!("Total transferred amount has been reset by the admin");
    Ok(())
}
