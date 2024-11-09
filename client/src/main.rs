use borsh::BorshSerialize;
use clap::Parser;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    signature::{read_keypair_file, Keypair, Signer},
    system_program,
    transaction::Transaction,
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The file containing the program's keypair.
    #[arg(
        long,
        default_value = "../program/target/deploy/solana_example_program-keypair.json"
    )]
    program: String,

    /// The file containing the payer's keypair.
    #[arg(long, default_value_t = payer_default())]
    payer: String,
}

fn payer_default() -> String {
    let mut home_dir = dirs::home_dir().unwrap();
    home_dir.push(".config/solana/payer-keypair.json");
    home_dir.to_str().unwrap().to_string()
}

#[derive(BorshSerialize, Debug)]
struct TransferInstruction {
    amount: u64,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    // Set up Solana RPC client to talk to Localnet
    let client = RpcClient::new("http://localhost:8899".to_string());

    // Program ID of the deployed program
    let program_id = read_keypair_file(args.program)?.pubkey();
    let payer = read_keypair_file(args.payer)?;
    let recipient = Keypair::new();
    let state_account = Keypair::new();
    let system_account = system_program::ID;

        // Fund the payer account on localnet
        client.request_airdrop(&payer.pubkey(), 1_000_000_000)?;

    let data = TransferInstruction { amount: 2 };
    // Create and send the "sol_transfer" transaction
    let sol_transfer_ix = Instruction::new_with_borsh(
        program_id,
        &data,
        vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(recipient.pubkey(), true),
            AccountMeta::new(state_account.pubkey(), true),
            AccountMeta::new_readonly(system_account, false),
        ],
    );

    let recent_blockhash = client.get_latest_blockhash()?;
    let mut transaction = Transaction::new_with_payer(&[sol_transfer_ix], Some(&payer.pubkey()));
    transaction.sign(&[&payer, &recipient, &state_account], recent_blockhash);
    client.send_and_confirm_transaction(&transaction)?;
    Ok(())
}
