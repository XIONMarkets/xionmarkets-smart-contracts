use crate::state::{INFORMATION, SHARES, TOTAL_ORDERS, ORDER_LIST};

#[cfg(not(feature = "library"))]

use cosmwasm_std::{Deps, to_json_binary, DepsMut, BalanceResponse, Env, MessageInfo, Response, StdError, StdResult, QueryRequest, WasmQuery, Binary, BankQuery, Coin, BankMsg};

use packages::market::{Shares, Quote, Order};

use packages::factory::{ExecuteMsg as ExecuteFactoryMsg, QueryMsg as QueryFactoryMsg};

pub mod execute_msg {

    use cosmwasm_std::{Addr, CosmosMsg, Isqrt, Uint128, WasmMsg};

    const MULTIPLIER:u128 = 10u128.pow(8);

    const RESOLVE_DURATION:u64 = 180; // 5 minutes

    use super::*;

    pub fn initialize_liquidity(
        deps: DepsMut,
        env: Env,
        info_: MessageInfo,
        yes_price: Uint128,
        liquidity: Uint128,
        receiver: Addr
    ) -> StdResult<Response> {

        let mut info = INFORMATION.load(deps.storage)?;

        let account = info_.sender;

        let msg = QueryFactoryMsg::IsAdmin { account: receiver.clone() };
        
        let query_msg = QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: info.factory.to_string(),
            msg: to_json_binary(&msg)?,
        });

        let is_admin: bool = deps.querier.query(&query_msg)?;

        if receiver.clone() != info.owner && !is_admin  {
            return Err(StdError::generic_err("Only owner or admins can initialize price"));
        }

        if account != info.factory {
            return Err(StdError::generic_err("Only callable by factory"));
        }

        if liquidity < Uint128::from(10u128 * 10u128.pow(6u32)) {
            return Err(StdError::generic_err("Liquidity must be up to 10 USD"));
        }
        if info.resolved {
            return Err(StdError::generic_err("Market has already been resolved"));
        }
        if info.yes_price != Uint128::from(0u128) {
            return Err(StdError::generic_err("Liquidity has already been initialized"));
        }
        if yes_price < Uint128::from(MULTIPLIER) / Uint128::from(10u128) || yes_price > Uint128::from(9u128) * (Uint128::from(MULTIPLIER) / Uint128::from(10u128)) {
            return Err(StdError::generic_err("Initial probability must be up to 10% and less than or equal to 90%"));
        }

        let no_price = Uint128::from(MULTIPLIER) - yes_price;
        info.yes_price = yes_price;
        info.no_price = no_price;

        let timestamp = env.block.time.seconds();

        let mut total_orders = TOTAL_ORDERS.load(deps.storage)?;
        total_orders += Uint128::from(1u128);

        ORDER_LIST.save(deps.storage, total_orders.u128(), &Order{
            timestamp,
            price: yes_price
        })?;

        TOTAL_ORDERS.save(deps.storage, &total_orders)?;

        let yes_liquidity = (yes_price * liquidity) / Uint128::from(MULTIPLIER);
        let no_liquidity = (no_price * liquidity) / Uint128::from(MULTIPLIER);

        info.yes_liquidity = yes_liquidity;
        info.no_liquidity = no_liquidity;

        let shares_to_give = Uint128::from(Isqrt::isqrt(yes_liquidity.u128() * no_liquidity.u128()));
        info.liquidity_shares += shares_to_give;

        let mut shares: Shares = SHARES.load(deps.storage, receiver.clone()).unwrap_or_else(|_| Shares::new());
        shares.liquidity_shares += shares_to_give;

        INFORMATION.save(deps.storage, &info)?;
        SHARES.save(deps.storage, receiver.clone(), &shares)?;

        Ok(Response::new())

    }

    fn min(
        x: Uint128,
        y: Uint128
    ) -> Uint128 {
        if x < y {
            x
        }
        else {
            y
        }
    }

    pub fn add_liquidity(
        deps: DepsMut,
        _env: Env,
        info_: MessageInfo,
        amount: Uint128,
        receiver: Addr
    ) -> StdResult<Response> {

        let mut info = INFORMATION.load(deps.storage)?;

        let account = info_.sender;

        if account != info.factory {
            return Err(StdError::generic_err("Only callable by factory"));
        }

        if info.yes_price == Uint128::from(0u128) {
            return Err(StdError::generic_err("Liquidity has not been initialized"));
        }
        if info.resolved {
            return Err(StdError::generic_err("Market has been resolved. Cannot add liquidity."));
        }

        let yes_liquidity = (info.yes_price * amount) / Uint128::from(MULTIPLIER);
        let no_liquidity = (info.no_price * amount) / Uint128::from(MULTIPLIER);

        let shares_to_give = min((yes_liquidity * info.liquidity_shares) / info.yes_liquidity, (no_liquidity * info.liquidity_shares) / info.no_liquidity);

        if shares_to_give == Uint128::from(0u128) {
            return Err(StdError::generic_err("Must use higher deposit limit."));
        }

        let mut shares: Shares = SHARES.load(deps.storage, receiver.clone()).unwrap_or_else(|_| Shares::new());

        shares.liquidity_shares += shares_to_give;

        info.liquidity_shares += shares_to_give;
        info.yes_liquidity += yes_liquidity;
        info.no_liquidity += no_liquidity;

        SHARES.save(deps.storage, receiver.clone(), &shares)?;
        INFORMATION.save(deps.storage, &info)?;

        Ok(Response::new())

    }

    pub fn remove_liquidity(
        deps: DepsMut,
        _env: Env,
        info_: MessageInfo,
        shares_: Uint128,
        receiver: Addr
    ) -> StdResult<Response> {

        let mut info = INFORMATION.load(deps.storage)?;

        let account = info_.sender;

        if account != info.factory {
            return Err(StdError::generic_err("Only callable by factory"));
        }

        let mut shares: Shares = SHARES.load(deps.storage, receiver.clone()).unwrap_or_else(|_| Shares::new());

        if shares.liquidity_shares < shares_ {
            return Err(StdError::generic_err("User must own up to the specified amount of shares"));
        }

        let yes_to_remove = (shares_ * info.yes_liquidity) / info.liquidity_shares;
        let no_to_remove = (shares_ * info.no_liquidity) / info.liquidity_shares;

        info.liquidity_shares -= shares_;
        shares.liquidity_shares -= shares_;

        let mut amount_to_remove = yes_to_remove + no_to_remove;

        if amount_to_remove == Uint128::from(0u128) {
            return Err(StdError::generic_err("Shares too minute for withdrawal."));
        }

        let usdc_balance: Uint128 = {
            let request = QueryRequest::Bank(BankQuery::Balance {
                denom: info.usdc.to_string(),
                address: _env.contract.address.to_string()
            });
            let response: BalanceResponse = deps.querier.query(&request)?;
            response.amount.amount
        };

        let total_balance = usdc_balance;

        if amount_to_remove > total_balance {
            amount_to_remove = total_balance;
        }

        if !info.resolved && ((info.yes_liquidity + info.no_liquidity) - amount_to_remove < Uint128::from(10u128 * 10u128.pow(6u32))) {
            return Err(StdError::generic_err("There must be at least 10 USDC leftover"));
        }

        info.yes_liquidity -= yes_to_remove;
        info.no_liquidity -= no_to_remove;

        let mut messages = vec![];

        let xfer_funds = Coin {
            denom: info.usdc.clone(),
            amount: amount_to_remove
        };

        let asset_transfer = CosmosMsg::Bank(BankMsg::Send {
            to_address: receiver.to_string(),
            amount: vec![xfer_funds],
        });
        
        messages.push(asset_transfer);

        SHARES.save(deps.storage, receiver.clone(), &shares)?;
        INFORMATION.save(deps.storage, &info)?;

        Ok(Response::new().add_messages(messages))

    }

    pub fn claim(
        deps: DepsMut,
        _env: Env,
        info_: MessageInfo,
        variant: Uint128,
        receiver: Addr
    ) -> StdResult<Response> {

        let mut info = INFORMATION.load(deps.storage)?;

        let account = info_.sender;

        let mut shares: Shares = SHARES.load(deps.storage, receiver.clone()).unwrap_or_else(|_| Shares::new());

        if account != info.factory {
            return Err(StdError::generic_err("Only callable by factory"));
        }

        if !info.resolved {
            return Err(StdError::generic_err("Market must be resolved for user to claim"));
        }

        if variant != info.resolved_to {
            return Err(StdError::generic_err("Can only claim from resolved (won) market"));
        }

        let mut messages = vec![];

        if variant == Uint128::from(1u128) {

            let owned_shares = shares.yes_shares;

            if owned_shares == Uint128::from(0u128) {
                return Err(StdError::generic_err("Must own shares to claim"));
            }

            let expected_usdc = (owned_shares * info.yes_price) / Uint128::from(MULTIPLIER);

            let xfer_asset = Coin {
                denom: info.usdc.clone(),
                amount: expected_usdc.clone()
            };
    
            let asset_transfer = CosmosMsg::Bank(BankMsg::Send {
                to_address: receiver.clone().to_string(),
                amount: vec![xfer_asset],
            });

            messages.push(asset_transfer);

            let total_liquidity = info.yes_liquidity + info.no_liquidity;

            let yes_to_remove = (info.yes_liquidity * expected_usdc) / total_liquidity;
            let no_to_remove = (info.no_liquidity * expected_usdc) / total_liquidity;
            
            info.yes_shares -= owned_shares;
            info.yes_liquidity -= yes_to_remove;
            info.no_liquidity -= no_to_remove;
            shares.yes_shares -= owned_shares;

        }
        else if variant == Uint128::from(0u128) {

            let owned_shares = shares.no_shares;

            if owned_shares == Uint128::from(0u128) {
                return Err(StdError::generic_err("Must own shares to claim"));
            }

            let expected_usdc = (owned_shares * info.no_price) / Uint128::from(MULTIPLIER);

            let xfer_asset = Coin {
                denom: info.usdc.clone(),
                amount: expected_usdc.clone()
            };
    
            let asset_transfer = CosmosMsg::Bank(BankMsg::Send {
                to_address: receiver.clone().to_string(),
                amount: vec![xfer_asset],
            });

            messages.push(asset_transfer);

            let total_liquidity = info.yes_liquidity + info.no_liquidity;

            let yes_to_remove = (info.yes_liquidity * expected_usdc) / total_liquidity;
            let no_to_remove = (info.no_liquidity * expected_usdc) / total_liquidity;

            info.no_shares -= owned_shares;
            info.yes_liquidity -= yes_to_remove;
            info.no_liquidity -= no_to_remove;
            shares.no_shares -= owned_shares;

        }

        SHARES.save(deps.storage, receiver.clone(), &shares)?;
        INFORMATION.save(deps.storage, &info)?;

        Ok(Response::new().add_messages(messages))

    }

    pub fn resolve_market(
        deps: DepsMut,
        env: Env,
        info_: MessageInfo,
        variant: Uint128,
        receiver: Addr,
        market_index: u128
    ) -> StdResult<Response> {

        let mut info = INFORMATION.load(deps.storage)?;

        let account = info_.sender;

        if account != info.factory {
            return Err(StdError::generic_err("Only callable by factory"));
        }

        let msg = QueryFactoryMsg::IsAdmin { account: receiver.clone() };
        
        let query_msg = QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: info.factory.to_string(),
            msg: to_json_binary(&msg)?,
        });

        let is_admin: bool = deps.querier.query(&query_msg)?;

        if receiver.clone() != info.owner && !is_admin  {
            return Err(StdError::generic_err("Only owner or admins can resolve a market"));
        }

        if env.block.time.seconds() < info.market_end {
            return Err(StdError::generic_err("Can only resolve when market has reached deadline"));
        }
        if variant > Uint128::from(1u128) {
            return Err(StdError::generic_err("Variant must only be 0 (No) or 1 (Yes)"));
        }
        if info.resolved {
            return Err(StdError::generic_err("Market already resolved"));
        }

        if variant == Uint128::from(1u128) {
            info.yes_price = Uint128::from(MULTIPLIER);
            info.no_price = Uint128::from(0u128);
        }
        else if variant == Uint128::from(0u128) {
            info.no_price = Uint128::from(MULTIPLIER);
            info.yes_price = Uint128::from(0u128);
        }

        info.resolved = true;
        info.resolved_to = variant;

        let data: Vec<Uint128> = vec![Uint128::from(market_index)];

        INFORMATION.save(deps.storage, &info)?;

        let msg = ExecuteFactoryMsg::RecordStats {
            amount: Uint128::from(0u128),
            account: receiver.clone(),
            stat_type: String::from("resolve"),
            data
        };

        let execute_msg = WasmMsg::Execute {
            contract_addr: info.factory.to_string(),
            msg: to_json_binary(&msg)?,
            funds: vec![]
        };

        Ok(Response::new().add_message(execute_msg))

    }

    pub fn place_order(
        deps: DepsMut,
        env: Env,
        info_: MessageInfo,
        variant: Uint128,
        buy_or_sell: Uint128,
        amount: Uint128,
        receiver: Addr
    ) -> StdResult<Response> {

        let mut info = INFORMATION.load(deps.storage)?;

        let mut total_orders = TOTAL_ORDERS.load(deps.storage)?;
        total_orders += Uint128::from(1u128);

        let account = info_.sender;

        let timestamp = env.block.time.seconds();

        if timestamp >= info.market_end && timestamp < (info.market_end + RESOLVE_DURATION) {
            return Err(StdError::generic_err("Cannot trade within resolution window"));
        }

        if account != info.factory {
            return Err(StdError::generic_err("Only callable by factory"));
        }

        if amount == Uint128::from(0u128) {
            return Err(StdError::generic_err("Amount must be greater than 0"));
        }

        let mut shares: Shares = SHARES.load(deps.storage, receiver.clone()).unwrap_or_else(|_| Shares::new());

        if info.resolved {
            return Err(StdError::generic_err("Market has already been resolved"));
        }
        if variant > Uint128::from(1u128) {
            return Err(StdError::generic_err("Variant must be 1 for Yes or 0 for No"));
        }
        if buy_or_sell > Uint128::from(1u128) {
            return Err(StdError::generic_err("Variant must be 1 for Buy or 0 for Sell"));
        }

        let max_impact = Uint128::from(5 * 10u128.pow(2u32)); // 5% max price impact

        let mut messages = vec![];

        let mut execute_msg:WasmMsg = WasmMsg::Execute {
            contract_addr: env.contract.address.clone().to_string(),
            msg: Binary(vec![]),
            funds: vec![],
        };

        let msg = QueryFactoryMsg::FeesAddress {};
        
        let query_msg = QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: info.factory.to_string(),
            msg: to_json_binary(&msg)?,
        });

        let fees_address: Addr = deps.querier.query(&query_msg)?;

        if variant == Uint128::from(1u128) { // Yes

            if buy_or_sell == Uint128::from(1u128) { // Buy

                let quote:Quote = quote(deps.as_ref(), env.clone(), variant, buy_or_sell, amount)?;

                if quote.impact > max_impact {
                    return Err(StdError::generic_err("Price impact must not be more than 5%"));
                }

                let data:Vec<Uint128> = vec![variant, buy_or_sell, info.yes_price];

                let msg = ExecuteFactoryMsg::RecordStats {
                    amount,
                    account: receiver.clone(),
                    stat_type: String::from("volume"),
                    data
                };
        
                execute_msg = WasmMsg::Execute {
                    contract_addr: info.factory.to_string(),
                    msg: to_json_binary(&msg)?,
                    funds: vec![]
                };

                shares.yes_shares += quote.amount_out;

                SHARES.save(deps.storage, receiver.clone(), &shares)?;

                let mut fee_shares: Shares = SHARES.load(deps.storage, fees_address.clone()).unwrap_or_else(|_| Shares::new());

                fee_shares.yes_shares += quote.fees / Uint128::from(2u128);
                info.yes_shares += quote.amount_out + (quote.fees / Uint128::from(2u128));

                SHARES.save(deps.storage, fees_address.clone(), &fee_shares)?;

                info.yes_liquidity += amount;
                info.yes_price = quote.price;
                info.no_price = Uint128::from(MULTIPLIER) - quote.price;

                ORDER_LIST.save(deps.storage, total_orders.u128(), &Order{
                    timestamp,
                    price: info.yes_price
                })?;

            }
            else if buy_or_sell == Uint128::from(0u128) { // Sell

                shares.yes_shares -= amount;

                info.yes_shares -= amount;
                
                let quote:Quote = quote(deps.as_ref(), env, variant, buy_or_sell, amount)?;

                if quote.impact > max_impact {
                    return Err(StdError::generic_err("Price impact must not be more than 5%"));
                }

                let data: Vec<Uint128> = vec![variant, buy_or_sell, info.yes_price];

                let msg = ExecuteFactoryMsg::RecordStats {
                    amount: quote.amount_out,
                    account: receiver.clone(),
                    stat_type: String::from("volume"),
                    data
                };
        
                execute_msg = WasmMsg::Execute {
                    contract_addr: info.factory.to_string(),
                    msg: to_json_binary(&msg)?,
                    funds: vec![]
                };

                info.yes_liquidity -= quote.amount_out + (quote.fees / Uint128::from(2u128));

                info.yes_price = quote.price;

                info.no_price = Uint128::from(MULTIPLIER) - quote.price;

                ORDER_LIST.save(deps.storage, total_orders.u128(), &Order{
                    timestamp,
                    price: info.yes_price
                })?;

                SHARES.save(deps.storage, receiver.clone(), &shares)?;

                let xfer_fees = Coin {
                    denom: info.usdc.clone(),
                    amount: (quote.fees / Uint128::from(2u128))
                };
        
                let transfer_to_fees_address = CosmosMsg::Bank(BankMsg::Send {
                    to_address: fees_address.clone().to_string(),
                    amount: vec![xfer_fees],
                });

                messages.push(transfer_to_fees_address);

                let xfer_account = Coin {
                    denom: info.usdc.clone(),
                    amount: quote.amount_out.clone()
                };
        
                let transfer_to_account = CosmosMsg::Bank(BankMsg::Send {
                    to_address: receiver.clone().to_string(),
                    amount: vec![xfer_account],
                });

                messages.push(transfer_to_account);

            }

        }
        else if variant == Uint128::from(0u128) { // No

            if buy_or_sell == Uint128::from(1u128) { // Buy

                let quote:Quote = quote(deps.as_ref(), env.clone(), variant, buy_or_sell, amount)?;

                if quote.impact > max_impact {
                    return Err(StdError::generic_err("Price impact must not be more than 5%"));
                }

                let data: Vec<Uint128> = vec![variant, buy_or_sell, info.no_price];

                let msg = ExecuteFactoryMsg::RecordStats {
                    amount,
                    account: receiver.clone(),
                    stat_type: String::from("volume"),
                    data
                };
        
                execute_msg = WasmMsg::Execute {
                    contract_addr: info.factory.to_string(),
                    msg: to_json_binary(&msg)?,
                    funds: vec![]
                };

                shares.no_shares += quote.amount_out;

                SHARES.save(deps.storage, receiver.clone(), &shares)?;

                let mut fee_shares: Shares = SHARES.load(deps.storage, fees_address.clone()).unwrap_or_else(|_| Shares::new());

                fee_shares.no_shares += quote.fees / Uint128::from(2u128);
                info.no_shares += quote.amount_out + (quote.fees / Uint128::from(2u128));

                SHARES.save(deps.storage, fees_address.clone(), &fee_shares)?;

                info.no_liquidity += amount;
                info.no_price = quote.price;
                info.yes_price = Uint128::from(MULTIPLIER) - quote.price;

                ORDER_LIST.save(deps.storage, total_orders.u128(), &Order{
                    timestamp,
                    price: info.yes_price
                })?;

            }
            else if buy_or_sell == Uint128::from(0u128) { // Sell

                shares.no_shares -= amount;

                info.no_shares -= amount;
                
                let quote:Quote = quote(deps.as_ref(), env, variant, buy_or_sell, amount)?;

                if quote.impact > max_impact {
                    return Err(StdError::generic_err("Price impact must not be more than 5%"));
                }

                let data: Vec<Uint128> = vec![variant, buy_or_sell, info.no_price];

                let msg = ExecuteFactoryMsg::RecordStats {
                    amount: quote.amount_out,
                    account: receiver.clone(),
                    stat_type: String::from("volume"),
                    data
                };
        
                execute_msg = WasmMsg::Execute {
                    contract_addr: info.factory.to_string(),
                    msg: to_json_binary(&msg)?,
                    funds: vec![]
                };

                SHARES.save(deps.storage, receiver.clone(), &shares)?;

                info.no_liquidity -= quote.amount_out + (quote.fees / Uint128::from(2u128));
                info.no_price = quote.price;
                info.yes_price = Uint128::from(MULTIPLIER) - quote.price;

                ORDER_LIST.save(deps.storage, total_orders.u128(), &Order{
                    timestamp,
                    price: info.yes_price
                })?;

                let xfer_fees = Coin {
                    denom: info.usdc.clone(),
                    amount: (quote.fees / Uint128::from(2u128))
                };
        
                let transfer_to_fees_address = CosmosMsg::Bank(BankMsg::Send {
                    to_address: fees_address.clone().to_string(),
                    amount: vec![xfer_fees],
                });

                messages.push(transfer_to_fees_address);

                let xfer_account = Coin {
                    denom: info.usdc.clone(),
                    amount: quote.amount_out.clone()
                };
        
                let transfer_to_account = CosmosMsg::Bank(BankMsg::Send {
                    to_address: receiver.clone().to_string(),
                    amount: vec![xfer_account],
                });

                messages.push(transfer_to_account);

            }

        }

        INFORMATION.save(deps.storage, &info)?;

        TOTAL_ORDERS.save(deps.storage, &total_orders)?;

        let response = if messages.len() > 0 {
            Response::new().add_messages(messages).add_message(execute_msg)
        }
        else {
            Response::new().add_message(execute_msg)
        };

        Ok(response)

    }

    pub fn calculate_impact(amount_a: Uint128, amount_b: Uint128) -> Uint128 {
        let new_sum = amount_a + amount_b;
        let ratio = (Uint128::from(10u128.pow(4u32)) * new_sum) / amount_a;
        ratio - Uint128::from(10u128.pow(4u32))
    }

    pub fn quote(deps: Deps, _env: Env, variant: Uint128, buy_or_sell: Uint128, amount: Uint128) -> StdResult<Quote> {

        let mut amount_out = Uint128::from(0u128);
        let mut impact = Uint128::from(0u128);
        let mut price = Uint128::from(0u128);
        let mut fees = Uint128::from(0u128);

        let info = INFORMATION.load(deps.storage)?;

        if variant > Uint128::from(1u128) {
            return Err(StdError::generic_err("Variant must be 1 for Yes or 0 for No"));
        }
        if buy_or_sell > Uint128::from(1u128) {
            return Err(StdError::generic_err("Variant must be 1 for Buy or 0 for Sell"));
        }

        if variant == Uint128::from(1u128) { // Yes

            if buy_or_sell == Uint128::from(1u128) { // Buy

                impact = calculate_impact(info.yes_liquidity, amount);

                let new_yes_liquidity = info.yes_liquidity + amount;
                let new_no_liquidity = info.no_liquidity;

                let new_yes_price = (Uint128::from(MULTIPLIER) * new_yes_liquidity) / (new_yes_liquidity + new_no_liquidity);

                price = new_yes_price;

                let output = (amount * Uint128::from(MULTIPLIER)) / new_yes_price;

                amount_out = output;

                fees = (Uint128::from(2u128) * amount_out) / Uint128::from(100u128);

                amount_out -= fees;
                
            }
            else if buy_or_sell == Uint128::from(0u128) { // Sell

                let expected_amount = (amount * info.yes_price) / Uint128::from(MULTIPLIER);
                impact = calculate_impact(info.yes_liquidity, expected_amount);

                let new_yes_liquidity = info.yes_liquidity - expected_amount;
                let new_no_liquidity = info.no_liquidity;

                let new_yes_price = (Uint128::from(MULTIPLIER) * new_yes_liquidity) / (new_yes_liquidity + new_no_liquidity);

                price = new_yes_price;

                let output = (amount * new_yes_price) / Uint128::from(MULTIPLIER);

                fees = (Uint128::from(2u128) * output) / Uint128::from(100u128);

                amount_out = output - fees;

            }

        }
        else if variant == Uint128::from(0u128) { // No
            
            if buy_or_sell == Uint128::from(1u128) { // Buy

                impact = calculate_impact(info.no_liquidity, amount);

                let new_no_liquidity = info.no_liquidity + amount;
                let new_yes_liquidity = info.yes_liquidity;

                let new_no_price = (Uint128::from(MULTIPLIER) * new_no_liquidity) / (new_no_liquidity + new_yes_liquidity);

                price = new_no_price;

                let output = (amount * Uint128::from(MULTIPLIER)) / new_no_price;

                amount_out = output;

                fees = (Uint128::from(2u128) * amount_out) / Uint128::from(100u128);

                amount_out -= fees;

            }
            else if buy_or_sell == Uint128::from(0u128) { // Sell

                let expected_amount = (amount * info.no_price) / Uint128::from(MULTIPLIER);

                impact = calculate_impact(info.no_liquidity, expected_amount);

                let new_no_liquidity = info.no_liquidity - expected_amount;
                let new_yes_liquidity = info.yes_liquidity;

                let new_no_price = (Uint128::from(MULTIPLIER) * new_no_liquidity) / (new_no_liquidity + new_yes_liquidity);

                price = new_no_price;

                let output = (amount * new_no_price) / Uint128::from(MULTIPLIER);

                fees = (Uint128::from(2u128) * output) / Uint128::from(100u128);

                amount_out = output - fees;

            }

        }

        let quote = Quote {
            amount_out,
            impact,
            price,
            fees
        };

        Ok(quote)
    }

}
