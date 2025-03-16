use cw_storage_plus::{Item, Map};
use cosmwasm_std::{Addr, Uint128};
use packages::market::{Information, Shares, Order};

pub const INFORMATION: Item<Information> = Item::new("information");

pub const TOTAL_ORDERS: Item<Uint128> = Item::new("total_orders");

pub const ORDER_LIST: Map<u128, Order> = Map::new("order_list");

pub const SHARES: Map<Addr, Shares> = Map::new("shares");