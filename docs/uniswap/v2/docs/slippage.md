# Understanding Slippage in Uniswap V2

Slippage is the difference between the expected price of a trade and the actual executed price.

## Why Slippage Occurs

1. **Block Time Delay**: Time between transaction submission and execution
2. **Large Trade Size**: Bigger trades cause more price impact
3. **Low Liquidity**: Small pools have higher slippage

## Handling Slippage

### Setting Slippage Tolerance
```solidity
// Example with 0.5% slippage tolerance
uint amountOutMin = amountOut * 995 / 1000;
```

### Best Practices

1. **Check Current Price**
   ```solidity
   function getAmountsOut(uint amountIn, address[] memory path)
       returns (uint[] memory amounts);
   ```

2. **Use Deadline Parameter**
   ```solidity
   uint deadline = block.timestamp + 15 minutes;
   ```

3. **Monitor Gas Prices**
   - Higher gas prices = faster execution
   - Lower slippage risk
