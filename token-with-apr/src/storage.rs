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
        let apr_empty = self.apr().is_empty();
        let reward_token_empty = self.reward_token().is_empty();
        require!(
            !staking_token_empty && !apr_empty && !reward_token_empty,
            "The contract is not initialized",
        );
    }

    #[view(getAdmin)]
    #[storage_mapper("admin")]
    fn admin(&self) -> SingleValueMapper<ManagedAddress>;

    #[view(getStakingToken)]
    #[storage_mapper("staking_token")]
    fn staking_token(&self) -> FungibleTokenMapper;

    #[view(getRewardToken)]
    #[storage_mapper("reward_token")]
    fn reward_token(&self) -> FungibleTokenMapper;

    #[view(getRewardsAmount)]
    #[storage_mapper("rewards_amount")]
    fn rewards_amount(&self) -> SingleValueMapper<BigUint>;

    #[view(getApr)]
    #[storage_mapper("apr")]
    fn apr(&self) -> SingleValueMapper<u64>;

    #[view(getUserStaking)]
    #[storage_mapper("user_staking")]
    fn user_staking(&self, user: &ManagedAddress) -> SingleValueMapper<StakingPosition<Self::Api>>;

    #[view(getStakedAddresses)]
    #[storage_mapper("stakedAddresses")]
    fn staked_addresses(&self) -> UnorderedSetMapper<ManagedAddress>;

    //
    // SETTERS
    //
    #[only_owner]
    #[endpoint]
    fn set_admin(&self, admin: ManagedAddress) {
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
    fn set_apr(&self, apr: u64) {
        self.require_admin_or_owner();
        self.apr().set(apr);
    }
}
