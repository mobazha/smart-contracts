# Mobazha Escrow Program

A secure, time-locked, multi-signature escrow program built on Solana using the Anchor framework.

## Features

- **SOL and SPL Token Support**: Escrow both native SOL and any SPL token
- **Multi-signature Release**: Configurable number of required signatures to release funds
- **Time-lock Mechanism**: Funds can be locked for a specified period
- **Optional Moderator**: Add a third-party moderator for dispute resolution
- **Multiple Recipients**: Support for distributing funds to up to 4 recipients

## Architecture

The program consists of four main instructions:

1. `initialize_sol`: Create a new SOL escrow account
2. `release_sol`: Release SOL from escrow to recipients
3. `initialize_token`: Create a new SPL token escrow account
4. `release_token`: Release tokens from escrow to recipients

## Security Features

- Ed25519 signature verification for secure multi-signature release
- PDA (Program Derived Address) accounts for secure fund storage
- Comprehensive validation checks throughout the program


## Usage

See the `tests/` directory for example usage.
