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
entrypoint!(process_instruction);

#[derive(BorshDeserialize, Debug)]
struct TransferInstruction {
    amount: u64,
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
    let transfer_instruction = TransferInstruction::try_from_slice(instruction_data)?;
    sol_transfer(accounts, transfer_instruction, program_id)
}

/// Ensures that the state account is created and initialized if it doesn't already exist.
fn ensure_state_account<'a>(
    payer: &AccountInfo<'a>,
    state_account: &AccountInfo<'a>,
    program_id: &Pubkey,
    system_program: &AccountInfo<'a>,
) -> ProgramResult {
    if state_account.lamports() == 0 {
        // The extra +8 bytes are needed because, in Solana, account data includes a Borsh serialization prefix.
        // Borsh, the serialization format used here, stores serialized data with a length prefix that indicates
        // the size of the data. This prefix is typically 8 bytes long for larger structures like TransferState.
        let account_space = std::mem::size_of::<TransferState>() + 8;
        // Calculate the minimum lamports required for rent exemption
        let rent_exempt_balance =  Rent::get()?.minimum_balance(account_space);

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
        state.serialize(&mut *state_account.data.borrow_mut())?;

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
        let rent_exempt_balance =  Rent::get()?.minimum_balance(account_space);

        // Create the recipient account as a system account
        invoke(
            &system_instruction::create_account(
                payer.key,
                recipient.key,
                rent_exempt_balance,
                account_space as u64,
                &system_program.key,
            ),
            &[payer.clone(), recipient.clone(), system_program.clone()],
        )?;
        msg!("Recipient account created");
    } else {
        msg!("Recipeint account already exists");
    }
    Ok(())
}

fn sol_transfer(
    accounts: &[AccountInfo],
    instruction: TransferInstruction,
    program_id: &Pubkey,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let payer = next_account_info(accounts_iter)?;
    let recipient = next_account_info(accounts_iter)?;
    let state_account = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;

    // msg!("Start {} SOL transfer from {} to {}",  instruction.amount, payer.key, recipient.key,);

    // Ensure the recipient account exists
    ensure_recipient_account(payer, recipient, system_program)?;

    // Ensure the state account is created and initialized
    ensure_state_account(payer, state_account, program_id, system_program)?;

    // Check if the payer has enough funds for the transfer
    if payer.lamports() < instruction.amount {
        msg!("Insufficient funds");
        return Err(ProgramError::InsufficientFunds);
    }

    // Check if the source account (payer) is a signer
    if !payer.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    msg!(
        "Transferring {} lamports from {} to {}",
        instruction.amount,
        payer.key,
        recipient.key
    );

    invoke(
        &system_instruction::transfer(payer.key, recipient.key, instruction.amount),
        &[payer.clone(), recipient.clone(), system_program.clone()],
    )?;

    msg!("Transfer successful");

    let mut state_data = state_account.try_borrow_mut_data()?;
    let mut state = TransferState::try_from_slice(&state_data)?;
    state.total_transferred += instruction.amount;
    state.serialize(&mut *state_data)?;

    msg!(
        "State updated successfully, total_transferred: {}",
        state.total_transferred
    );
    Ok(())
}
