# IOTA Wallet TUI

A terminal user interface for the [IOTA](https://iota.org) network. Provides an interactive wallet for managing keys, coins, objects, transactions, and more — all from your terminal.

Built with [Ratatui](https://ratatui.rs) and the [IOTA Rust SDK](https://github.com/iotaledger/iota-rust-sdk).

## Features

- **Key management** — Generate, import, rename, and toggle visibility of Ed25519/Secp256k1/Secp256r1 keys
- **Coins & objects** — View coins and objects across all visible addresses with search/filter
- **Multi-account portfolio** — Aggregated balance view across multiple addresses
- **Coin management** — Merge, split, and quick-transfer coins directly from the Coins tab
- **Staking dashboard** — View staked IOTA, validator info, and unstake with one key
- **Transaction builder** — Compose programmable transaction blocks (transfer, split, merge, stake, move call) with dry-run simulation
- **Transaction detail** — Inspect transactions with full gas breakdown
- **Activity feed** — Live transaction polling with real-time updates
- **Package browser** — Explore on-chain packages, drill into modules and functions, and jump to move calls
- **Address book** — Save addresses with labels and notes; resolve IOTA-Names automatically
- **Network explorer** — Browse network overview, checkpoints, validators, and look up objects/addresses by type
- **Network switching** — Connect to mainnet, testnet, or devnet with persisted selection
- **Faucet** — Request test tokens on testnet/devnet
- **Clipboard & export** — Copy selected items or export lists as CSV
- **Collapsible sidebar** — Auto-collapsing navigation with hover reveal and keyboard toggle
- **Dynamic theming** — Pastel color palette with sparkle effects (hidden toggle)
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
View coin balances across visible addresses with filtering, merge/split, and quick transfer
![Coins page](/screenshots/coins_page.png)

### Objects
Browse owned objects with type search and filtering
![Objects Page](/screenshots/objects_page.png)

### Transactions
View transaction history with search/filter and detail pane
![Transactions Page](/screenshots/transactions_page.png)

### Staking
View staked IOTA balances, validator info, and unstake directly
![Staking Page](/screenshots/staking_page.png)

### Packages
Browse on-chain packages, explore modules and functions
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

### Activity Feed
Live transaction feed with automatic polling

## Keybindings

### Navigation

| Key           | Action                          |
|---------------|---------------------------------|
| `1`–`0`       | Switch to screen (1=Coins … 0=Activity) |
| `Tab`          | Toggle sidebar                  |
| `Up`/`Down`   | Navigate lists                  |
| `Left`/`Right` | Switch Tx Builder steps / package drill-down |
| `Enter`       | Explore item / confirm          |
| `Esc`         | Cancel / close popup / collapse sidebar |

### Actions

| Key        | Action                              |
|------------|-------------------------------------|
| `a`        | Add entry (address, command)        |
| `e`        | Edit entry / rename key / explore package |
| `d`/`Del`  | Delete entry                        |
| `/`        | Filter list (Coins/Objects/Transactions) |
| `t`        | Search objects by type (Coins/Objects) |
| `x`        | Quick transfer (Coins) / explore address (Keys) |
| `m`        | Merge coins                         |
| `s`        | Split coin / type search (Explorer) |
| `u`        | Unstake (Staking)                   |
| `p`        | Portfolio view (Coins) / toggle private key (Keys) |
| `c`        | Copy selected / clear Tx Builder    |
| `C`        | Export CSV                          |
| `g`        | Generate new key                    |
| `i`        | Import key                          |
| `Space`    | Toggle key visibility               |
| `n`        | Switch network                      |
| `r`        | Refresh data from network           |
| `f`        | Request faucet tokens               |
| `.`        | Actions menu                        |
| `E`        | View error log                      |
| `?`        | Help                                |
| `q`/`Ctrl-c` | Quit                             |

Mouse click and scroll are supported for the sidebar, lists, and popups.

## License

MIT
