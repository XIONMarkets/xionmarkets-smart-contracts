use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

use packages::factory::{Details, Statistics, TempInformation};

pub const ACTIVE_MARKETS: Map<u128, Addr> = Map::new("active_markets");
pub const MARKETS: Map<u128, Addr> = Map::new("all_markets");
pub const COMPLETED_MARKETS: Map<u128, Addr> = Map::new("completed_markets");
pub const VOLUMES: Map<Addr, Uint128> = Map::new("volumes");
pub const KNOWN_MARKETS: Map<Addr, bool> = Map::new("known_markets");
pub const UNIQUE_WALLETS: Map<Addr, bool> = Map::new("unique_wallets");
pub const MEDIA: Map<Addr, Vec<String>> = Map::new("media");
pub const ADMINS_MAP: Map<Addr, bool> = Map::new("admins_map");
pub const TEMP_INFORMATION: Item<TempInformation> = Item::new("information");
pub const STATISTICS: Item<Statistics> = Item::new("statistics");
pub const DETAILS: Item<Details> = Item::new("details");
pub const INCENTIVES: Map<Addr, u64> = Map::new("incentives");