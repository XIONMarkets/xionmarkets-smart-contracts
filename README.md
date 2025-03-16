# XIONMarkets Contracts

The smart contracts repository for XIONMarkets

# Setup Guide

- Install Rust
- Install cargo-contract
- Install xiond cli
- Install Docker Engine

After the above steps must have been completed, generate a wallet via xiond thus:

```bash
xiond keys add key_name
```

Get the displayed mnemonic and store it safely (it cannot be recovered after the terminal is closed).

# Getting Started

First of all, navigate to the project root and run:

```bash
bash compile.sh
```
Wait for the Docker rust-optimizer image to compile the contracts to a very optimized and compressed version (it will take a while) and then proceed to deploy and instantiate the contracts.

Navigate to the 'scripts' folder and run these commands in order:

## Deploy Market contract and get Code ID

```bash
bash deploy_market.sh
```
Get the tx hash and query the transaction for the code id via the explorer.

## Deploy USDC contract and get Code ID

```bash
bash deploy_cw20.sh
```
Get the tx hash and query the transaction for the code id via the explorer.

## Deploy Factory contract and get Code ID

```bash
bash deploy_factory.sh
```
Get the tx hash and query the transaction for the code id via the explorer.

## Instantiate USDC contract and get contract address

Make sure to copy the code ID associated with each deployment for use (replacement) within the code (CODE_ID).

```bash
bash instantiate_cw20.sh
```
Get the tx hash and query the transaction for the contract address.

## Instantiate Factory and get contract address

Use the Factory Code ID to instantiate the factory, setting the `fees_address`, `market_code_id` and the `usdc_address` entries in the initialization code.

```bash
bash instantiate_factory.sh
```
Get the tx hash and query the transaction for the contract address.

# Deployed Contract Addresses on xion-testnet-1

- Factory: `xion1t3c2daahluryrf66ec47fjasfp7xtnrfm9f0xdkz4jenkmaxfk6sx20kkk`
- USDC: `xion1kgq2pmddmwxqsz8rdqp4rzg6uvt6dkwg3hr423psvprd65tgrukqqf9y6r`