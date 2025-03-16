# set the initial state of the instance
INIT='{
    "usdc":{
        "c_w20_token": {
            "contract_address":"xion1kgq2pmddmwxqsz8rdqp4rzg6uvt6dkwg3hr423psvprd65tgrukqqf9y6r"
        }
    },
    "fees_address":"xion13pt0cc57lf943wqjd3ges78mcmulwden5ssy80",
    "market_code_id":1953
}'

# instantiate the contract

CODE_ID=1954

xiond tx wasm instantiate $CODE_ID "$INIT" \
    --from xion13pt0cc57lf943wqjd3ges78mcmulwden5ssy80 \
    --label "factory"\
    --gas-prices 0.025uxion \
    --gas auto \
    --gas-adjustment 1.3 \
    --chain-id xion-testnet-1 \
    --node https://rpc.xion-testnet-1.burnt.com:443 \
    --no-admin

#contract_address=xion1t3c2daahluryrf66ec47fjasfp7xtnrfm9f0xdkz4jenkmaxfk6sx20kkk