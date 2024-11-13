use clap::Parser;
use solana_client::rpc_client::RpcClient;
use solana_example_program::SolanaExampleInstruction;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    signature::{read_keypair_file, Keypair, Signer},
    system_program,
    transaction::Transaction,
};
use std::io::{Read, Write};

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

    /// The file containing the state account's keypair.
    #[arg(long, default_value_t = state_account_default())]
    state_account: String,

    #[command(flatten)]
    pay_or_reset: PayOrReset,
}

#[derive(Parser, Debug)]
#[group(required = true, multiple = false)]
struct PayOrReset {
    /// Transfer the specified amount
    #[arg(short, long)]
    transfer: Option<u64>,

    /// Reset the total transferred amount
    #[arg(short, long)]
    reset: bool,
}

fn payer_default() -> String {
    let mut home_dir = dirs::home_dir().unwrap();
    home_dir.push(".config/solana/payer-keypair.json");
    home_dir.to_str().unwrap().to_string()
}

fn state_account_default() -> String {
    let mut home_dir = dirs::home_dir().unwrap();
    home_dir.push(".config/solana/solana-example-state-keypair.json");
    home_dir.to_str().unwrap().to_string()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    // Set up Solana RPC client to talk to Localnet
    let client = RpcClient::new("http://localhost:8899".to_string());

    let program = read_keypair_file(args.program)?;
    let payer = read_keypair_file(args.payer)?;
    let recipient = Keypair::new();

    let state_account_keypair_path = args.state_account;
    let state_account = if std::path::PathBuf::from(&state_account_keypair_path).exists() {
        let mut file = std::fs::File::open(state_account_keypair_path)?;
        let mut keypair_bytes = Vec::new();
        file.read_to_end(&mut keypair_bytes)?;
        Keypair::from_bytes(&keypair_bytes).unwrap()
    } else {
        println!(
            "Generate new state account keypair and store in: {:?}",
            state_account_keypair_path
        );
        let state_account = Keypair::new();
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(state_account_keypair_path)?;
        file.write_all(&state_account.to_bytes()).unwrap();
        state_account
    };
    let system_account = system_program::ID;
    // Fund the payer account on localnet
    client.request_airdrop(&payer.pubkey(), 1_000_000_000)?;

    if let Some(amount) = args.pay_or_reset.transfer {
        let data = SolanaExampleInstruction::Transfer { amount };
        println!(
            "Prepare and send a new sol_transfer instruction with data = {:?}",
            data
        );
        let sol_transfer_ix = Instruction::new_with_borsh(
            program.pubkey(),
            &data,
            vec![
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new(recipient.pubkey(), true),
                AccountMeta::new(state_account.pubkey(), true),
                AccountMeta::new_readonly(system_account, false),
            ],
        );

        let recent_blockhash = client.get_latest_blockhash()?;
        let mut transaction =
            Transaction::new_with_payer(&[sol_transfer_ix], Some(&payer.pubkey()));
        transaction.sign(&[&payer, &recipient, &state_account], recent_blockhash);
        client.send_and_confirm_transaction(&transaction)?;
    } else {
        let data = SolanaExampleInstruction::Reset;
        let reset_transferred_ix = Instruction::new_with_borsh(
            program.pubkey(),
            &data,
            vec![
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new(state_account.pubkey(), true),
            ],
        );
        let recent_blockhash = client.get_latest_blockhash()?;
        let mut transaction =
            Transaction::new_with_payer(&[reset_transferred_ix], Some(&payer.pubkey()));
        transaction.sign(&[&payer, &state_account], recent_blockhash);
        client.send_and_confirm_transaction(&transaction)?;
    }

    Ok(())
}
