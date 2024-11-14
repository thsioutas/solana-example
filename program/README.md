# Solana Example Program

## Overview
This Solana program demonstrates a simple SOL transfer operation with persistent state tracking. The main operations are:

- **Transfer**: Transfers SOL from a payer to a recipient while recording the cumulative total amount transferred.
- **Reset**: Resets the total transferred amount (admin-only).

## Instructions

### `Transfer { amount }`
Transfers SOL from the `payer` to a `recipient`, updating the cumulative total in the `state_account`.

### `Reset`
Resets the cumulative transferred amount to zero. Requires the transaction to be signed by an admin account.

## Account Structure

### `TransferState` (Persistent State)
This struct holds the `total_transferred` field, which tracks the cumulative SOL amount transferred via the program.

### Accounts
1. **Payer**: The account providing funds for the transfer and potential rent costs.
2. **Recipient**: The account receiving SOL.
3. **State Account**: Holds the `TransferState` struct to persistently track the cumulative total.
4. **System Program**: Used for creating accounts and handling SOL transfers.

## Error Handling
- **InsufficientFunds**: Thrown when the payer's account has insufficient lamports.
- **MissingRequiredSignature**: Occurs when an unauthorized account attempts a reset operation.

## Deployment and Testing
1. Build and deploy the program with
```
cargo build-sbf
solana program deploy target/deploy/solana-example-program.so
```
2. Use the available [test client](../client/README.md) to test the program