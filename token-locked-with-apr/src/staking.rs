#![no_std]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[derive(
    TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq, Debug, ManagedVecItem,
)]
pub struct StakingPosition<M: ManagedTypeApi> {
    pub id: u64,
    pub staked_amount: BigUint<M>,
    pub staked_epoch: u64,
    pub unlock_timestamp: u64,
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
    ) -> u64 {
        self.require_settings();

        let staking_token = self.staking_token().get_token_id();
        require!(
            &token.unwrap_esdt() == &staking_token,
            "The staking token must be: {}",
            staking_token
        );

        let caller = self.blockchain().get_caller();
        self._stake(amount, &caller)
    }

    //TODO check
    //TODO p2: if stake in the same epoch, add to the same position
    fn _stake(&self, amount: BigUint, user: &ManagedAddress) -> u64 {
        let new_id = self.last_id().get() + 1;
        let lock_days = self.lock_days().get();

        let unlock_timestamp = self.blockchain().get_block_timestamp() + lock_days * 24 * 60 * 60;

        let new_position = StakingPosition {
            id: new_id,
            staked_amount: amount,
            staked_epoch: self.blockchain().get_block_epoch(),
            unlock_timestamp,
            last_claimed_timestamp: self.blockchain().get_block_timestamp(),
        };
        self.user_staking(user).insert(new_position);
        self.staked_addresses().insert(user.clone());
        self.last_id().set(new_id);
        new_id
    }

    #[view(calculateRewardsForUser)]
    fn calculate_rewards_for_user(&self, address: ManagedAddress) -> BigUint {
        self._calculate_rewards_for_user(&address)
    }

    //TODO check
    fn _calculate_rewards_for_user(&self, address: &ManagedAddress) -> BigUint {
        let mut total_rewards = BigUint::zero();
        for position in self.user_staking(&address).iter() {
            total_rewards += self._calculate_rewards_for_position(&position);
        }
        total_rewards
    }

    fn _calculate_rewards_for_position(
        &self,
        staking_position: &StakingPosition<Self::Api>,
    ) -> BigUint {
        let apr = self.apr().get();
        let current_timestamp = self.blockchain().get_block_timestamp();

        if current_timestamp <= staking_position.last_claimed_timestamp {
            return BigUint::zero();
        }

        let timestamp_diff = current_timestamp - staking_position.last_claimed_timestamp;
        staking_position.staked_amount.clone() * apr / BigUint::from(100u32) * timestamp_diff
            / BigUint::from(365u32 * 24u32 * 60u32 * 60u32)
    }

    #[endpoint]
    fn claim_rewards(&self) {
        let caller = self.blockchain().get_caller();
        self._claim_rewards_for_user(&caller);
    }

    //TODO check
    fn _claim_rewards_for_user(&self, user: &ManagedAddress) {
        let rewards = self._calculate_rewards_for_user(&user);
        require!(rewards > BigUint::zero(), "No rewards to claim");

        self._send_reward_token(rewards, &user);

        let mut positions_to_re_add: ManagedVec<StakingPosition<Self::Api>> = ManagedVec::new();
        for mut position in self.user_staking(&user).iter() {
            position.last_claimed_timestamp = self.blockchain().get_block_timestamp();
            positions_to_re_add.push(position);
        }

        self.user_staking(&user).clear();
        self.user_staking(&user).extend(&positions_to_re_add);
    }

    //TODO check
    #[endpoint]
    fn unstake(&self, id: u64) {
        let caller = self.blockchain().get_caller();
        let user_staking = self.user_staking(&caller);
        let mut found = false;
        for position in user_staking.iter() {
            if position.id == id {
                self._unstake(&caller, &position);
                found = true;
                break;
            }
        }

        require!(found, "Staking position not found");
    }

    //TODO check
    fn _unstake(&self, user: &ManagedAddress, staking_position: &StakingPosition<Self::Api>) {
        let current_timestamp = self.blockchain().get_block_timestamp();
        require!(
            current_timestamp >= staking_position.unlock_timestamp,
            "The staking position is still locked"
        );

        self._send_staking_token(staking_position.staked_amount.clone(), &user);
        self._claim_rewards_for_user(&user);

        self.user_staking(&user).swap_remove(staking_position);
        if self.user_staking(&user).is_empty() {
            self.staked_addresses().swap_remove(&user);
        }
    }

    fn _send_reward_token(&self, amount: BigUint, to: &ManagedAddress) {
        if amount == BigUint::zero() {
            return;
        }
        let reward_token = self.reward_token();
        let rewards_amount = self.rewards_amount().get();

        require!(rewards_amount > amount, "Not enough deposited rewards");

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
