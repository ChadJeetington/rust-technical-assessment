# Uniswap V2 Core Concepts

## Automated Market Maker (AMM)

Uniswap V2 uses an automated market maker mechanism called the "Constant Product Formula":

```
x * y = k
```

Where:
- x is the reserve of token A
- y is the reserve of token B
- k is a constant that must be maintained

## Key Features

### 1. Flash Swaps
Flash swaps allow you to withdraw up to the full amount of any ERC20 token on Uniswap and execute arbitrary logic before paying for it at the end of the transaction.

### 2. Price Oracles
Uniswap v2 implements price oracles that track the time-weighted average price (TWAP) of a pair. This provides a more manipulation-resistant price feed.

### 3. Protocol Fee
A 0.05% protocol fee can be turned on by UNI governance, which is taken from the 0.30% trading fee.

## Liquidity Providers

Liquidity providers (LPs) deposit pairs of tokens into Uniswap pools. In return, they receive:
- LP tokens representing their share of the pool
- Trading fees proportional to their share
- Exposure to impermanent loss
