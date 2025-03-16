# set the initial state of the instance
INIT='{
    "name" : "Circle USD", 
    "symbol": "USDC",
    "decimals": 6,
    "initial_balances": [
        {
            "address": "xion13pt0cc57lf943wqjd3ges78mcmulwden5ssy80",
            "amount": "100000000000000000000000000000000000"
        }
    ]
}'

# instantiate the contract

CODE_ID=1453

xiond tx wasm instantiate $CODE_ID "$INIT" \
    --from xion13pt0cc57lf943wqjd3ges78mcmulwden5ssy80 \
    --label "cw20"\
    --gas-prices 0.025uxion \
    --gas auto \
    --gas-adjustment 1.3 \
    --chain-id xion-testnet-1 \
    --node https://rpc.xion-testnet-1.burnt.com:443 \
    --no-admin

#contract_address=xion1kgq2pmddmwxqsz8rdqp4rzg6uvt6dkwg3hr423psvprd65tgrukqqf9y6r