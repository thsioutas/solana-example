# Solana Example Program Test Client

This Rust-based client application is designed to interact with the [Solana Example Program](../program/README.md).
It allows users to send two main types of instructions to the program: transferring SOL between accounts and resetting the cumulative transfer amount.

## Overview

The client performs the following actions based on user input:

- **Transfer**: Transfers a specified amount of SOL from a payer account to a recipient account and updates the cumulative transfer total in the state account.
- **Reset**: Resets the cumulative transfer total to zero. This is an admin-only action.

## Configuration

This client requires several keypair files to identify the relevant accounts:

- **Program Keypair**: Identifies the deployed Solana program.
- **Payer Keypair**: Identifies the payer account, which funds transfers and account creations.
- **State Account Keypair**: Stores the cumulative transfer state, created if it does not exist.

## Client Arguments

| Argument        | Description                                                           | Default Value                                                  | Required |
|-----------------|-----------------------------------------------------------------------|----------------------------------------------------------------|----------|
| `--program`     | Path to the program’s keypair file.                                   | `../program/target/deploy/solana_example_program-keypair.json` | No       |
| `--payer`       | Path to the payer’s keypair file.                                     | `~/.config/solana/payer-keypair.json`                          | No       |
| `--state_account` | Path to the state account’s keypair file.                           | `~/.config/solana/solana-example-state-keypair.json`           | No       |
| `--transfer <amount>` | Specifies the amount of SOL to transfer.                        | N/A                                                            | Optional (Mutually exclusive with `--reset`) |
| `--reset`       | Resets the total transferred amount to zero.                          | N/A                                                            | Optional (Mutually exclusive with `--transfer`) |

- **Note**: Either `--transfer <amount>` or `--reset` must be specified, but not both.
