# Flash Swaps in Uniswap V2

Flash swaps are a powerful feature that allows you to withdraw tokens from a Uniswap pair and use them before paying for them.

## How Flash Swaps Work

1. **Withdraw Tokens**: Call `swap()` with `data` parameter
2. **Execute Logic**: Your contract receives tokens and can use them
3. **Pay Back**: Return tokens by end of transaction

## Example Usage

```solidity
// Flash swap 100 WETH, use it, then pay it back
function flashSwap(address pair, uint amount) external {
    // Withdraw WETH
    IUniswapV2Pair(pair).swap(
        amount,  // amount0Out (WETH)
        0,       // amount1Out
        address(this),  // recipient
        bytes('flash') // trigger callback
    );
}

// Callback from Uniswap
function uniswapV2Call(
    address sender,
    uint amount0,
    uint amount1,
    bytes calldata data
) external {
    // Do something with the tokens...
    
    // Pay back tokens + fee
    uint fee = (amount0 * 3) / 997 + 1;
    IERC20(token0).transfer(msg.sender, amount0 + fee);
}
```

## Best Practices

1. **Calculate Fees**: Always account for 0.3% fee
2. **Check Callback**: Verify caller is Uniswap pair
3. **Handle Failures**: Ensure payback always happens
