# IOTA Wallet TUI

A terminal user interface for the [IOTA](https://iota.org) network. Provides an interactive wallet for managing keys, coins, objects, transactions, and more — all from your terminal.

Built with [Ratatui](https://ratatui.rs) and the [IOTA Rust SDK](https://github.com/iotaledger/iota-rust-sdk).

## Features

- **Key management** — Generate, import, rename, and toggle visibility of Ed25519/Secp256k1/Secp256r1 keys
- **Coins & objects** — View coins and objects across all visible addresses
- **Transaction builder** — Compose programmable transaction blocks (transfer, split, merge, stake, move call) with dry-run simulation
- **Address book** — Save addresses with labels and notes; resolve IOTA-Names automatically
- **Network switching** — Connect to mainnet, testnet, or devnet with persisted selection
- **Faucet** — Request test tokens on testnet/devnet
- **Error log** — View backend errors in-app
- **Mouse & keyboard** — Fully operable with either input method

## Installation

Requires [Rust](https://rustup.rs) (nightly toolchain for formatting).

```sh
git clone https://github.com/anthropics/iota-wallet-tui.git
cd iota-wallet-tui
cargo build --release
```

The binary will be at `target/release/iota-wallet-tui`.

## Usage

```sh
cargo run --release
```

Data is stored in `~/Library/Application Support/iota-wallet-tui/` (macOS) including keys, address book, network selection, and error logs.

## Screens

| # | Screen        | Description                                    |
|---|---------------|------------------------------------------------|
| 1 | Coins         | View coin balances across visible addresses    |
| 2 | Objects        | Browse owned objects                           |
| 3 | Transactions  | View transaction history                       |
| 4 | Packages      | Package explorer (placeholder)                 |
| 5 | Address Book  | Manage saved addresses and IOTA-Names          |
| 6 | Keys          | Manage cryptographic keys                      |
| 7 | Tx Builder    | Build and execute programmable transactions    |

## Keybindings

### Navigation

| Key          | Action                          |
|--------------|---------------------------------|
| `1`–`7`      | Switch to screen                |
| `Tab`/`S-Tab`| Cycle screens forward/backward  |
| `Up`/`Down`  | Navigate lists                  |
| `Left`/`Right`| Switch Tx Builder steps        |
| `Enter`      | Select / confirm                |
| `Esc`        | Cancel / close popup            |

### Actions

| Key     | Action                              |
|---------|-------------------------------------|
| `a`     | Add entry (address, command)        |
| `e`     | Edit entry / rename key             |
| `d`/`Del`| Delete entry                       |
| `g`     | Generate new key                    |
| `i`     | Import key                          |
| `p`     | Toggle private key display          |
| `Space` | Toggle key visibility               |
| `n`     | Switch network                      |
| `r`     | Refresh data from network           |
| `f`     | Request faucet tokens               |
| `c`     | Clear/reset transaction builder     |
| `E`     | View error log                      |
| `?`     | Help                                |
| `q`/`Ctrl-c` | Quit                           |

Mouse click and scroll are supported for tabs, lists, and popups.

## Development

```sh
# Run
cargo run

# Format (requires nightly)
cargo +nightly fmt

# Build release
cargo build --release
```

## License

MIT
