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

### Coins
View coin balances across visible addresses
![Coins page](/screenshots/coins_page.png)

### Objects
Browse owned objects
![Objects Page](/screenshots/objects_page.png)

### Transactions
View transaction history

![Transactions Page](/screenshots/transactions_page.png)

### Packages
Package explorer (placeholder)

![Packages Page](/screenshots/packages_page.png)

### Address Book
Manage saved addresses and IOTA-Names

![Address Book Page](/screenshots/address_book_page.png)

### Keys
Manage cryptographic keys

![Keys Page](/screenshots/keys_page.png)

### Tx Builder
Build and execute programmable transactions

![Tx Builder Page](/screenshots/tx_builder_page.png)

### Explorer
Browse network state: overview, checkpoints, validators, and lookup

## Keybindings

### Navigation

| Key          | Action                          |
|--------------|---------------------------------|
| `1`–`8`      | Switch to screen                |
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

## License

MIT
