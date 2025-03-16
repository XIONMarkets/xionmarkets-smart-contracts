#![allow(unused_imports)]

use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128};

use crate::market::{Shares, Information, Quote};

#[cw_serde]
pub struct InstantiateMsg {
    pub usdc: String,
    pub fees_address: Addr,
    pub market_code_id: u64
}

#[cw_serde]
pub struct Details {
    pub usdc: String,
    pub fees_address: Addr,
    pub market_code_id: u64
}

#[cw_serde]
pub struct Statistics {
    pub volume: Uint128,
    pub total_pools: Uint128,
    pub unique_wallets: Uint128,
    pub active_events: Uint128,
    pub completed_events: Uint128
}

#[cw_serde]
pub enum ExecuteMsg {
    CreateMarket {
        title: String,
        description: String,
        end_date: u64,
        categories: Vec<String>,
        media: [String; 2]
    },
    RecordStats {
        amount: Uint128,
        account: Addr,
        stat_type: String,
        data: Vec<Uint128>
    },
    AddAdmin {
        account: Addr
    },
    RemoveAdmin {
        account: Addr
    },
    InitializeLiquidity {
        market: Addr,
        yes_price: Uint128,
        liquidity: Uint128
    },
    AddLiquidity {
        market: Addr,
        amount: Uint128
    },
    RemoveLiquidity {
        market: Addr,
        shares: Uint128
    },
    Claim {
        market: Addr,
        variant: Uint128
    },
    ResolveMarket {
        market: Addr,
        variant: Uint128,
        market_index: u128
    },
    PlaceOrder {
        market: Addr,
        variant: Uint128,
        buy_or_sell: Uint128,
        amount: Uint128
    }
}

#[cw_serde]
pub struct MarketInfo {
    pub information: Information,
    pub shares: Shares,
    pub volume: Uint128,
    pub media: [String; 2]
}

#[cw_serde]
pub struct MarketList {
    pub information: Vec<Information>,
    pub contracts: Vec<Addr>,
    pub indexes: Vec<u128>
}

#[cw_serde]
pub struct TempInformation {
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
    pub resolved_to: Uint128,
    pub media: Vec<String>
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(MarketInfo)]
    GetMarketInfo { contract_address: Addr, account: Addr },
    #[returns(Statistics)]
    GetStatistics {},
    #[returns(MarketList)]
    FetchMarkets { page: u128, items_per_page: u128, account: Addr, market_type: u128 },
    #[returns(Details)]
    Details {},
    #[returns(Addr)]
    FeesAddress {},
    #[returns(Quote)]
    Quote { market: Addr, variant: Uint128, buy_or_sell: Uint128, amount: Uint128 },
    #[returns(bool)]
    IsAdmin { account: Addr },
    #[returns(u64)]
    GetIncentives { account: Addr }
}