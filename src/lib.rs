#![no_std]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();


#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, Clone, Copy, Debug)]
pub enum Status {
    NotStarted,
    Started,
    Ended,
}

const EGLD_IN_WEI: u64 = 1_000_000_000_000_000_000u64;
const ONE_DAY_IN_TIMESTAMPS: u64 = 24 * 3600;

/// Manage ICO of a new ESDT
#[elrond_wasm::contract]
pub trait LandboardIcoSecond {
    // goal in ESDT, min_buy_limit, max_buy_limit are in EGLD
    #[init]
    fn init(&self, token_id: TokenIdentifier, token_price: BigUint, start_time: u64, end_time: u64, goal: BigUint, min_buy_limit: BigUint, max_buy_limit: BigUint) {
        require!(
            token_id.is_valid_esdt_identifier(),
            "invalid token_id"
        );
        
        self.token_id().set(&token_id);
        self.token_price().set(&token_price);
        self.start_time().set(&start_time);
        self.end_time().set(&end_time);
        self.goal().set(&goal);
        self.min_buy_limit().set(&min_buy_limit);
        self.max_buy_limit().set(&max_buy_limit);
    }

    /// endpoint - only owner

    // set config

    #[only_owner]
    #[endpoint(updateTokenId)]
    fn update_token_id(&self, token_id: TokenIdentifier) {
        require!(
            token_id.is_valid_esdt_identifier(),
            "invalid token_id"
        );
        self.token_id().set(&token_id);
    }

    #[only_owner]
    #[endpoint(updateTokenPrice)]
    fn update_token_price(&self, token_price: BigUint) {
        self.token_price().set(&token_price);
    }

    #[only_owner]
    #[endpoint(addWhitelist)]
    fn add_whilelist(&self, #[var_args] addresses: MultiValueEncoded<ManagedAddress>) {
        self.whilelist().extend(addresses);
    }

    #[only_owner]
    #[endpoint(removeWhitelist)]
    fn remove_whilelist(&self, #[var_args] addresses: MultiValueEncoded<ManagedAddress>) {
        self.whilelist().remove_all(addresses);
    }

    #[only_owner]
    #[endpoint(clearWhitelist)]
    fn clear_whilelist(&self) {
        self.whilelist().clear();
    }

    #[only_owner]
    #[endpoint(updateStartTime)]
    fn update_start_time(&self, start_time: u64) {
        self.start_time().set(&start_time);
    }

    #[only_owner]
    #[endpoint(updateEndTime)]
    fn update_end_time(&self, end_time: u64) {
        self.end_time().set(&end_time);
    }

    #[only_owner]
    #[endpoint(updateGoal)]
    fn update_goal(&self, goal: BigUint) {
        self.goal().set(&goal);
    }

    #[only_owner]
    #[endpoint(updateMinBuyLimit)]
    fn update_min_buy_limit(&self, min_buy_limit: BigUint) {
        self.min_buy_limit().set(&min_buy_limit);
    }

    #[only_owner]
    #[endpoint(updateMaxBuyLimit)]
    fn update_max_buy_limit(&self, max_buy_limit: BigUint) {
        self.max_buy_limit().set(&max_buy_limit);
    }

    //

    #[only_owner]
    #[endpoint(withdraw)]
    fn withdraw(&self,
        #[var_args] opt_token_id: OptionalValue<TokenIdentifier>,
        #[var_args] opt_token_amount: OptionalValue<BigUint>) {
        // if token_id is not given, set it to eGLD
        let token_id = match opt_token_id {
            OptionalValue::Some(v) => v,
            OptionalValue::None => TokenIdentifier::egld()
        };
        // if token_amount is not given, set it to balance of SC - max value to withdraw
        let token_amount = match opt_token_amount {
            OptionalValue::Some(v) => v,
            OptionalValue::None => self.blockchain().get_sc_balance(&token_id, 0)
        };

        self.send().direct(&self.blockchain().get_caller(), &token_id, 0, &token_amount, &[]);
    }

    /// endpoint ///

    #[payable("EGLD")]
    #[endpoint(buy)]
    fn buy(&self, #[payment_amount] payment_amount: BigUint) {
        self.require_activation();

        let caller = self.blockchain().get_caller();

        // only whitelist members can buy tokens on the first day
        if self.blockchain().get_block_timestamp() < self.start_time().get() + ONE_DAY_IN_TIMESTAMPS {
            require!(self.whilelist().contains(&caller), "only whitelist members can buy tokens on the first day of sale");
        }
        
        require!(payment_amount >= self.min_buy_limit().get(), "cannot buy less than min_buy_limit at once");
        require!(payment_amount <= self.max_buy_limit().get(), "cannot buy more than max_buy_limit at once");

        let buy_amount = BigUint::from(EGLD_IN_WEI) * &payment_amount / &self.token_price().get();

        require!(&buy_amount + &self.total_bought_amount_of_egld().get() <= self.goal().get(), "cannot buy more than goal amount");

        require!(buy_amount <= self.blockchain().get_sc_balance(&self.token_id().get(), 0), "not enough tokens in smart contract");

        self.total_bought_amount_of_egld().update(|v| *v += &payment_amount);
        self.total_bought_amount_of_esdt().update(|v| *v += &buy_amount);

        self.send().direct(&caller, &self.token_id().get(), 0, &buy_amount, &[]);
    }

    /// view ///

    // return status of ico and left time from start_time or end_time
    // return goal and total_bought_amount_of_esdt
    #[view(getStatus)]
    fn get_status(&self) -> (Status, u64, BigUint, BigUint) {
        let current_timestamp = self.blockchain().get_block_timestamp();
        
        let (status, target_time) = if self.start_time().get() > current_timestamp {
            (Status::NotStarted, self.start_time().get())
        } else if current_timestamp < self.end_time().get() {
            (Status::Started, self.end_time().get())
        } else {
            (Status::Ended, 0u64)
        };

        (status, target_time, self.goal().get(), self.total_bought_amount_of_esdt().get())
    }

    /// private functions ///
    
    fn require_activation(&self) {
        let current_timestamp = self.blockchain().get_block_timestamp();
        require!(self.start_time().get() <= current_timestamp, "sale is not started");
        require!(current_timestamp < self.end_time().get(), "sale is not started");
    }


    /// storage ///

    // config

    #[view(getTokenId)]
    #[storage_mapper("token_id")]
    fn token_id(&self) -> SingleValueMapper<TokenIdentifier>;

    #[view(getTokenPrice)]
    #[storage_mapper("token_price")]
    fn token_price(&self) -> SingleValueMapper<BigUint>;

    #[view(getWhitelist)]
    #[storage_mapper("whilelist")]
    fn whilelist(&self) -> SetMapper<ManagedAddress>;

    #[view(getStartTime)]
    #[storage_mapper("start_time")]
    fn start_time(&self) -> SingleValueMapper<u64>;

    #[view(getEndTime)]
    #[storage_mapper("end_time")]
    fn end_time(&self) -> SingleValueMapper<u64>;

    #[view(getGoal)]
    #[storage_mapper("goal")]
    fn goal(&self) -> SingleValueMapper<BigUint>;

    #[view(getMinBuyLimit)]
    #[storage_mapper("min_buy_limit")]
    fn min_buy_limit(&self) -> SingleValueMapper<BigUint>;

    #[view(getMaxBuyLimit)]
    #[storage_mapper("max_buy_limit")]
    fn max_buy_limit(&self) -> SingleValueMapper<BigUint>;

    // non-config

    #[view(getTotalBoughtAmountOfEgld)]
    #[storage_mapper("total_bought_amount_of_egld")]
    fn total_bought_amount_of_egld(&self) -> SingleValueMapper<BigUint>;

    #[view(getTotalBoughtAmountOfEsdt)]
    #[storage_mapper("total_bought_amount_of_esdt")]
    fn total_bought_amount_of_esdt(&self) -> SingleValueMapper<BigUint>;
}
