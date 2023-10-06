# Statistica Verbōrum Latīna

[![CI](https://github.com/bakkdoor/svl/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/bakkdoor/svl/actions/workflows/ci.yml)


This repo implements various algorithms to analyze Latin texts found on [thelatinlibrary.com](https://thelatinlibrary.com) and produce statistics about them.
It uses [CozoDB](https://www.cozodb.org/) for storage and querying (using its Datalog dialect) of the textual data as well as [Rust](https://www.rust-lang.org/) for the overall implementation.

It is a work in progress.


## Build

Make sure you have Rust installed. If not, you can install it from [here](https://www.rust-lang.org/tools/install).

Then, clone this repository and build the project:

```bash
git clone https://github.com/bakkdoor/svl.git
cd svl
cargo build --release
```

The executable will be in the `./target/release` directory.

## Installation

You can install svl using cargo from this directory with:

```bash
cargo install --path .
```

Or directly (without the need for cloning the repository) with:

```bash
cargo install --git https://github.com/bakkdoor/svl.git
```

## Usage

You can run the program with:

```bash
./target/release/svl --help
```

CozoDB is used to store the data using the rocksdb storage backend.
### Create Cozo Graph DB with schema

```bash
svl create-db
```

### Import texts from [thelatinlibrary.com](https://thelatinlibrary.com)


```bash
svl import-library
```

### Run REPL to query DB interactively via CLI

```bash
svl repl
```
