USER_PEM="~/wallets/development.pem"
PROXY="http://192.168.1.29:7950"
CHAIN_ID="local-testnet"
CONTRACT_ADDRESS="erd1qqqqqqqqqqqqqpgqdhas4g0qfldpjcpm7mprxuce0vmc3prf4jwst8e2le"

deploy() {
    mxpy --verbose contract deploy --bytecode="output/staking.wasm" \
    --recall-nonce --pem=${USER_PEM} \
    --gas-limit=100000000 \
    --send --wait-result \
    --proxy=${PROXY} --chain=${CHAIN_ID} || return
}

build() {
    mxpy contract build
}

upgrade() {
    mxpy contract upgrade ${CONTRACT_ADDRESS} --project=${PROJECT} \
    --recall-nonce --pem=${USER_PEM} \
    --gas-limit=120000000 \
    --send --wait-result \
    --proxy=${PROXY} --chain=${CHAIN_ID} || return
}

call() {
    mxpy contract call ${CONTRACT_ADDRESS} \
    --recall-nonce --pem=${USER_PEM} \
    --gas-limit=20000000 \
    --function "unstake_multiple" \
    --arguments 6 \
    --send --wait-result \
    --proxy=${PROXY} --chain=${CHAIN_ID} || return
}

getgenerico() {
    echo User Staking Positions
    erdpy contract query ${CONTRACT_ADDRESS} \
    --function "getUserStaking" --arguments erd19hcnc2djsjay3prvhuzr0phveducv93khj435pqjza73tcyu4jwsuqywdh \
    --proxy=${PROXY} || return

    echo User Rewards
    erdpy contract query ${CONTRACT_ADDRESS} \
    --function "calculateRewardsForUser" --arguments erd19hcnc2djsjay3prvhuzr0phveducv93khj435pqjza73tcyu4jwsuqywdh \
    --proxy=${PROXY} || return
}


set_admin() {
    mxpy contract call ${CONTRACT_ADDRESS} \
    --recall-nonce --pem=${USER_PEM} \
    --gas-limit=20000000 \
    --function "set_admin" \
    --arguments erd1e9nnlhee5rvn9k8y3ysmtsx490h8nw9jels5vjdlezl72yrtg3yq6vvj3r \
    --send --wait-result \
    --proxy=${PROXY} --chain=${CHAIN_ID} || return
}
set_reward_token() {
    mxpy contract call ${CONTRACT_ADDRESS} \
    --recall-nonce --pem=${USER_PEM} \
    --gas-limit=20000000 \
    --function "set_reward_token" \
    --arguments str:TOKTE-a9357c \
    --send --wait-result \
    --proxy=${PROXY} --chain=${CHAIN_ID} || return
}
set_staking_token() {
    mxpy contract call ${CONTRACT_ADDRESS} \
    --recall-nonce --pem=${USER_PEM} \
    --gas-limit=20000000 \
    --function "set_staking_token" \
    --arguments str:TEST-a2d9b7 \
    --send --wait-result \
    --proxy=${PROXY} --chain=${CHAIN_ID} || return
}
set_tokens_per_day() {
    mxpy contract call ${CONTRACT_ADDRESS} \
    --recall-nonce --pem=${USER_PEM} \
    --gas-limit=20000000 \
    --function "set_tokens_per_day" \
    --arguments 10000000000000000000 \
    --send --wait-result \
    --proxy=${PROXY} --chain=${CHAIN_ID} || return
}
deposit_rewards() {
    mxpy contract call ${CONTRACT_ADDRESS} \
    --recall-nonce --pem=${USER_PEM} \
    --gas-limit=20000000 \
    --function "ESDTTransfer" \
    --arguments str:TOKTE-a9357c 10000000000000000000000 str:deposit_rewards \
    --send --wait-result \
    --proxy=${PROXY} --chain=${CHAIN_ID} || return
}
stake() {
    mxpy contract call erd19hcnc2djsjay3prvhuzr0phveducv93khj435pqjza73tcyu4jwsuqywdh \
    --recall-nonce --pem=${USER_PEM} \
    --gas-limit=20000000 \
    --function "ESDTNFTTransfer" \
    --arguments str:TEST-a2d9b7 2 1 ${CONTRACT_ADDRESS} str:stake \
    --send --wait-result \
    --proxy=${PROXY} --chain=${CHAIN_ID} || return
}
unstake() {
    mxpy contract call ${CONTRACT_ADDRESS} \
    --recall-nonce --pem=${USER_PEM} \
    --gas-limit=20000000 \
    --function "unstake" \
    --arguments 14 \
    --send --wait-result \
    --proxy=${PROXY} --chain=${CHAIN_ID} || return
}
claim_rewards() {
    mxpy contract call ${CONTRACT_ADDRESS} \
    --recall-nonce --pem=${USER_PEM} \
    --gas-limit=100000000 \
    --function "claim_rewards" \
    --send --wait-result \
    --proxy=${PROXY} --chain=${CHAIN_ID} || return
}