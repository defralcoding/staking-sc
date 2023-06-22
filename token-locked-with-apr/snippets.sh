SIGNER="--pem=~/wallets/development.pem"

PROXY="https://devnet-gateway.multiversx.com"
CHAIN_ID="D"

CONTRACT_ADDRESS="erd1qqqqqqqqqqqqqpgqxgcasq6e6lgnugenj6lmdwudp9ymgwcs4jwsu9hn6c"
USER_ADDRESS="erd19hcnc2djsjay3prvhuzr0phveducv93khj435pqjza73tcyu4jwsuqywdh"
STAKING_TOKEN="XCUMB-da0e35"
REWARD_TOKEN="XCUMB-da0e35"

deploy() {
    mxpy contract deploy --bytecode="output/staking.wasm" \
    --recall-nonce ${SIGNER} \
    --gas-limit=100000000 \
    --send --wait-result \
    --proxy=${PROXY} --chain=${CHAIN_ID} || return
}

build() {
    mxpy contract build
}

upgrade() {
    mxpy contract upgrade ${CONTRACT_ADDRESS} --project=${PROJECT} \
    --recall-nonce ${SIGNER} \
    --gas-limit=120000000 \
    --send --wait-result \
    --proxy=${PROXY} --chain=${CHAIN_ID} || return
}

call() {
    mxpy contract call ${CONTRACT_ADDRESS} \
    --recall-nonce ${SIGNER} \
    --gas-limit=20000000 \
    --function "unstake_multiple" \
    --arguments 6 \
    --send --wait-result \
    --proxy=${PROXY} --chain=${CHAIN_ID} || return
}

getgenerico() {
    echo User Staking Positions
    erdpy contract query ${CONTRACT_ADDRESS} \
    --function "getUserStaking" --arguments ${USER_ADDRESS} \
    --proxy=${PROXY} || return

    echo User Rewards
    erdpy contract query ${CONTRACT_ADDRESS} \
    --function "calculateRewardsForUser" --arguments ${USER_ADDRESS} \
    --proxy=${PROXY} || return
}


set_admin() {
    mxpy contract call ${CONTRACT_ADDRESS} \
    --recall-nonce ${SIGNER} \
    --gas-limit=20000000 \
    --function "set_admin" \
    --arguments erd1e9nnlhee5rvn9k8y3ysmtsx490h8nw9jels5vjdlezl72yrtg3yq6vvj3r \
    --send --wait-result \
    --proxy=${PROXY} --chain=${CHAIN_ID} || return
}
set_reward_token() {
    mxpy contract call ${CONTRACT_ADDRESS} \
    --recall-nonce ${SIGNER} \
    --gas-limit=20000000 \
    --function "set_reward_token" \
    --arguments str:${REWARD_TOKEN} \
    --send --wait-result \
    --proxy=${PROXY} --chain=${CHAIN_ID} || return
}
set_staking_token() {
    mxpy contract call ${CONTRACT_ADDRESS} \
    --recall-nonce ${SIGNER} \
    --gas-limit=20000000 \
    --function "set_staking_token" \
    --arguments str:${STAKING_TOKEN} \
    --send --wait-result \
    --proxy=${PROXY} --chain=${CHAIN_ID} || return
}
set_apr() {
    mxpy contract call ${CONTRACT_ADDRESS} \
    --recall-nonce ${SIGNER} \
    --gas-limit=20000000 \
    --function "set_apr" \
    --arguments 180 \
    --send --wait-result \
    --proxy=${PROXY} --chain=${CHAIN_ID} || return
}
set_lock_days() {
    mxpy contract call ${CONTRACT_ADDRESS} \
    --recall-nonce ${SIGNER} \
    --gas-limit=20000000 \
    --function "set_lock_days" \
    --arguments 365 \
    --send --wait-result \
    --proxy=${PROXY} --chain=${CHAIN_ID} || return
}
deposit_rewards() {
    mxpy contract call ${CONTRACT_ADDRESS} \
    --recall-nonce ${SIGNER} \
    --gas-limit=20000000 \
    --function "ESDTTransfer" \
    --arguments str:${REWARD_TOKEN} 10000000000000000000000 str:deposit_rewards \
    --send --wait-result \
    --proxy=${PROXY} --chain=${CHAIN_ID} || return
}
stake() {
    mxpy contract call ${CONTRACT_ADDRESS} \
    --recall-nonce ${SIGNER} \
    --gas-limit=20000000 \
    --function "ESDTTransfer" \
    --arguments str:${STAKING_TOKEN} 10000000000000000000000 str:stake \
    --send --wait-result \
    --proxy=${PROXY} --chain=${CHAIN_ID} || return
}
unstake() {
    mxpy contract call ${CONTRACT_ADDRESS} \
    --recall-nonce ${SIGNER} \
    --gas-limit=20000000 \
    --function "unstake" \
    --arguments 1000000000000000000 \
    --send --wait-result \
    --proxy=${PROXY} --chain=${CHAIN_ID} || return
}
claim_rewards() {
    mxpy contract call ${CONTRACT_ADDRESS} \
    --recall-nonce ${SIGNER} \
    --gas-limit=10000000 \
    --function "claim_rewards" \
    --send --wait-result \
    --proxy=${PROXY} --chain=${CHAIN_ID} || return
}