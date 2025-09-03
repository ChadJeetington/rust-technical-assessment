// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.0;

/**
 * @title UniswapV2Pair
 * @notice Implementation of Uniswap V2 pair contract
 * @dev Handles core AMM logic using constant product formula
 */
contract UniswapV2Pair {
    // Token balances
    uint112 private reserve0;
    uint112 private reserve1;
    uint32  private blockTimestampLast;

    // Price oracle accumulators
    uint public price0CumulativeLast;
    uint public price1CumulativeLast;

    /**
     * @notice Executes a swap between the pair tokens
     * @param amount0Out Amount of token0 to output
     * @param amount1Out Amount of token1 to output
     * @param to Recipient address
     * @param data Optional callback data for flash swaps
     */
    function swap(
        uint amount0Out,
        uint amount1Out,
        address to,
        bytes calldata data
    ) external {
        require(amount0Out > 0 || amount1Out > 0, 'INSUFFICIENT_OUTPUT_AMOUNT');
        require(amount0Out < reserve0 && amount1Out < reserve1, 'INSUFFICIENT_LIQUIDITY');
        
        // Transfer tokens
        if (amount0Out > 0) _safeTransfer(token0, to, amount0Out);
        if (amount1Out > 0) _safeTransfer(token1, to, amount1Out);
        
        // Handle flash swap if data is provided
        if (data.length > 0) {
            IUniswapV2Callee(to).uniswapV2Call(msg.sender, amount0Out, amount1Out, data);
        }
        
        // Update reserves
        _update(balance0, balance1, reserve0, reserve1);
    }

    /**
     * @notice Adds liquidity to the pair
     * @param to Recipient of LP tokens
     * @return liquidity Amount of LP tokens minted
     */
    function mint(address to) external returns (uint liquidity) {
        (uint112 _reserve0, uint112 _reserve1,) = getReserves();
        uint balance0 = IERC20(token0).balanceOf(address(this));
        uint balance1 = IERC20(token1).balanceOf(address(this));
        uint amount0 = balance0 - _reserve0;
        uint amount1 = balance1 - _reserve1;

        // Initial liquidity provision
        if (totalSupply == 0) {
            liquidity = Math.sqrt(amount0 * amount1) - MINIMUM_LIQUIDITY;
            _mint(address(0), MINIMUM_LIQUIDITY);
        } else {
            liquidity = Math.min(
                (amount0 * totalSupply) / _reserve0,
                (amount1 * totalSupply) / _reserve1
            );
        }

        require(liquidity > 0, 'INSUFFICIENT_LIQUIDITY_MINTED');
        _mint(to, liquidity);
        _update(balance0, balance1, _reserve0, _reserve1);
    }
}