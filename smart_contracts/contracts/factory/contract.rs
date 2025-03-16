use crate::state::{DETAILS, VOLUMES, MEDIA, MARKETS, ACTIVE_MARKETS, STATISTICS, COMPLETED_MARKETS, ADMINS_MAP, INCENTIVES};

use packages::factory::{ExecuteMsg, InstantiateMsg, QueryMsg, MarketInfo, MarketList, Details, Statistics};

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response,
    StdError, StdResult, WasmQuery, Uint128, Addr, QueryRequest
};

use packages::market::{Data, Quote, QueryMsg as QueryMarketMsg, Information, Shares};

use crate::execute::execute_msg;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> { 
    
    deps.api.addr_validate(msg.fees_address.as_str())?;
    
    let details: Details = Details {
        usdc: msg.usdc,
        fees_address: msg.fees_address,
        market_code_id: msg.market_code_id
    };
    let statistics: Statistics = Statistics {
        volume: Uint128::from(0u128),
        total_pools: Uint128::from(0u128),
        unique_wallets: Uint128::from(0u128),
        active_events: Uint128::from(0u128),
        completed_events: Uint128::from(0u128)
    };
    STATISTICS.save(deps.storage, &statistics)?;
    DETAILS.save(deps.storage, &details)?;
    ADMINS_MAP.save(deps.storage, info.sender, &true)?;
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::CreateMarket { title, description, end_date, categories, media } => {
            execute_msg::create_market(deps, env, info, title, description, end_date, categories, media)
        },
        ExecuteMsg::RecordStats { amount, account, stat_type, data } => {
            execute_msg::record_stats(deps, env, info, amount, account, stat_type, data)
        }
        ExecuteMsg::AddAdmin { account } => {
            execute_msg::add_admin(deps, env, info, account)
        }
        ExecuteMsg::RemoveAdmin { account } => {
            execute_msg::remove_admin(deps, env, info, account)
        }
        ExecuteMsg::InitializeLiquidity {
            market,
            yes_price,
            liquidity
        } => execute_msg::initialize_liquidity(
            deps,
            env,
            info,
            market,
            yes_price,
            liquidity
        ),
        ExecuteMsg::AddLiquidity {
            market,
            amount
        } => execute_msg::add_liquidity(deps, env, info, market, amount),

        ExecuteMsg::RemoveLiquidity { market, shares } => {
            execute_msg::remove_liquidity(deps, env, info, market, shares)
        },
        ExecuteMsg::Claim { market, variant } => {
            execute_msg::claim(deps, env, info, market, variant)
        },
        ExecuteMsg::ResolveMarket { market, variant, market_index } => {
            execute_msg::resolve_market(deps, env, info, market, variant, market_index)
        },
        ExecuteMsg::PlaceOrder { market, variant, buy_or_sell, amount } => {
            execute_msg::place_order(deps, env, info, market, variant, buy_or_sell, amount)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetMarketInfo { contract_address, account } => to_json_binary(&query::get_market_info(deps, _env, contract_address, account)?),
        QueryMsg::FetchMarkets { page, items_per_page, account, market_type } => to_json_binary(&query::fetch_markets(deps, _env, page, items_per_page, account, market_type)?),
        QueryMsg::FeesAddress {} => to_json_binary(&query::fees_address(deps, _env)?),
        QueryMsg::Quote { market, variant, buy_or_sell, amount } => to_json_binary(&query::quote(deps, _env, market, variant, buy_or_sell, amount)?),
        QueryMsg::IsAdmin { account } => to_json_binary(&query::is_admin(deps, _env, account)?),
        QueryMsg::GetIncentives { account } => {
            to_json_binary::<u64>(&INCENTIVES.load(deps.storage, account)?)
        },
        QueryMsg::Details {} => {
            to_json_binary::<Details>(&DETAILS.load(deps.storage)?)
        },
        QueryMsg::GetStatistics {} => {
            to_json_binary::<Statistics>(&STATISTICS.load(deps.storage)?)
        }
    }
}

pub mod query {

    use super::*;

    pub fn fees_address(_deps: Deps, _env: Env) -> StdResult<Addr> {

        let details = DETAILS.load(_deps.storage)?;

        let fees_address = details.fees_address;

        Ok(fees_address)

    }

    pub fn is_admin(_deps: Deps, _env: Env, account: Addr) -> StdResult<bool> {

        let is_admin = ADMINS_MAP.load(_deps.storage, account).unwrap_or(false);

        Ok(is_admin)

    }

    pub fn quote(_deps: Deps, _env: Env, market: Addr, variant: Uint128, buy_or_sell: Uint128, amount: Uint128) -> StdResult<Quote> {

        let msg = QueryMarketMsg::Quote {
            variant,
            buy_or_sell,
            amount
        };
        
        let query_msg = QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: market.to_string(),
            msg: to_json_binary(&msg)?,
        });

        let quote: Quote = _deps.querier.query(&query_msg)?;
        
        Ok(quote)

    }

    pub fn get_market_info(_deps: Deps, _env: Env, contract_address: Addr, account: Addr) -> StdResult<MarketInfo> {

        let msg = QueryMarketMsg::GetInfo {
            account
        };
        
        let query_msg = QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: contract_address.to_string(),
            msg: to_json_binary(&msg)?,
        });

        let data: Data = _deps.querier.query(&query_msg)?;

        let information: Information = data.information;
        let shares: Shares = data.shares;

        let volume: Uint128 = VOLUMES.load(_deps.storage, contract_address.clone()).unwrap_or(Uint128::from(0u128));
        let media_: Vec<String> = MEDIA.load(_deps.storage, contract_address.clone()).unwrap_or(vec![String::from(""), String::from("")]);

        let media: [String; 2] = media_.try_into().unwrap();

        let market_info: MarketInfo = MarketInfo {
            information,
            shares,
            volume,
            media
        };

        Ok(market_info)

    }

    pub fn fetch_markets(_deps: Deps, _env: Env, page_: u128, items_per_page: u128, account: Addr, market_type: u128) -> StdResult<MarketList> {

        if page_ == 0u128 {
            return Err(StdError::generic_err("Page must be greater than 0"));
        }
        if items_per_page == 0u128 {
            return Err(StdError::generic_err("Items per page must be greater than 0"));
        }

        let statistics = STATISTICS.load(_deps.storage)?;

        let total_markets;
        
        if market_type == 0u128 {
            total_markets = statistics.total_pools;
        }
        else if market_type == 1u128 {
            total_markets = statistics.active_events;
        }
        else {
            total_markets = statistics.completed_events;
        }

        let number_of_pages = if total_markets % Uint128::from(items_per_page) == Uint128::from(0u128) {
            total_markets / Uint128::from(items_per_page)
        }
        else {
            (total_markets / Uint128::from(items_per_page)) + Uint128::from(1u128)
        };

        let page = Uint128::from(page_);

        if page > number_of_pages {
            return Err(StdError::generic_err("Page is out of bounds"));
        }

        if total_markets == Uint128::from(0u128) {
            return Err(StdError::generic_err("No market exists at the moment"));
        }

        let start_index = total_markets - ((page - Uint128::from(1u128)) * Uint128::from(items_per_page));

        let end_index = if start_index < Uint128::from(items_per_page) {
            Uint128::from(1u128)
        }
        else {
            (start_index - Uint128::from(items_per_page)) + Uint128::from(1u128)
        };

        let number_of_items = (start_index - end_index) + Uint128::from(1u128);

        if number_of_items == Uint128::from(0u128) {
            return Err(StdError::generic_err("No market found for this page"));
        }

        let mut paginated_markets: Vec<Information> = vec![];
        let mut contract_addresses: Vec<Addr> = vec![];
        let mut indexes: Vec<u128> = vec![];

        let mut current_index = start_index;

        while current_index >= end_index {

            let index = current_index.u128();

            let market_at;
            
            if market_type == 0u128 {
                market_at = MARKETS.load(_deps.storage, index)?;
            }
            else if market_type == 1u128 {
                market_at = ACTIVE_MARKETS.load(_deps.storage, index)?;
            }
            else {
                market_at = COMPLETED_MARKETS.load(_deps.storage, index)?;
            }
            
            let msg = QueryMarketMsg::GetInfo {
                account: account.clone()
            };

            let query_msg = &QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: market_at.to_string(),
                msg: to_json_binary(&msg)?,
            });

            let data: Data = _deps.querier.query(query_msg)?;

            let information: Information = data.information;

            paginated_markets.push(information);
            contract_addresses.push(market_at);
            indexes.push(index);

            current_index = Uint128::from(current_index.u128() - 1u128);

        }

        let market_list: MarketList = MarketList {
            information: paginated_markets,
            contracts: contract_addresses,
            indexes
        };

        Ok(market_list)

    }
}