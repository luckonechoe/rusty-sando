// ABI bindings for on-chain contracts, ported from `ethers::abigen!` to
// `alloy::sol!` in Phase 2b.  Only the function signatures actually used in
// non-simulator code paths are included; the rest carry empty bodies so the
// type names remain available for future phases.

alloy::sol! {
    #[sol(rpc)]
    interface IErc20 {
        function balanceOf(address account) external view returns (uint256);
        function transfer(address to, uint256 amount) external returns (bool);
        function transferFrom(address from, address to, uint256 amount) external returns (bool);
        function approve(address spender, uint256 amount) external returns (bool);
        function allowance(address owner, address spender) external view returns (uint256);
        function totalSupply() external view returns (uint256);
        function name() external view returns (string);
        function symbol() external view returns (string);
        function decimals() external view returns (uint8);

        event Transfer(address indexed from, address indexed to, uint256 value);
        event Approval(address indexed owner, address indexed spender, uint256 value);
    }
}

alloy::sol! {
    #[sol(rpc)]
    interface IUniswapV2Factory {
        function getPair(address tokenA, address tokenB) external view returns (address pair);
        function allPairs(uint256) external view returns (address pair);
        function allPairsLength() external view returns (uint256);
        function createPair(address tokenA, address tokenB) external returns (address pair);
    }
}

alloy::sol! {
    #[sol(rpc)]
    interface IUniswapV2Pair {
        function token0() external view returns (address);
        function token1() external view returns (address);
        function getReserves() external view returns (uint112 reserve0, uint112 reserve1, uint32 blockTimestampLast);
        function swap(uint256 amount0Out, uint256 amount1Out, address to, bytes calldata data) external;
    }
}

alloy::sol! {
    #[sol(rpc)]
    interface IUniswapV2Router {
        function swapExactTokensForTokens(
            uint256 amountIn,
            uint256 amountOutMin,
            address[] calldata path,
            address to,
            uint256 deadline
        ) external returns (uint256[] memory amounts);
        function swapTokensForExactTokens(
            uint256 amountOut,
            uint256 amountInMax,
            address[] calldata path,
            address to,
            uint256 deadline
        ) external returns (uint256[] memory amounts);
    }
}

alloy::sol! {
    #[sol(rpc)]
    interface IUniswapV3Factory {
        function getPool(address tokenA, address tokenB, uint24 fee) external view returns (address pool);
        function createPool(address tokenA, address tokenB, uint24 fee) external returns (address pool);
    }
}

alloy::sol! {
    #[sol(rpc)]
    interface IUniswapV3Pool {
        function token0() external view returns (address);
        function token1() external view returns (address);
        function fee() external view returns (uint24);
        function slot0() external view returns (
            uint160 sqrtPriceX96,
            int24 tick,
            uint16 observationIndex,
            uint16 observationCardinality,
            uint16 observationCardinalityNext,
            uint8 feeProtocol,
            bool unlocked
        );
        function swap(
            address recipient,
            bool zeroForOne,
            int256 amountSpecified,
            uint160 sqrtPriceLimitX96,
            bytes calldata data
        ) external returns (int256 amount0, int256 amount1);
    }
}

/// Re-export under legacy names for compatibility with existing code.
pub use IErc20::IErc20Instance as Erc20;
pub use IUniswapV2Factory::IUniswapV2FactoryInstance as UniswapV2Factory;
pub use IUniswapV2Pair::IUniswapV2PairInstance as UniswapV2Pair;
pub use IUniswapV2Router::IUniswapV2RouterInstance as UniswapV2Router;
pub use IUniswapV3Factory::IUniswapV3FactoryInstance as UniswapV3Factory;
pub use IUniswapV3Pool::IUniswapV3PoolInstance as UniswapV3Pool;
