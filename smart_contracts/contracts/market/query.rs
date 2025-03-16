use crate::state::{INFORMATION, SHARES, TOTAL_ORDERS, ORDER_LIST};
use cosmwasm_std::{Deps, Env, StdResult, StdError, Uint128};
use packages::market::{Information, Shares, Order, Data};

pub mod query_msg {

    use super::*;

    use cosmwasm_std::{
        Addr
    };

    pub fn get_info(deps: Deps, _env: Env, account: Addr) -> StdResult<Data> {
        let information: Information = INFORMATION.load(deps.storage).unwrap();
        let shares: Shares = SHARES.load(deps.storage, account).unwrap_or_else(|_| Shares::new());
        let data = Data {
            information,
            shares,
        };
        Ok(data)
    }

    pub fn get_orders(_deps: Deps, _env: Env, page_: u128, items_per_page: u128) -> StdResult<(u64, Vec<Order>)> {

        if page_ == 0u128 {
            return Err(StdError::generic_err("Page must be greater than 0"));
        }
        if items_per_page == 0u128 {
            return Err(StdError::generic_err("Items per page must be greater than 0"));
        }

        let total_orders = TOTAL_ORDERS.load(_deps.storage)?;

        let number_of_pages = if total_orders % Uint128::from(items_per_page) == Uint128::from(0u128) {
            total_orders / Uint128::from(items_per_page)
        }
        else {
            (total_orders / Uint128::from(items_per_page)) + Uint128::from(1u128)
        };

        let page = Uint128::from(page_);

        if page > number_of_pages {
            return Err(StdError::generic_err("Page is out of bounds"));
        }

        if total_orders == Uint128::from(0u128) {
            return Err(StdError::generic_err("No order exists at the moment"));
        }

        let start_index = total_orders - ((page - Uint128::from(1u128)) * Uint128::from(items_per_page));

        let end_index = if start_index < Uint128::from(items_per_page) {
            Uint128::from(1u128)
        }
        else {
            (start_index - Uint128::from(items_per_page)) + Uint128::from(1u128)
        };

        let number_of_items = (start_index - end_index) + Uint128::from(1u128);

        if number_of_items == Uint128::from(0u128) {
            return Err(StdError::generic_err("No order found for this page"));
        }

        let mut orders: Vec<Order> = vec![];

        let mut current_index = start_index;

        while current_index >= end_index {

            let index = current_index.u128();

            let order_at = ORDER_LIST.load(_deps.storage, index)?;

            orders.push(order_at);

            current_index = Uint128::from(current_index.u128() - 1u128);

        }

        Ok((_env.block.time.seconds(), orders))

    }

    pub fn get_total_orders(_deps: Deps, _env: Env) -> StdResult<Uint128> {

        let total_orders = TOTAL_ORDERS.load(_deps.storage)?;

        Ok(total_orders)

    }

}