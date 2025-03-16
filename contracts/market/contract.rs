use crate::state::{INFORMATION, TOTAL_ORDERS};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
    Uint128,
};
use packages::market::{ExecuteMsg, InstantiateMsg, Information, QueryMsg};

use crate::execute::execute_msg;
use crate::query::query_msg;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {

    let market_info: Information = Information {
        title: msg.title,
        description: msg.description,
        yes_price: Uint128::from(0u128),
        no_price: Uint128::from(0u128),
        yes_liquidity: Uint128::from(0u128),
        no_liquidity: Uint128::from(0u128),
        yes_shares: Uint128::from(0u128),
        no_shares: Uint128::from(0u128),
        market_created: env.block.time.seconds(),
        market_end: msg.end_date,
        categories: msg.categories,
        liquidity_shares: Uint128::from(0u128),
        usdc: msg.usdc,
        owner: msg.owner,
        resolved: false,
        factory: msg.factory,
        resolved_to: Uint128::from(0u128)
    };

    INFORMATION.save(deps.storage, &market_info)?;

    TOTAL_ORDERS.save(deps.storage, &Uint128::from(0u128))?;

    Ok(Response::new())

}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::InitializeLiquidity {
            yes_price,
            liquidity,
            receiver
        } => execute_msg::initialize_liquidity(
            deps,
            env,
            info,
            yes_price,
            liquidity,
            receiver
        ),
        ExecuteMsg::AddLiquidity {
            amount,
            receiver
        } => execute_msg::add_liquidity(deps, env, info, amount, receiver),

        ExecuteMsg::RemoveLiquidity { shares, receiver } => {
            execute_msg::remove_liquidity(deps, env, info, shares, receiver)
        },
        ExecuteMsg::Claim { variant, receiver } => {
            execute_msg::claim(deps, env, info, variant, receiver)
        },
        ExecuteMsg::ResolveMarket { variant, receiver, market_index } => {
            execute_msg::resolve_market(deps, env, info, variant, receiver, market_index)
        },
        ExecuteMsg::PlaceOrder { variant, buy_or_sell, amount, receiver } => {
            execute_msg::place_order(deps, env, info, variant, buy_or_sell, amount, receiver)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetInfo { account } => to_json_binary(&query_msg::get_info(deps, env, account)?),
        QueryMsg::Quote { variant, buy_or_sell, amount } => to_json_binary(&execute_msg::quote(deps, env, variant, buy_or_sell, amount)?),
        QueryMsg::GetTotalOrders { } => to_json_binary(&query_msg::get_total_orders(deps, env)?),
        QueryMsg::GetOrders { page, items_per_page } => to_json_binary(&query_msg::get_orders(deps, env, page, items_per_page)?),
    }
}