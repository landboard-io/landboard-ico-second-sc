##### - configuration - #####
PROXY=https://devnet-gateway.elrond.com
CHAIN_ID="D"

WALLET="./wallets/wallet.pem"

TOKEN_ID="LAND-2685e5"
LOCKED_TOKEN_ID="LKLAND-bad786"
TOKEN_ID_HEX="0x$(echo -n ${TOKEN_ID} | xxd -p -u | tr -d '\n')"
LOCKED_TOKEN_ID_HEX="0x$(echo -n ${LOCKED_TOKEN_ID} | xxd -p -u | tr -d '\n')"

TOKEN_PRICE=20000000000000          # 0.00002 EGLD
MIN_BUY_LIMIT=200000000000000000    # 0.2 EGLD
MAX_BUY_LIMIT=1000000000000000000   # 1 EGLD
GOAL=500000000000000000000          # 500 EGLD
START_TIME=1646498778
END_TIME=1654135000

CALLER_ADDRESS_HEX="0x418c125e5a25d88f2ee0e4daaee26c1b4d878aeeb8a178ef23dcafb87b36d19d"

######
ADDRESS=$(erdpy data load --key=address-devnet)
TRANSACTION=$(erdpy data load --key=deployTransaction-devnet)
######

deploy() {
    erdpy --verbose contract deploy \
    --project=${PROJECT} \
    --recall-nonce \
    --pem=${WALLET} \
    --gas-limit=50000000 \
    --arguments ${TOKEN_ID_HEX} ${LOCKED_TOKEN_ID_HEX} ${TOKEN_PRICE} ${START_TIME} ${END_TIME} ${GOAL} ${MIN_BUY_LIMIT} ${MAX_BUY_LIMIT} \
    --send \
    --outfile="deploy-devnet.interaction.json" \
    --proxy=${PROXY} \
    --metadata-payable \
    --metadata-payable-by-sc \
    --chain=${CHAIN_ID} || return

    TRANSACTION=$(erdpy data parse --file="deploy-devnet.interaction.json" --expression="data['emitted_tx']['hash']")
    ADDRESS=$(erdpy data parse --file="deploy-devnet.interaction.json" --expression="data['emitted_tx']['address']")

    erdpy data store --key=address-devnet --value=${ADDRESS}
    erdpy data store --key=deployTransaction-devnet --value=${TRANSACTION}
}

buy() {
    erdpy --verbose contract call ${ADDRESS} \
    --recall-nonce --pem=${WALLET} \
    --gas-limit=6000000 \
    --value=1000000000000000000 \
    --function="buy" \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

withdraw() {
    erdpy --verbose contract call ${ADDRESS} \
    --recall-nonce --pem=${WALLET} \
    --gas-limit=6000000 \
    --function="withdraw" \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

addWhitelist() {
    erdpy --verbose contract call ${ADDRESS} \
    --recall-nonce --pem=${WALLET} \
    --gas-limit=10000000 \
    --function="addWhitelist" \
    --arguments 0x418c125e5a25d88f2ee0e4daaee26c1b4d878aeeb8a178ef23dcafb87b36d19d 0x687bf6e9c03fd4c2906da834906d546518b0586937c3910fee8d1b5a1dad0e01 0x3f25e71d420356b7d425757b58b4a27383f04f554a8a5831a6af346d56b08a4f 0xd743983a9576609aad104dc1228d49b5884102c880fab7681a658edb3b292002 0xb77a7aac2cc46d42dfc134b915af1eabff49fffc8f10b711e6468a591141331e \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

clearWhitelist() {
    erdpy --verbose contract call ${ADDRESS} \
    --recall-nonce --pem=${WALLET} \
    --gas-limit=8000000 \
    --function="clearWhitelist" \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

# config

getTokenId() {
    erdpy --verbose contract query ${ADDRESS} --function="getTokenId" --proxy=${PROXY}
}

getTokenPrice() {
    erdpy --verbose contract query ${ADDRESS} --function="getTokenPrice" --proxy=${PROXY}
}

getWhitelist() {
    erdpy --verbose contract query ${ADDRESS} --function="getWhitelist" --proxy=${PROXY}
}

getStartTime() {
    erdpy --verbose contract query ${ADDRESS} --function="getStartTime" --proxy=${PROXY}
}

getEndTime() {
    erdpy --verbose contract query ${ADDRESS} --function="getEndTime" --proxy=${PROXY}
}

getGoalInEgld() {
    erdpy --verbose contract query ${ADDRESS} --function="getGoalInEgld" --proxy=${PROXY}
}

getMinBuyLimit() {
    erdpy --verbose contract query ${ADDRESS} --function="getMinBuyLimit" --proxy=${PROXY}
}

getMaxBuyLimit() {
    erdpy --verbose contract query ${ADDRESS} --function="getMaxBuyLimit" --proxy=${PROXY}
}

# state

getTotalBoughtAmountOfEgld() {
    erdpy --verbose contract query ${ADDRESS} --function="getTotalBoughtAmountOfEgld" --proxy=${PROXY}
}

getTotalBoughtAmountOfEsdt() {
    erdpy --verbose contract query ${ADDRESS} --function="getTotalBoughtAmountOfEsdt" --proxy=${PROXY}
}

getStatus() {
    erdpy --verbose contract query ${ADDRESS} --function="getStatus" --proxy=${PROXY}
}