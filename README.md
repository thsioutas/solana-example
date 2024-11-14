# Solana Example Program

This repository contains a [Solana example program](program/program_readme.md) and a [test client](client/README.md).
The example program implements a simple token transfer logic where users can transfer SOL between accounts
and track the total transferred amount in a state account. The program also allows an admin to reset the total transferred amount.

The test client is a command-line tool that interacts with the deployed program, allowing users to perform transfers or reset the transferred amount.

## Project Structure

- `program/`: Contains the Solana smart contract (on-chain program) written in Rust.
- `client/`: Contains the test client written in Rust, which interacts with the program.

## Solana Program

The Solana example program implements a basic logic for transferring SOL and tracking the total amount transferred.
The program also includes a function to reset the transferred amount, which is only accessible by the program's admin.

### How to Deploy the Program

1. **Build the Program**:
    ```bash
    cd program
    cargo build-sbf
    ```

2. **Deploy the Program**:
    ```bash
    solana program deploy dist/solana_example_program.so
    ```

3. After deployment, you'll receive the program's public key, which you'll use in the test client.

## Test Client

The test client allows you to interact with the deployed program by sending transactions to transfer SOL between accounts or reset the transferred amount.

You can find detailed instructions on how to use the client in the [client_readme.md](client/README.md).

### How to Run the Test Client

1. **Build the client**:
    ```bash
    cargo build --release
    ```

2. **Run the Client**:
    You can use the test client to either transfer SOL or reset the transferred amount.

    - **Transfer SOL**:
        ```bash
        cargo run -- --transfer <amount> --program <program_keypair_file> --payer <payer_keypair_file> --state_account <state_account_keypair_file>
        ```

    - **Reset the Total Transferred Amount**:
        ```bash
        cargo run -- --reset --program <program_keypair_file> --payer <payer_keypair_file> --state_account <state_account_keypair_file>
        ```