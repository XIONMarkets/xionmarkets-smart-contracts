use cosmwasm_std::{Addr, Empty, Uint128, BankQuery, BalanceResponse, QueryRequest, coins};
use cw_multi_test::{App, Contract, ContractWrapper, Executor};
use packages::factory::{
    ExecuteMsg as FactoryExecuteMsg, InstantiateMsg as FactoryInstantiate
};
use packages::market::{
    Quote, Data,
    QueryMsg as MarketQueryMsg
};

fn mock_app() -> App {
    App::default()
}

const USDC_DENOM: &str = "usdc";

#[allow(dead_code)]
fn factory_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new_with_empty(
        factory::contract::execute,
        factory::contract::instantiate,
        factory::contract::query,
    ).with_reply(factory::execute::reply);
    Box::new(contract)
}

#[allow(dead_code)]
fn market_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new_with_empty(
        market::contract::execute,
        market::contract::instantiate,
        market::contract::query,
    );
    Box::new(contract)
}

#[derive(Debug)]
#[allow(dead_code)]
struct ContractInfo {
    usdc_contract_addr: String,
    factory_contract_addr: Addr
}

fn initialize_contracts(app: &mut App) -> ContractInfo {

    let factory_code_id = app.store_code(factory_contract());
    println!("Factory code id: {}", factory_code_id);

    let market_code_id = app.store_code(market_contract());
    println!("market code id: {}", market_code_id);

    let usdc = String::from(USDC_DENOM);

    let factory_contract_addr = app
        .instantiate_contract(
            factory_code_id,
            Addr::unchecked("user"),
            &FactoryInstantiate {
                usdc: usdc.clone(),
                fees_address: Addr::unchecked("fees"),
                market_code_id
            },
            &[],
            "Instantiate Factory",
            Some("user".to_string()),
        )
        .unwrap();

    ContractInfo {
        usdc_contract_addr: usdc,
        factory_contract_addr
    }
}

#[test]
fn initialization_test() {
    let mut app = mock_app();
    initialize_contracts(&mut app);
}

fn create_market(
    app: &mut App,
    factory_contract_addr: Addr
) -> String {

    let create_market_res = app
        .execute_contract(
            Addr::unchecked("user"),
            factory_contract_addr,
            &FactoryExecuteMsg::CreateMarket {
                title: "Will BTC reach $100,000?".to_string(),
                description: "Bet on this market today!".to_string(),
                end_date: 12456788910111213,
                categories: vec!["finance".to_string(), "crypto".to_string()],
                media: ["https://site.com/assets/media-0.png".to_string(), "https://site.com/assets/media-1.png".to_string()]
            },
            &[],
        )
        .unwrap();
        
    let market_address = create_market_res.events[1].attributes[0].value.clone();

    return market_address;

}

#[test]
fn create_market_test() {

    let mut app = mock_app();

    let contract_info = initialize_contracts(&mut app);

    let market_address = create_market(
        &mut app,
        contract_info.factory_contract_addr
    );

    let result: Data = app
    .wrap()
    .query_wasm_smart(
        Addr::unchecked(market_address.clone()),
        &MarketQueryMsg::GetInfo {
            account: Addr::unchecked("user")
        },
    )
    .unwrap();

    assert_eq!(result.information.title, "Will BTC reach $100,000?");

}


#[test]
fn initialize_liquidity_test() {

    let mut app = mock_app();

    let contract_info = initialize_contracts(&mut app);

    let factory_address = contract_info.factory_contract_addr.clone().to_string();

    let market_address = create_market(
        &mut app,
        Addr::unchecked(factory_address.clone())
    );

    let usdc_denom = String::from(USDC_DENOM);
    app.sudo(cw_multi_test::SudoMsg::Bank(cw_multi_test::BankSudo::Mint {
        to_address: "user".to_string(),
        amount: coins(10_000_000_000, &usdc_denom),
    }))
    .unwrap();

    app
    .execute_contract(
        Addr::unchecked("user"),
        Addr::unchecked(factory_address.clone()),
        &FactoryExecuteMsg::InitializeLiquidity {
            market: Addr::unchecked(market_address.clone()),
            yes_price: Uint128::from(50_000_000u128),
            liquidity: Uint128::from(1_000_000_000u128)
        },
        &coins(1_000_000_000, &usdc_denom),
    )
    .unwrap();

    let result: Data = app
    .wrap()
    .query_wasm_smart(
        Addr::unchecked(market_address.clone()),
        &MarketQueryMsg::GetInfo {
            account: Addr::unchecked("user")
        },
    )
    .unwrap();

    assert_eq!(result.information.yes_liquidity, Uint128::from(500_000_000u128));

}


#[test]
fn add_liquidity_test() {

    let mut app = mock_app();

    let contract_info = initialize_contracts(&mut app);

    let factory_address = contract_info.factory_contract_addr.clone().to_string();

    let market_address = create_market(
        &mut app,
        Addr::unchecked(factory_address.clone())
    );

    let usdc_denom = String::from(USDC_DENOM);
    app.sudo(cw_multi_test::SudoMsg::Bank(cw_multi_test::BankSudo::Mint {
        to_address: "user".to_string(),
        amount: coins(10_000_000_000, &usdc_denom),
    }))
    .unwrap();

    app
    .execute_contract(
        Addr::unchecked("user"),
        Addr::unchecked(factory_address.clone()),
        &FactoryExecuteMsg::InitializeLiquidity {
            market: Addr::unchecked(market_address.clone()),
            yes_price: Uint128::from(50_000_000u128),
            liquidity: Uint128::from(1_000_000_000u128)
        },
        &coins(1_000_000_000, &usdc_denom),
    )
    .unwrap();

    app
    .execute_contract(
        Addr::unchecked("user"),
        Addr::unchecked(factory_address.clone()),
        &FactoryExecuteMsg::AddLiquidity {
            market: Addr::unchecked(market_address.clone()),
            amount: Uint128::from(1_000_000_000u128)
        },
        &coins(1_000_000_000, &usdc_denom),
    )
    .unwrap();

    let result: Data = app
    .wrap()
    .query_wasm_smart(
        Addr::unchecked(market_address.clone()),
        &MarketQueryMsg::GetInfo {
            account: Addr::unchecked("user")
        },
    )
    .unwrap();

    assert_eq!(result.information.yes_liquidity, Uint128::from(1_000_000_000u128));

}


#[test]
fn remove_liquidity_test() {

    let mut app = mock_app();

    let contract_info = initialize_contracts(&mut app);

    let factory_address = contract_info.factory_contract_addr.clone().to_string();

    let market_address = create_market(
        &mut app,
        Addr::unchecked(factory_address.clone())
    );

    let usdc_denom = String::from(USDC_DENOM);
    app.sudo(cw_multi_test::SudoMsg::Bank(cw_multi_test::BankSudo::Mint {
        to_address: "user".to_string(),
        amount: coins(10_000_000_000, &usdc_denom),
    }))
    .unwrap();

    app
    .execute_contract(
        Addr::unchecked("user"),
        Addr::unchecked(factory_address.clone()),
        &FactoryExecuteMsg::InitializeLiquidity {
            market: Addr::unchecked(market_address.clone()),
            yes_price: Uint128::from(50_000_000u128),
            liquidity: Uint128::from(1_000_000_000u128)
        },
        &coins(1_000_000_000, &usdc_denom),
    )
    .unwrap();

    app
    .execute_contract(
        Addr::unchecked("user"),
        Addr::unchecked(factory_address.clone()),
        &FactoryExecuteMsg::AddLiquidity {
            market: Addr::unchecked(market_address.clone()),
            amount: Uint128::from(1_000_000_000u128)
        },
        &coins(1_000_000_000, &usdc_denom),
    )
    .unwrap();

    app
    .execute_contract(
        Addr::unchecked("user"),
        Addr::unchecked(factory_address.clone()),
        &FactoryExecuteMsg::RemoveLiquidity {
            market: Addr::unchecked(market_address.clone()),
            shares: Uint128::from(500_000_000u128)
        },
        &[],
    )
    .unwrap();

    let result: Data = app
    .wrap()
    .query_wasm_smart(
        Addr::unchecked(market_address.clone()),
        &MarketQueryMsg::GetInfo {
            account: Addr::unchecked("user")
        },
    )
    .unwrap();

    assert_eq!(result.information.yes_liquidity, Uint128::from(500_000_000u128));

}


#[test]
fn buy_yes_test() {

    let mut app = mock_app();

    let contract_info = initialize_contracts(&mut app);

    let factory_address = contract_info.factory_contract_addr.clone().to_string();

    let market_address = create_market(
        &mut app,
        Addr::unchecked(factory_address.clone())
    );

    let usdc_denom = String::from(USDC_DENOM);
    app.sudo(cw_multi_test::SudoMsg::Bank(cw_multi_test::BankSudo::Mint {
        to_address: "user".to_string(),
        amount: coins(10_000_000_000, &usdc_denom),
    }))
    .unwrap();

    app
    .execute_contract(
        Addr::unchecked("user"),
        Addr::unchecked(factory_address.clone()),
        &FactoryExecuteMsg::InitializeLiquidity {
            market: Addr::unchecked(market_address.clone()),
            yes_price: Uint128::from(50_000_000u128),
            liquidity: Uint128::from(1_000_000_000u128)
        },
        &coins(1_000_000_000, &usdc_denom),
    )
    .unwrap();

    let shares_quote: Quote = app
        .wrap()
        .query_wasm_smart(
            market_address.clone(),
            &MarketQueryMsg::Quote {
                variant: Uint128::from(1u128),
                buy_or_sell: Uint128::from(1u128),
                amount: Uint128::from(15_000_000u128)
            },
        )
        .unwrap();

    let shares_out = shares_quote.amount_out;

    app
    .execute_contract(
        Addr::unchecked("user"),
        Addr::unchecked(factory_address.clone()),
        &FactoryExecuteMsg::PlaceOrder {
            market: Addr::unchecked(market_address.clone()),
            variant: Uint128::from(1u128),
            buy_or_sell: Uint128::from(1u128),
            amount: Uint128::from(15_000_000u128)
        },
        &coins(15_000_000, &usdc_denom),
    )
    .unwrap();

    let result: Data = app
    .wrap()
    .query_wasm_smart(
        Addr::unchecked(market_address.clone()),
        &MarketQueryMsg::GetInfo {
            account: Addr::unchecked("user")
        },
    )
    .unwrap();

    println!("BUY YES => Information: {:?}", result.information);

    assert_eq!(result.shares.yes_shares, shares_out);

}

#[test]
fn sell_yes_test() {

    let mut app = mock_app();

    let contract_info = initialize_contracts(&mut app);

    let factory_address = contract_info.factory_contract_addr.clone().to_string();

    let market_address = create_market(
        &mut app,
        Addr::unchecked(factory_address.clone())
    );

    let usdc_denom = String::from(USDC_DENOM);
    app.sudo(cw_multi_test::SudoMsg::Bank(cw_multi_test::BankSudo::Mint {
        to_address: "user".to_string(),
        amount: coins(10_000_000_000, &usdc_denom),
    }))
    .unwrap();

    app
    .execute_contract(
        Addr::unchecked("user"),
        Addr::unchecked(factory_address.clone()),
        &FactoryExecuteMsg::InitializeLiquidity {
            market: Addr::unchecked(market_address.clone()),
            yes_price: Uint128::from(50_000_000u128),
            liquidity: Uint128::from(1_000_000_000u128)
        },
        &coins(1_000_000_000, &usdc_denom),
    )
    .unwrap();

    let shares_quote: Quote = app
        .wrap()
        .query_wasm_smart(
            market_address.clone(),
            &MarketQueryMsg::Quote {
                variant: Uint128::from(1u128),
                buy_or_sell: Uint128::from(1u128),
                amount: Uint128::from(15_000_000u128)
            },
        )
        .unwrap();

    let shares_out = shares_quote.amount_out;

    app
    .execute_contract(
        Addr::unchecked("user"),
        Addr::unchecked(factory_address.clone()),
        &FactoryExecuteMsg::PlaceOrder {
            market: Addr::unchecked(market_address.clone()),
            variant: Uint128::from(1u128),
            buy_or_sell: Uint128::from(1u128),
            amount: Uint128::from(15_000_000u128)
        },
        &coins(15_000_000, &usdc_denom),
    )
    .unwrap();

    let usdc_quote: Quote = app
        .wrap()
        .query_wasm_smart(
            market_address.clone(),
            &MarketQueryMsg::Quote {
                variant: Uint128::from(1u128),
                buy_or_sell: Uint128::from(0u128),
                amount: shares_out.clone()
            },
        )
        .unwrap();

    let usdc_out = usdc_quote.amount_out;

    app
    .execute_contract(
        Addr::unchecked("user"),
        Addr::unchecked(factory_address.clone()),
        &FactoryExecuteMsg::PlaceOrder {
            market: Addr::unchecked(market_address.clone()),
            variant: Uint128::from(1u128),
            buy_or_sell: Uint128::from(0u128),
            amount: shares_out.clone()
        },
        &[],
    )
    .unwrap();

    let expected_remaining_balance = Uint128::from(10_000_000_000u128 - 1_000_000_000u128 - 15_000_000u128 + usdc_out.u128());

    let result: BalanceResponse = app
    .wrap()
    .query(&QueryRequest::Bank(
        BankQuery::Balance {
            address: "user".to_string(),
            denom: USDC_DENOM.to_string(),
        },
    ))
    .unwrap();

    let info: Data = app
    .wrap()
    .query_wasm_smart(
        Addr::unchecked(market_address.clone()),
        &MarketQueryMsg::GetInfo {
            account: Addr::unchecked("user")
        },
    )
    .unwrap();

    println!("SELL YES => Information: {:?}", info.information);

    assert_eq!(result.amount.amount, expected_remaining_balance);

}

#[test]
fn buy_no_test() {

    let mut app = mock_app();

    let contract_info = initialize_contracts(&mut app);

    let factory_address = contract_info.factory_contract_addr.clone().to_string();

    let market_address = create_market(
        &mut app,
        Addr::unchecked(factory_address.clone())
    );

    let usdc_denom = String::from(USDC_DENOM);
    app.sudo(cw_multi_test::SudoMsg::Bank(cw_multi_test::BankSudo::Mint {
        to_address: "user".to_string(),
        amount: coins(10_000_000_000, &usdc_denom),
    }))
    .unwrap();

    app
    .execute_contract(
        Addr::unchecked("user"),
        Addr::unchecked(factory_address.clone()),
        &FactoryExecuteMsg::InitializeLiquidity {
            market: Addr::unchecked(market_address.clone()),
            yes_price: Uint128::from(50_000_000u128),
            liquidity: Uint128::from(1_000_000_000u128)
        },
        &coins(1_000_000_000, &usdc_denom),
    )
    .unwrap();

    let shares_quote: Quote = app
        .wrap()
        .query_wasm_smart(
            market_address.clone(),
            &MarketQueryMsg::Quote {
                variant: Uint128::from(0u128),
                buy_or_sell: Uint128::from(1u128),
                amount: Uint128::from(15_000_000u128)
            },
        )
        .unwrap();

    let shares_out = shares_quote.amount_out;

    app
    .execute_contract(
        Addr::unchecked("user"),
        Addr::unchecked(factory_address.clone()),
        &FactoryExecuteMsg::PlaceOrder {
            market: Addr::unchecked(market_address.clone()),
            variant: Uint128::from(0u128),
            buy_or_sell: Uint128::from(1u128),
            amount: Uint128::from(15_000_000u128)
        },
        &coins(15_000_000, &usdc_denom),
    )
    .unwrap();

    let result: Data = app
    .wrap()
    .query_wasm_smart(
        Addr::unchecked(market_address.clone()),
        &MarketQueryMsg::GetInfo {
            account: Addr::unchecked("user")
        },
    )
    .unwrap();

    println!("BUY NO => Information: {:?}", result.information);

    assert_eq!(result.shares.no_shares, shares_out);

}

#[test]
fn sell_no_test() {

    let mut app = mock_app();

    let contract_info = initialize_contracts(&mut app);

    let factory_address = contract_info.factory_contract_addr.clone().to_string();

    let market_address = create_market(
        &mut app,
        Addr::unchecked(factory_address.clone())
    );

    let usdc_denom = String::from(USDC_DENOM);
    app.sudo(cw_multi_test::SudoMsg::Bank(cw_multi_test::BankSudo::Mint {
        to_address: "user".to_string(),
        amount: coins(10_000_000_000, &usdc_denom),
    }))
    .unwrap();

    app
    .execute_contract(
        Addr::unchecked("user"),
        Addr::unchecked(factory_address.clone()),
        &FactoryExecuteMsg::InitializeLiquidity {
            market: Addr::unchecked(market_address.clone()),
            yes_price: Uint128::from(50_000_000u128),
            liquidity: Uint128::from(1_000_000_000u128)
        },
        &coins(1_000_000_000, &usdc_denom),
    )
    .unwrap();

    let shares_quote: Quote = app
        .wrap()
        .query_wasm_smart(
            market_address.clone(),
            &MarketQueryMsg::Quote {
                variant: Uint128::from(0u128),
                buy_or_sell: Uint128::from(1u128),
                amount: Uint128::from(15_000_000u128)
            },
        )
        .unwrap();

    let shares_out = shares_quote.amount_out;

    app
    .execute_contract(
        Addr::unchecked("user"),
        Addr::unchecked(factory_address.clone()),
        &FactoryExecuteMsg::PlaceOrder {
            market: Addr::unchecked(market_address.clone()),
            variant: Uint128::from(0u128),
            buy_or_sell: Uint128::from(1u128),
            amount: Uint128::from(15_000_000u128)
        },
        &coins(15_000_000, &usdc_denom),
    )
    .unwrap();

    let usdc_quote: Quote = app
        .wrap()
        .query_wasm_smart(
            market_address.clone(),
            &MarketQueryMsg::Quote {
                variant: Uint128::from(0u128),
                buy_or_sell: Uint128::from(0u128),
                amount: shares_out.clone()
            },
        )
        .unwrap();

    let usdc_out = usdc_quote.amount_out;

    app
    .execute_contract(
        Addr::unchecked("user"),
        Addr::unchecked(factory_address.clone()),
        &FactoryExecuteMsg::PlaceOrder {
            market: Addr::unchecked(market_address.clone()),
            variant: Uint128::from(0u128),
            buy_or_sell: Uint128::from(0u128),
            amount: shares_out.clone()
        },
        &[],
    )
    .unwrap();

    let expected_remaining_balance = Uint128::from(10_000_000_000u128 - 1_000_000_000u128 - 15_000_000u128 + usdc_out.u128());

    let result: BalanceResponse = app
    .wrap()
    .query(&QueryRequest::Bank(
        BankQuery::Balance {
            address: "user".to_string(),
            denom: USDC_DENOM.to_string(),
        },
    ))
    .unwrap();

    let info: Data = app
    .wrap()
    .query_wasm_smart(
        Addr::unchecked(market_address.clone()),
        &MarketQueryMsg::GetInfo {
            account: Addr::unchecked("user")
        },
    )
    .unwrap();

    println!("SELL NO => Information: {:?}", info.information);

    assert_eq!(result.amount.amount, expected_remaining_balance);

}