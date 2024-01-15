use ink::{
    prelude::{string::String, vec::Vec},
    primitives::AccountId,
};

#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum PSP22Error {
    /// Custom error type for implementation-based errors.
    Custom(String),
    /// Returned when an account does not have enough tokens to complete the operation.
    InsufficientBalance,
    /// Returned if there is not enough allowance to complete the operation.
    InsufficientAllowance,
    /// Returned if recipient's address is zero [deprecated].
    ZeroRecipientAddress,
    /// Returned if sender's address is zero [deprecated].
    ZeroSenderAddress,
    /// Returned if a safe transfer check failed [deprecated].
    SafeTransferCheckFailed(String),
}

#[ink::trait_definition]
pub trait PSP22 {
    /// Returns the total token supply.
    #[ink(message)]
    fn total_supply(&self) -> u128;

    /// Returns the account balance for the specified `owner`.
    ///
    /// Returns `0` if the account is non-existent.
    #[ink(message)]
    fn balance_of(&self, owner: AccountId) -> u128;

    /// Returns the amount which `spender` is still allowed to withdraw from `owner`.
    ///
    /// Returns `0` if no allowance has been set.
    #[ink(message)]
    fn allowance(&self, owner: AccountId, spender: AccountId) -> u128;

    /// Transfers `value` amount of tokens from the caller's account to account `to`
    /// with additional `data` in unspecified format.
    ///
    /// # Events
    ///
    /// On success a `Transfer` event is emitted.
    ///
    /// No-op if the caller and `to` is the same address or `value` is zero, returns success
    /// and no events are emitted.
    ///
    /// # Errors
    ///
    /// Reverts with `InsufficientBalance` if the `value` exceeds the caller's balance.
    #[ink(message)]
    fn transfer(&mut self, to: AccountId, value: u128, data: Vec<u8>) -> Result<(), PSP22Error>;

    /// Transfers `value` tokens on the behalf of `from` to the account `to`
    /// with additional `data` in unspecified format.
    ///
    /// If `from` and the caller are different addresses, the caller must be allowed
    /// by `from` to spend at least `value` tokens.
    ///
    /// # Events
    ///
    /// On success a `Transfer` event is emitted.
    ///
    /// No-op if `from` and `to` is the same address or `value` is zero, returns success
    /// and no events are emitted.
    ///
    /// If `from` and the caller are different addresses, a successful transfer results
    /// in decreased allowance by `from` to the caller and an `Approval` event with
    /// the new allowance amount is emitted.
    ///
    /// # Errors
    ///
    /// Reverts with `InsufficientBalance` if the `value` exceeds the balance of the account `from`.
    ///
    /// Reverts with `InsufficientAllowance` if `from` and the caller are different addresses and
    /// the `value` exceeds the allowance granted by `from` to the caller.
    ///
    /// If conditions for both `InsufficientBalance` and `InsufficientAllowance` errors are met,
    /// reverts with `InsufficientAllowance`.
    #[ink(message)]
    fn transfer_from(
        &mut self,
        from: AccountId,
        to: AccountId,
        value: u128,
        data: Vec<u8>,
    ) -> Result<(), PSP22Error>;

    /// Allows `spender` to withdraw from the caller's account multiple times, up to
    /// the total amount of `value`.
    ///
    /// Successive calls of this method overwrite previous values.
    ///
    /// # Events
    ///
    /// An `Approval` event is emitted.
    ///
    /// No-op if the caller and `spender` is the same address, returns success and no events are emitted.
    #[ink(message)]
    fn approve(&mut self, spender: AccountId, value: u128) -> Result<(), PSP22Error>;

    /// Increases by `delta-value` the allowance granted to `spender` by the caller.
    ///
    /// # Events
    ///
    /// An `Approval` event with the new allowance amount is emitted.
    ///
    /// No-op if the caller and `spender` is the same address or `delta-value` is zero, returns success
    /// and no events are emitted.
    #[ink(message)]
    fn increase_allowance(
        &mut self,
        spender: AccountId,
        delta_value: u128,
    ) -> Result<(), PSP22Error>;

    /// Decreases by `delta-value` the allowance granted to `spender` by the caller.
    ///
    /// # Events
    ///
    /// An `Approval` event with the new allowance amount is emitted.
    ///
    /// No-op if the caller and `spender` is the same address or `delta-value` is zero, returns success
    /// and no events are emitted.
    ///
    /// # Errors
    ///
    /// Reverts with `InsufficientAllowance` if `spender` and the caller are different addresses and
    /// the `delta-value` exceeds the allowance granted by the caller to `spender`.
    #[ink(message)]
    fn decrease_allowance(
        &mut self,
        spender: AccountId,
        delta_value: u128,
    ) -> Result<(), PSP22Error>;
}
