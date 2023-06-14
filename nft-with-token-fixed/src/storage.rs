multiversx_sc::imports!();

use crate::StakingPosition;

#[multiversx_sc::module]
pub trait Storage {
    fn require_admin_or_owner(&self) {
        let admin = self.admin().get();
        let owner = self.blockchain().get_owner_address();
        let caller = self.blockchain().get_caller();
        require!(
            caller == admin || caller == owner,
            "Caller must be an admin",
        );
    }

    fn require_settings(&self) {
        let staking_token_empty = self.staking_token().is_empty();
        let tokens_per_day_empty = self.tokens_per_day().is_empty();
        let reward_token_empty = self.reward_token().is_empty();
        require!(
            !staking_token_empty && !tokens_per_day_empty && !reward_token_empty,
            "The contract is not initialized",
        );
    }

    #[view(getAdmin)]
    #[storage_mapper("admin")]
    fn admin(&self) -> SingleValueMapper<ManagedAddress>;

    #[view(getStakingToken)]
    #[storage_mapper("staking_token")]
    fn staking_token(&self) -> NonFungibleTokenMapper;

    #[view(getRewardToken)]
    #[storage_mapper("reward_token")]
    fn reward_token(&self) -> FungibleTokenMapper;

    #[view(getTokensPerDay)]
    #[storage_mapper("tokens_per_day")]
    fn tokens_per_day(&self) -> SingleValueMapper<BigUint>;

    #[view(getUserStaking)]
    #[storage_mapper("user_staking")]
    fn user_staking(&self, user: &ManagedAddress) -> UnorderedSetMapper<StakingPosition>;

    //
    // SETTERS
    //
    #[endpoint]
    fn set_admin(&self, admin: ManagedAddress) {
        self.require_admin_or_owner();
        self.admin().set(admin);
    }

    #[endpoint]
    fn set_staking_token(&self, staking_token: TokenIdentifier) {
        self.require_admin_or_owner();
        self.staking_token().set_token_id(staking_token);
    }

    #[endpoint]
    fn set_reward_token(&self, reward_token: TokenIdentifier) {
        self.require_admin_or_owner();
        self.reward_token().set_token_id(reward_token);
    }

    #[endpoint]
    fn set_tokens_per_day(&self, tokens_per_day: BigUint) {
        self.require_admin_or_owner();
        self.tokens_per_day().set(tokens_per_day);
    }
}
