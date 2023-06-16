#![no_std]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[derive(
    TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq, Debug, ManagedVecItem,
)]
pub struct StakingPosition<M: ManagedTypeApi> {
    pub staked_amount: BigUint<M>,
    pub staked_epoch: u64,
    pub last_claimed_timestamp: u64,
}

pub mod storage;

#[multiversx_sc::contract]
pub trait StakingContract: storage::Storage {
    #[init]
    fn init(&self) {}

    #[payable("*")]
    #[endpoint]
    fn deposit_rewards(
        &self,
        #[payment_token] token: EgldOrEsdtTokenIdentifier,
        #[payment_amount] amount: BigUint,
    ) {
        self.require_admin_or_owner();
        self.require_settings();

        let reward_token = self.reward_token().get_token_id();
        require!(
            token.unwrap_esdt() == reward_token,
            "The reward token must be: {}",
            reward_token
        );

        let prev_rewards_amount = self.rewards_amount().get();
        let new_rewards_amount = prev_rewards_amount + amount;
        self.rewards_amount().set(new_rewards_amount);
    }

    #[payable("*")]
    #[endpoint]
    fn stake(
        &self,
        #[payment_token] token: EgldOrEsdtTokenIdentifier,
        #[payment_amount] amount: BigUint,
    ) {
        self.require_settings();

        let staking_token = self.staking_token().get_token_id();
        require!(
            &token.unwrap_esdt() == &staking_token,
            "The staking token must be: {}",
            staking_token
        );

        let caller = self.blockchain().get_caller();
        self._stake(amount, &caller);
    }

    fn _stake(&self, amount: BigUint, user: &ManagedAddress) {
        let user_staking_mapper = self.user_staking(&user);
        let new_user = self.staked_addresses().insert(user.clone());

        let mut staking_position = if !new_user {
            user_staking_mapper.get()
        } else {
            StakingPosition {
                staked_amount: BigUint::zero(),
                staked_epoch: self.blockchain().get_block_epoch(),
                last_claimed_timestamp: self.blockchain().get_block_timestamp(),
            }
        };

        if !new_user {
            self._claim_rewards_for_user(&user);
        }

        staking_position.staked_amount += amount;
        user_staking_mapper.set(staking_position);
    }

    #[view(calculateRewardsForUser)]
    fn calculate_rewards_for_user(&self, address: ManagedAddress) -> BigUint {
        self._calculate_rewards_for_user(&address)
    }

    fn _calculate_rewards_for_user(&self, address: &ManagedAddress) -> BigUint {
        let staking_position = self.user_staking(&address).get();
        let apr = self.apr().get();
        let current_timestamp = self.blockchain().get_block_timestamp();

        if current_timestamp <= staking_position.last_claimed_timestamp {
            //TODO check if needed
            return BigUint::zero();
        }

        let timestamp_diff = current_timestamp - staking_position.last_claimed_timestamp;
        staking_position.staked_amount * apr / BigUint::from(100u32) * timestamp_diff
            / BigUint::from(365u32 * 24u32 * 60u32 * 60u32) //TODO check if this is correct
    }

    #[endpoint]
    fn claim_rewards(&self) {
        let caller = self.blockchain().get_caller();
        self._claim_rewards_for_user(&caller);
    }

    fn _claim_rewards_for_user(&self, user: &ManagedAddress) {
        let rewards = self._calculate_rewards_for_user(&user);
        let user_staking_mapper = self.user_staking(&user);

        require!(rewards > BigUint::zero(), "No rewards to claim");

        self._send_reward_token(rewards, &user);

        let mut staking_position = user_staking_mapper.get();
        staking_position.last_claimed_timestamp = self.blockchain().get_block_timestamp();
        user_staking_mapper.set(staking_position);
    }

    #[endpoint]
    fn unstake(&self, amount: BigUint) {
        let caller = self.blockchain().get_caller();

        require!(
            self.staked_addresses().contains(&caller),
            "User has not staked"
        );

        let mut staking_position = self.user_staking(&caller).get();

        require!(
            staking_position.staked_amount >= amount,
            "User has not staked enough"
        );

        self._unstake(amount, &caller, &mut staking_position);
    }

    fn _unstake(
        &self,
        amount: BigUint,
        user: &ManagedAddress,
        staking_position: &mut StakingPosition<Self::Api>,
    ) {
        let rewards = self._calculate_rewards_for_user(&user);

        self._send_staking_token(amount.clone(), &user);
        self._send_reward_token(rewards, &user);

        staking_position.staked_amount -= amount;
        if staking_position.staked_amount == BigUint::zero() {
            self.staked_addresses().swap_remove(&user);
            self.user_staking(&user).clear();
        } else {
            self.user_staking(&user).set(staking_position);
        }
    }

    fn _send_reward_token(&self, amount: BigUint, to: &ManagedAddress) {
        if amount == BigUint::zero() {
            return;
        }
        let reward_token = self.reward_token();
        let rewards_amount = self.rewards_amount().get();

        require!(rewards_amount >= amount, "Not enough deposited rewards");

        self.send()
            .direct_esdt(&to, &reward_token.get_token_id(), 0, &amount);

        self.rewards_amount().set(rewards_amount - amount);
    }

    fn _send_staking_token(&self, amount: BigUint, to: &ManagedAddress) {
        let staking_token = self.staking_token();
        self.send()
            .direct_esdt(&to, &staking_token.get_token_id(), 0, &amount);
    }
}
