#![allow(clippy::too_many_arguments)]
#[cfg(not(feature = "library"))]

use crate::state::{DETAILS, VOLUMES, MEDIA, MARKETS, ACTIVE_MARKETS, COMPLETED_MARKETS, ADMINS_MAP, TEMP_INFORMATION, KNOWN_MARKETS, STATISTICS, UNIQUE_WALLETS, INCENTIVES};

use packages::factory::{Statistics, TempInformation};

use cosmwasm_std::{
    entry_point, to_json_binary, CosmosMsg, DepsMut, Env, MessageInfo, Reply, ReplyOn, Response,
    StdError, StdResult, SubMsg, WasmMsg, Uint128, Addr, Event, Coin
};

use cw0::*;

use packages::market::{InstantiateMsg as InstantiateMarketMsg, ExecuteMsg as ExecuteMarketMsg};

pub mod execute_msg {

    use super::*;

    pub fn create_market(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        title: String,
        description: String,
        end_date: u64,
        categories: Vec<String>,
        media_: [String; 2]
    ) -> StdResult<Response> {

        let owner = info.sender;

        if end_date <= env.block.time.seconds() {
            return Err(StdError::generic_err("End date must be a date in the future"));
        }

        let status: bool = ADMINS_MAP.load(deps.storage, owner.clone()).unwrap_or(false);

        if !status {
            return Err(StdError::generic_err("Only admins can create markets"));
        }

        let details = DETAILS.load(deps.storage).unwrap();

        TEMP_INFORMATION.save(
            deps.storage,
            &TempInformation {
                title: title.clone(),
                description: description.clone(),
                yes_price: Uint128::from(0u128),
                no_price: Uint128::from(0u128),
                yes_liquidity: Uint128::from(0u128),
                no_liquidity: Uint128::from(0u128),
                yes_shares: Uint128::from(0u128),
                no_shares: Uint128::from(0u128),
                market_created: env.block.time.seconds(),
                market_end: end_date,
                categories: categories.clone(),
                liquidity_shares: Uint128::from(0u128),
                usdc: details.usdc.clone(),
                owner: owner.clone(),
                resolved: false,
                factory: env.contract.address.clone(),
                resolved_to: Uint128::from(0u128),
                media: media_.to_vec()
            }
        )?;

        let instantiate_market = CosmosMsg::Wasm(WasmMsg::Instantiate {
            code_id: details.market_code_id,
            funds: vec![],
            admin: Some(env.contract.address.clone().to_string()),
            label: "market_contract".to_string(),
            msg: to_json_binary(&InstantiateMarketMsg {
                title: title.clone(),
                description: description.clone(),
                end_date,
                categories,
                usdc: details.usdc.clone(),
                owner: owner.clone(),
                factory: env.contract.address.clone()
            })?
        });

        Ok(Response::new().add_submessage(SubMsg {
            id: 1u64,
            msg: instantiate_market,
            gas_limit: None,
            reply_on: ReplyOn::Success
        }))

    }

    pub fn add_admin(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        account: Addr
    )-> StdResult<Response> {


        deps.api.addr_validate(account.as_str())?;
        
        let is_admin = ADMINS_MAP.load(deps.storage, info.sender).unwrap_or(false);

        if !is_admin {
            return Err(StdError::generic_err("Only admins can add new admins".to_string()));
        }

        ADMINS_MAP.save(deps.storage, account, &true)?;

        Ok(Response::new())
        
    }

    pub fn remove_admin(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        account: Addr
    )-> StdResult<Response> {
        
        let is_admin = ADMINS_MAP.load(deps.storage, info.sender).unwrap_or(false);

        if !is_admin {
            return Err(StdError::generic_err("Only admins can remove admins".to_string()));
        }

        ADMINS_MAP.save(deps.storage, account, &false)?;

        Ok(Response::new())
        
    }

    pub fn record_stats(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        amount: Uint128,
        account: Addr,
        stat_type: String,
        data: Vec<Uint128>
    ) -> StdResult<Response> {

        let sender = info.sender;

        let exists = KNOWN_MARKETS.load(deps.storage, sender.clone()).unwrap_or(false);

        let mut response = Response::new();

        if !exists {
            return Err(StdError::generic_err("Call must be made from known market contract".to_string()));
        }

        let mut statistics = STATISTICS.load(deps.storage)?;

        if stat_type == *"volume" {

            let volume = VOLUMES.load(deps.storage, sender.clone()).unwrap_or(Uint128::from(0u128));
            VOLUMES.save(deps.storage, sender.clone(), &(volume + amount))?;
            
            statistics.volume += amount;

            let is_unique_wallet = UNIQUE_WALLETS.load(deps.storage, account.clone()).unwrap_or(false);
            if !is_unique_wallet {
                UNIQUE_WALLETS.save(deps.storage, account.clone(), &true)?;
                statistics.unique_wallets += Uint128::from(1u128);
            }

            response = Response::new()
                .add_event(Event::new("xionmarkets_event")
                    .add_attribute("market", sender)
                    .add_attribute("account", account)
                    .add_attribute("variant", data[usize::try_from(0).unwrap()])
                    .add_attribute("buy_or_sell", data[usize::try_from(1).unwrap()])
                    .add_attribute("price", data[usize::try_from(2).unwrap()])
                    .add_attribute("volume", amount)
                    .add_attribute("type", "order")
                    .add_attribute("factory", env.contract.address.clone()))

        }
        else if stat_type ==*"resolve" {

            statistics.completed_events += Uint128::from(1u128);

            let index = data[usize::try_from(0).unwrap()];

            let market_address = ACTIVE_MARKETS.load(deps.storage, index.u128()).unwrap_or(env.contract.address.clone());
            if market_address != sender {
                return Err(StdError::generic_err("Market index does not match intended market".to_string()));
            }

            let last_market = ACTIVE_MARKETS.load(deps.storage, statistics.active_events.u128()).unwrap_or(env.contract.address);

            ACTIVE_MARKETS.save(deps.storage, index.u128(), &last_market)?;
            ACTIVE_MARKETS.remove(deps.storage, statistics.active_events.u128());

            statistics.active_events -= Uint128::from(1u128);

            COMPLETED_MARKETS.save(deps.storage, statistics.completed_events.u128(), &market_address.clone())?;

            response = Response::new()
                .add_event(Event::new("xionmarkets_event")
                    .add_attribute("market", sender)
                    .add_attribute("type", "resolution"));
        }

        STATISTICS.save(deps.storage, &statistics)?;

        Ok(response)

    }

    pub fn initialize_liquidity(
        deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        market: Addr,
        yes_price: Uint128,
        liquidity: Uint128
    ) -> StdResult<Response> {

        let info = DETAILS.load(deps.storage)?;

        let account = _info.sender;

        let is_market_known = KNOWN_MARKETS.load(deps.storage, market.clone())?;
        if !is_market_known {
            return Err(StdError::generic_err("Incorrect market"));
        }

        let sent_sufficient_funds = _info.funds.iter().any(|coin| {
            coin.denom == info.usdc.to_string() && coin.amount == liquidity
        });

        if !sent_sufficient_funds {
            return Err(StdError::generic_err("Invalid funds"));
        }

        let xfer_funds = Coin {
            denom: info.usdc.clone(),
            amount: liquidity
        };

        let external_msg = WasmMsg::Execute {
            contract_addr: market.to_string(),
            msg: to_json_binary(&ExecuteMarketMsg::InitializeLiquidity {
                yes_price,
                liquidity,
                receiver: account
            })?,
            funds: vec![xfer_funds]
        };

        Ok(Response::new().add_message(external_msg))

    }

    pub fn add_liquidity(
        deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        market: Addr,
        amount: Uint128
    ) -> StdResult<Response> {

        let info = DETAILS.load(deps.storage)?;

        let account = _info.sender;

        let is_market_known = KNOWN_MARKETS.load(deps.storage, market.clone())?;
        if !is_market_known {
            return Err(StdError::generic_err("Incorrect market"));
        }

        let sent_sufficient_funds = _info.funds.iter().any(|coin| {
            coin.denom == info.usdc.to_string() && coin.amount == amount
        });

        if !sent_sufficient_funds {
            return Err(StdError::generic_err("Invalid funds"));
        }

        let xfer_funds = Coin {
            denom: info.usdc.clone(),
            amount
        };

        let external_msg = WasmMsg::Execute {
            contract_addr: market.to_string(),
            msg: to_json_binary(&ExecuteMarketMsg::AddLiquidity {
                amount,
                receiver: account
            })?,
            funds: vec![xfer_funds]
        };

        Ok(Response::new().add_message(external_msg))

    }

    pub fn remove_liquidity(
        _deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        market: Addr,
        shares: Uint128
    ) -> StdResult<Response> {

        let account = _info.sender;
        
        let is_market_known = KNOWN_MARKETS.load(_deps.storage, market.clone())?;
        if !is_market_known {
            return Err(StdError::generic_err("Incorrect market"));
        }
        
        let external_msg = WasmMsg::Execute {
            contract_addr: market.to_string(),
            msg: to_json_binary(&ExecuteMarketMsg::RemoveLiquidity {
                shares,
                receiver: account
            })?,
            funds: vec![]
        };

        Ok(Response::new().add_message(external_msg))

    }

    pub fn claim(
        _deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        market: Addr,
        variant: Uint128
    ) -> StdResult<Response> {

        let account = _info.sender;

        let is_market_known = KNOWN_MARKETS.load(_deps.storage, market.clone())?;
        if !is_market_known {
            return Err(StdError::generic_err("Incorrect market"));
        }

        let external_msg = WasmMsg::Execute {
            contract_addr: market.to_string(),
            msg: to_json_binary(&ExecuteMarketMsg::Claim {
                variant,
                receiver: account
            })?,
            funds: vec![]
        };

        Ok(Response::new().add_message(external_msg))

    }

    pub fn resolve_market(
        _deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        market: Addr,
        variant: Uint128,
        market_index: u128
    ) -> StdResult<Response> {

        let account = _info.sender;

        let is_market_known = KNOWN_MARKETS.load(_deps.storage, market.clone())?;
        if !is_market_known {
            return Err(StdError::generic_err("Incorrect market"));
        }

        let external_msg = WasmMsg::Execute {
            contract_addr: market.to_string(),
            msg: to_json_binary(&ExecuteMarketMsg::ResolveMarket {
                variant,
                receiver: account,
                market_index
            })?,
            funds: vec![]
        };

        Ok(Response::new().add_message(external_msg))

    }

    pub fn place_order(
        deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        market: Addr,
        variant: Uint128,
        buy_or_sell: Uint128,
        amount: Uint128
    ) -> StdResult<Response> {

        let info = DETAILS.load(deps.storage)?;

        let account = _info.sender;
        
        let is_market_known = KNOWN_MARKETS.load(deps.storage, market.clone())?;
        if !is_market_known {
            return Err(StdError::generic_err("Incorrect market"));
        }

        let mut funds = vec![];

        if buy_or_sell == Uint128::from(1u128) {

            let sent_sufficient_funds = _info.funds.iter().any(|coin| {
                coin.denom == info.usdc.to_string() && coin.amount == amount
            });

            if !sent_sufficient_funds {
                return Err(StdError::generic_err("Invalid funds"));
            }
    
            let xfer_funds = Coin {
                denom: info.usdc.clone(),
                amount
            };

            funds = vec![xfer_funds];

        }

        let external_msg = WasmMsg::Execute {
            contract_addr: market.to_string(),
            msg: to_json_binary(&ExecuteMarketMsg::PlaceOrder {
                variant,
                buy_or_sell,
                amount,
                receiver: account.clone()
            })?,
            funds
        };

        let incentives = INCENTIVES.load(deps.storage, account.clone()).unwrap_or(0u64);
        INCENTIVES.save(deps.storage, account.clone(), &(incentives + 10u64))?;

        Ok(Response::new().add_message(external_msg))

    }

}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> StdResult<Response> {
    match msg.id {
        1u64 => reply::instantiate_reply(deps, env, msg),
        _ => Ok(Response::default()),
    }
}

pub mod reply {
    use super::*;
    pub fn instantiate_reply(deps: DepsMut, _env: Env, msg: Reply) -> StdResult<Response> {
        
        let temp_information: TempInformation = TEMP_INFORMATION.load(deps.storage)?;
        let mut statistics: Statistics = STATISTICS.load(deps.storage)?;

        let res = parse_reply_instantiate_data(msg).map_err(|e| {
            StdError::generic_err(format!("parse reply instantiate data error: {}", e))
        })?;

        let contract_address = Addr::unchecked(res.contract_address);

        let media: Vec<String> = temp_information.media;

        statistics.total_pools += Uint128::from(1u128);
        statistics.active_events += Uint128::from(1u128);
        STATISTICS.save(deps.storage, &statistics)?;

        KNOWN_MARKETS.save(deps.storage, contract_address.clone(), &true)?;
        ACTIVE_MARKETS.save(deps.storage, statistics.active_events.u128(), &contract_address.clone())?;
        MARKETS.save(deps.storage, statistics.total_pools.u128(), &contract_address.clone())?;
        
        MEDIA.save(deps.storage, contract_address.clone(), &media)?;
        
        Ok(Response::new()
            .add_attribute("contract_address", contract_address.clone())
        )
        
    }
}