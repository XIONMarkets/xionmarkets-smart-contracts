FACTORY_ADDRESS=xion1t3c2daahluryrf66ec47fjasfp7xtnrfm9f0xdkz4jenkmaxfk6sx20kkk
USDC_ADDRESS=xion1kgq2pmddmwxqsz8rdqp4rzg6uvt6dkwg3hr423psvprd65tgrukqqf9y6r
#MARKET_ADDRESS=xion1sm8407qlm24p5f7507vgta6kn2exve25sd57evmqa6cfj0ks63lsxz0feq
SENDER=xion13pt0cc57lf943wqjd3ges78mcmulwden5ssy80
FAUCET_ADDRESS=xion1d5j46m5t2z9e3j03esxgjgm2f0xp30q5fdfqfz
ADMIN_ADDRESS=xion1jvjzsfq4rhs9xgmqv6f34jpt6dnmc9dmlcthcmlkeh6fzhfydvkqgdeua7

# Create a market

#CREATE='{
#    "create_market": {
#        "title":"Will Bitcoin get to 100k USD on or before the end of 2024?",
#        "description":"Bitcoin has been on the run since Trump won the US elections against Kamala Harris. We cannot tell for sure if Bitcoin will advance to 100k USD, but well, it is a possibility. Show us your betting prowess in this!",
#        "end_date":1742878729,
#        "categories":["finance","crypto"],
#        "media":["https://jade-particular-kite-725.mypinata.cloud/ipfs/QmNYEjhZ6trWYC1nLoT5cHKiyKLtSeuwyBAyWsCXiTeBye","https://jade-particular-kite-725.mypinata.cloud/ipfs/QmT6hoguXM5xaneUrVUXKpHGBxczZ9kEynhXxCPa6bDQ82"]
#    }
#}'

#xiond tx wasm execute $FACTORY_ADDRESS "$CREATE" \
#    --from $SENDER \
#    --gas-prices 0.025uxion \
#    --gas auto \
#    --gas-adjustment 1.3 \
#    --chain-id xion-testnet-1 \
#    --node https://rpc.xion-testnet-1.burnt.com:443


# Get platform statistics

GET_STATISTICS='{
    "get_statistics": {}
}'

xiond query wasm contract-state smart $FACTORY_ADDRESS "$GET_STATISTICS" \
    --node https://rpc.xion-testnet-1.burnt.com:443 \
    --output json


# Get USDC Balance

USDC_BALANCE='{
    "balance": {
        "address":"'$SENDER'"
    }
}'

xiond query wasm contract-state smart $USDC_ADDRESS "$USDC_BALANCE" \
    --node https://rpc.xion-testnet-1.burnt.com:443 \
    --output json


# Add admin

#ADD_ADMIN='{
#    "add_admin": {
#        "account":"'$ADMIN_ADDRESS'"
#    }
#}
#'

#xiond tx wasm execute $FACTORY_ADDRESS "$ADD_ADMIN" \
#    --from $SENDER \
#    --gas-prices 0.025uxion \
#    --gas auto \
#    --gas-adjustment 1.3 \
#    --chain-id xion-testnet-1 \
#    --node https://rpc.xion-testnet-1.burnt.com:443



# Transfer USDC

#TRANSFER_USDC='{
#    "transfer": {
#        "recipient":"'$ADMIN_ADDRESS'",
#        "amount":"10000000000000000000000000"
#    }
#}
#'

#xiond tx wasm execute $USDC_ADDRESS "$TRANSFER_USDC" \
#    --from $SENDER \
#    --gas-prices 0.025uxion \
#    --gas auto \
#    --gas-adjustment 1.3 \
#    --chain-id xion-testnet-1 \
#    --node https://rpc.xion-testnet-1.burnt.com:443


# Paginated query of markets

#FETCH_MARKETS='{
#    "fetch_markets": {
#        "page":"1",
#        "items_per_page":"1",
#        "account":"'$SENDER'",
#        "market_type":"0"
#    }
#}'

#xiond query wasm contract-state smart $FACTORY_ADDRESS "$FETCH_MARKETS" \
#    --node https://rpc.xion-testnet-1.burnt.com:443 \
#    --output json


# Approve USDC spending by factory contract

#APPROVE='{
#    "increase_allowance": {
#        "spender":"'$FACTORY_ADDRESS'",
#        "amount":"1000000000000",
#        "expires":{
#           "never":{}
#        }
#    }
#}
#'

#xiond tx wasm execute $USDC_ADDRESS "$APPROVE" \
#    --from $SENDER \
#    --gas-prices 0.025uxion \
#    --gas auto \
#    --gas-adjustment 1.3 \
#    --chain-id xion-testnet-1 \
#    --node https://rpc.xion-testnet-1.burnt.com:443


# Initialize Liquidity for market

#INIT_LIQUIDITY='{
#    "initialize_liquidity": {
#        "market":"'$MARKET_ADDRESS'",
#        "yes_price":"50000000",
#        "liquidity":"5000000000"
#    }
#}
#'

#xiond tx wasm execute $FACTORY_ADDRESS "$INIT_LIQUIDITY" \
#    --from $SENDER \
#    --gas-prices 0.025uxion \
#    --gas auto \
#    --gas-adjustment 1.3 \
#    --chain-id xion-testnet-1 \
#    --node https://rpc.xion-testnet-1.burnt.com:443


# Add liquidity to market

#ADD_LIQUIDITY='{
#    "add_liquidity": {
#        "market":"'$MARKET_ADDRESS'",
#        "amount":"10000000"
#    }
#}
#'

#xiond tx wasm execute $FACTORY_ADDRESS "$ADD_LIQUIDITY" \
#    --from $SENDER \
#    --gas-prices 0.025uxion \
#    --gas auto \
#    --gas-adjustment 1.3 \
#    --chain-id xion-testnet-1 \
#    --node https://rpc.xion-testnet-1.burnt.com:443


# Get information for a specified market via contract address

#GET_MARKET_INFO='{
#    "get_market_info": {
#        "contract_address":"'$MARKET_ADDRESS'",
#        "account":"'$SENDER'"
#    }
#}'

#xiond query wasm contract-state smart $FACTORY_ADDRESS "$GET_MARKET_INFO" \
#    --node https://rpc.xion-testnet-1.burnt.com:443 \
#    --output json


# Get order quote for buy or sell orders

#QUOTE='{
#    "quote": {
#        "market":"'$MARKET_ADDRESS'",
#        "variant":"0",
#        "buy_or_sell":"1",
#        "amount":"250000000"
#    }
#}'

#xiond query wasm contract-state smart $FACTORY_ADDRESS "$QUOTE" \
#    --node https://rpc.xion-testnet-1.burnt.com:443 \
#    --output json


# Place a buy or sell order for 'Yes' or 'No'

#PLACE_ORDER='{
#    "place_order": {
#        "market":"'$MARKET_ADDRESS'",
#        "variant":"1",
#        "buy_or_sell":"1",
#        "amount":"25000000"
#    }
#}
#'

#xiond tx wasm execute $FACTORY_ADDRESS "$PLACE_ORDER" \
#    --from $SENDER \
#    --gas-prices 0.025uxion \
#    --gas auto \
#    --gas-adjustment 1.3 \
#    --chain-id xion-testnet-1 \
#    --node https://rpc.xion-testnet-1.burnt.com:443


# Remove liquidity from market

#REMOVE_LIQUIDITY='{
#    "remove_liquidity": {
#        "market":"'$MARKET_ADDRESS'",
#        "shares":"5000000"
#    }
#}
#'

#xiond tx wasm execute $FACTORY_ADDRESS "$REMOVE_LIQUIDITY" \
#    --from $SENDER \
#    --gas-prices 0.025uxion \
#    --gas auto \
#    --gas-adjustment 1.3 \
#    --chain-id xion-testnet-1 \
#    --node https://rpc.xion-testnet-1.burnt.com:443



# Resolve a market to 'Yes' or 'No'

#RESOLVE='{
#    "resolve_market": {
#        "market":"'$MARKET_ADDRESS'",
#        "variant":"1",
#        "market_index":"1"
#    }
#}
#'

#xiond tx wasm execute $FACTORY_ADDRESS "$RESOLVE" \
#    --from $SENDER \
#    --gas-prices 0.025uxion \
#    --gas auto \
#    --gas-adjustment 1.3 \
#    --chain-id xion-testnet-1 \
#    --node https://rpc.xion-testnet-1.burnt.com:443