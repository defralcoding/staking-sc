#![no_std]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[derive(
    TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq, Debug, ManagedVecItem,
)]
pub struct StakingPosition {
    pub nonce: u64,
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
        #[payment_nonce] nonce: u64,
    ) {
        self.require_settings();

        let staking_token = self.staking_token().get_token_id();
        require!(
            &token.unwrap_esdt() == &staking_token,
            "The staking token must be: {}",
            staking_token
        );

        let caller = self.blockchain().get_caller();
        self._stake(nonce, &caller);
    }

    #[payable("*")]
    #[endpoint]
    fn stake_multiple(
        &self,
        #[payment_multi] payments: ManagedRef<'static, ManagedVec<EsdtTokenPayment<Self::Api>>>,
    ) {
        self.require_settings();

        let staking_token = self.staking_token().get_token_id();
        let caller = self.blockchain().get_caller();

        for payment in payments.iter() {
            let token_payment = EgldOrEsdtTokenPayment::from(payment);
            let (token_identifier, nonce, _) = token_payment.into_tuple();

            require!(
                &token_identifier == &staking_token,
                "The staking token must be: {}",
                staking_token
            );

            self._stake(nonce, &caller);
        }
    }

    fn _stake(&self, nonce: u64, user: &ManagedAddress) {
        let new_position = StakingPosition {
            nonce,
            staked_epoch: self.blockchain().get_block_epoch(),
            last_claimed_timestamp: self.blockchain().get_block_timestamp(),
        };
        self.user_staking(user).insert(new_position);
    }

    #[view(calculateRewardsForUser)]
    fn calculate_rewards_for_user(&self, address: ManagedAddress) -> BigUint {
        self._calculate_rewards_for_user(&address)
    }

    fn _calculate_rewards_for_user(&self, address: &ManagedAddress) -> BigUint {
        let mut total_rewards = BigUint::zero();
        for position in self.user_staking(&address).iter() {
            total_rewards += self._calculate_rewards_for_position(&position);
        }
        total_rewards
    }

    fn _calculate_rewards_for_position(&self, position: &StakingPosition) -> BigUint {
        let current_timestamp = self.blockchain().get_block_timestamp();

        if current_timestamp <= position.last_claimed_timestamp {
            return BigUint::zero();
        }

        let tokens_per_day = self.tokens_per_day().get();
        let timestamp_diff = current_timestamp - position.last_claimed_timestamp;
        BigUint::from(timestamp_diff) * tokens_per_day / BigUint::from(24u32 * 60u32 * 60u32)
    }

    #[endpoint]
    fn claim_rewards(&self) {
        let caller = self.blockchain().get_caller();
        self._claim_rewards_for_user(&caller);
    }

    fn _claim_rewards_for_user(&self, user: &ManagedAddress) {
        let rewards = self._calculate_rewards_for_user(&user);
        require!(rewards > BigUint::zero(), "No rewards to claim");

        self._send_reward_token(rewards, &user);

        let mut positions_to_re_add: ManagedVec<StakingPosition> = ManagedVec::new();
        for mut position in self.user_staking(&user).iter() {
            position.last_claimed_timestamp = self.blockchain().get_block_timestamp();
            positions_to_re_add.push(position);
        }

        self.user_staking(&user).clear();
        self.user_staking(&user).extend(&positions_to_re_add);
    }

    #[endpoint]
    fn unstake(&self, nonce: u64) {
        let caller = self.blockchain().get_caller();
        let user_staking = self.user_staking(&caller);
        let mut found = false;
        for position in user_staking.iter() {
            if position.nonce == nonce {
                let rewards = self._unstake_position(&caller, &position);
                self._send_reward_token(rewards, &caller);
                found = true;
                break;
            }
        }

        require!(found, "User has not staked nonce {}", nonce);
    }

    #[endpoint]
    fn unstake_multiple(&self, nonces_to_unstake: MultiValueEncoded<u64>) {
        let caller = self.blockchain().get_caller();
        let user_staking = self.user_staking(&caller);
        let mut rewards_to_send = BigUint::zero();

        for nonce in nonces_to_unstake {
            let mut found = false;
            for position in user_staking.iter() {
                if position.nonce == nonce {
                    rewards_to_send += self._unstake_position(&caller, &position);
                    found = true;
                    break;
                }
            }
            require!(found, "User has not staked nonce {}", nonce);
        }

        self._send_reward_token(rewards_to_send, &caller);
    }

    // returns the rewards to send to the user
    fn _unstake_position(&self, user: &ManagedAddress, position: &StakingPosition) -> BigUint {
        let rewards = self._calculate_rewards_for_position(position);
        self._send_staking_token(position.nonce, &user);
        self.user_staking(&user).swap_remove(position);
        rewards
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

    fn _send_staking_token(&self, nonce: u64, to: &ManagedAddress) {
        let staking_token = self.staking_token();
        self.send().direct_esdt(
            &to,
            &staking_token.get_token_id(),
            nonce,
            &BigUint::from(1u8),
        );
    }
}
