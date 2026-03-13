## Goals

This project is a terminal UI for the `iota` SDK. It provides an easy-to-use interface for developers, and an alternative to the `iota` CLI. It should:
1. Be usable entirely with both mouse and keyboard
2. Store and fetch local data so that manual copy and pasting is not necessary
3. Present data in a consise and readable way, with the ability to see more detailed data
4. Allow viewing all relevant data to an address in one place
5. Make use of the various services on the IOTA network which enable simpler usage:
    - IOTA-Names
    - IOTA package manager (future project)

## Formatting

Code in this repo should be formatted using the latest nightly formatter.

```
cargo +nightly fmt
```
