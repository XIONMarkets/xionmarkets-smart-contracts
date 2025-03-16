use cosmwasm_schema::cw_serde;
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{Addr, Uint128};

#[cw_serde]
pub struct InstantiateMsg {
    pub title: String,
    pub description: String,
    pub end_date: u64,
    pub categories: Vec<String>,
    pub usdc: String,
    pub owner: Addr,
    pub factory: Addr
}

#[cw_serde]
pub enum ExecuteMsg {
    InitializeLiquidity {
        yes_price: Uint128,
        liquidity: Uint128,
        receiver: Addr
    },
    AddLiquidity {
        amount: Uint128,
        receiver: Addr
    },
    RemoveLiquidity {
        shares: Uint128,
        receiver: Addr
    },
    Claim {
        variant: Uint128,
        receiver: Addr
    },
    ResolveMarket {
        variant: Uint128,
        receiver: Addr,
        market_index: u128
    },
    PlaceOrder {
        variant: Uint128,
        buy_or_sell: Uint128,
        amount: Uint128,
        receiver: Addr
    }
}

#[cw_serde]
pub struct Information {
    pub title: String,
    pub description: String,
    pub yes_price: Uint128,
    pub no_price: Uint128,
    pub yes_liquidity: Uint128,
    pub no_liquidity: Uint128,
    pub yes_shares: Uint128,
    pub no_shares: Uint128,
    pub market_created: u64,
    pub market_end: u64,
    pub categories: Vec<String>,
    pub liquidity_shares: Uint128,
    pub usdc: String,
    pub owner: Addr,
    pub resolved: bool,
    pub factory: Addr,
    pub resolved_to: Uint128
}

#[cw_serde]
pub struct Order {
    pub timestamp: u64,
    pub price: Uint128,
}

#[cw_serde]
pub struct Shares {
    pub yes_shares: Uint128,
    pub no_shares: Uint128,
    pub liquidity_shares: Uint128
}

impl Shares {
    pub fn new() -> Self {
        Shares {
            yes_shares: Uint128::from(0u128),
            no_shares: Uint128::from(0u128),
            liquidity_shares: Uint128::from(0u128)
        }
    }
}

impl Default for Shares {
    fn default() -> Self {
        Self::new()
    }
}

#[cw_serde]
pub struct Data {
    pub information: Information,
    pub shares: Shares
}

#[cw_serde]
pub struct Quote {
    pub amount_out: Uint128,
    pub impact: Uint128,
    pub price: Uint128,
    pub fees: Uint128
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Data)]
    GetInfo {
        account: Addr
    },
    #[returns(Quote)]
    Quote {
        variant: Uint128,
        buy_or_sell: Uint128,
        amount: Uint128
    },
    #[returns((u64, Vec<Order>))]
    GetOrders {
        page: u128,
        items_per_page: u128
    },
    #[returns(Uint128)]
    GetTotalOrders {}
}