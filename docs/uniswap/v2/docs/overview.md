# Uniswap V2 Overview

Uniswap V2 is an automated market maker (AMM) protocol that enables decentralized token swaps on Ethereum.

## Key Features

- **Constant Product Formula**: x * y = k
- **Flash Swaps**: Borrow and repay in same transaction
- **Price Oracles**: Time-weighted average price (TWAP)

## Core Contracts

The main contracts in Uniswap V2 are:

1. **UniswapV2Factory**
   - Creates new pairs
   - Manages pair addresses
   - Handles protocol fees

2. **UniswapV2Pair**
   - Holds token reserves
   - Executes swaps
   - Manages liquidity

3. **UniswapV2Router**
   - User-friendly interface
   - Handles multi-hop swaps
   - Safety checks for slippage

## Common Operations

### Adding Liquidity
```solidity
function addLiquidity(
    address tokenA,
    address tokenB,
    uint amountADesired,
    uint amountBDesired,
    uint amountAMin,
    uint amountBMin,
    address to,
    uint deadline
) returns (uint amountA, uint amountB, uint liquidity);
```

### Performing Swaps
```solidity
function swapExactTokensForTokens(
    uint amountIn,
    uint amountOutMin,
    address[] calldata path,
    address to,
    uint deadline
) returns (uint[] memory amounts);
```
