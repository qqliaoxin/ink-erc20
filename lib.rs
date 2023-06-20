#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod erc20 {
    use ink::storage::Mapping;

    /// A simple ERC-20 contract.
    #[ink(storage)]
    #[derive(Default)]
    pub struct Erc20 {
        /// token 发行总量
        total_supply: Balance,
        /// 用户余额 存储 Mapping 
        balances: Mapping<AccountId, Balance>,
        /// Mapping of the token amount which an account is allowed to withdraw
        /// from another account.
        allowances: Mapping<(AccountId, AccountId), Balance>,
    }

    /// Event emitted when a token transfer occurs.
    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: Option<AccountId>,
        value: Balance,
    }

    /// Event emitted when an approval occurs that `spender` is allowed to withdraw
    /// up to the amount of `value` tokens from `owner`.
    #[ink(event)]
    pub struct Approval {
        #[ink(topic)]
        owner: AccountId,
        #[ink(topic)]
        spender: AccountId,
        value: Balance,
    }

    /// The ERC-20 error types.
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        /// Returned if not enough balance to fulfill a request is available.
        InsufficientBalance,
        /// Returned if not enough allowance to fulfill a request is available.
        InsufficientAllowance,
    }

    /// The ERC-20 result type.
    pub type Result<T> = core::result::Result<T, Error>;

    impl Erc20 {
        // 合约初始化
        #[ink(constructor)]
        pub fn new(total_supply: Balance) -> Self {
            // 初始化 Mapping 实例
            let mut balances = Mapping::default();
            // 当前调用者
            let caller = Self::env().caller();
            balances.insert(caller, &total_supply);
            // total_supply 总量给于 当前调用者
            Self::env().emit_event(Transfer {
                from: None,
                to: Some(caller),
                value: total_supply,
            });
            // 反回合约初始化结构对象
            Self {
                total_supply,
                balances,
                allowances: Default::default(),
            }
        }

        /// Returns the total token supply.
        #[ink(message)]
        pub fn total_supply(&self) -> Balance {
            self.total_supply
        }

        /// Returns the account balance for the specified `owner`.
        ///
        /// 返回用户余额
        #[ink(message)]
        pub fn balance_of(&self, owner: AccountId) -> Balance {
            self.balance_of_impl(&owner)
        }

        /// Returns the account balance for the specified `owner`.
        ///
        /// Returns `0` if the account is non-existent.
        ///
        /// # Note
        /// 内部查询余额方法
        #[inline]
        fn balance_of_impl(&self, owner: &AccountId) -> Balance {
            self.balances.get(owner).unwrap_or_default()
        }

        /// Returns the amount which `spender` is still allowed to withdraw from `owner`.
        ///
        /// Returns `0` if no allowance has been set.
        #[ink(message)]
        pub fn allowance(&self, owner: AccountId, spender: AccountId) -> Balance {
            self.allowance_impl(&owner, &spender)
        }

        /// Returns the amount which `spender` is still allowed to withdraw from `owner`.
        ///
        /// Returns `0` if no allowance has been set.
        ///
        /// # Note
        ///
        /// Prefer to call this method over `allowance` since this
        /// works using references which are more efficient in Wasm.
        #[inline]
        fn allowance_impl(&self, owner: &AccountId, spender: &AccountId) -> Balance {
            self.allowances.get((owner, spender)).unwrap_or_default()
        }

        /// Transfers `value` amount of tokens from the caller's account to account `to`.
        ///
        /// On success a `Transfer` event is emitted.
        ///
        /// # Errors
        ///
        /// 代币转账 to other
        #[ink(message)]
        pub fn transfer(&mut self, to: AccountId, value: Balance) -> Result<()> {
            let from = self.env().caller();
            self.transfer_from_to(&from, &to, value)
        }

        /// Allows `spender` to withdraw from the caller's account multiple times, up to
        /// the `value` amount.
        ///
        /// If this function is called again it overwrites the current allowance with
        /// `value`.
        ///
        /// 授予转账
        #[ink(message)]
        pub fn approve(&mut self, spender: AccountId, value: Balance) -> Result<()> {
            let owner = self.env().caller();
            self.allowances.insert((&owner, &spender), &value);
            self.env().emit_event(Approval {
                owner,
                spender,
                value,
            });
            Ok(())
        }

        /// Transfers `value` tokens on the behalf of `from` to the account `to`.
        ///
        /// This can be used to allow a contract to transfer tokens on ones behalf and/or
        /// to charge fees in sub-currencies, for example.
        ///
        /// On success a `Transfer` event is emitted.
        ///
        /// # Errors
        ///
        /// Returns `InsufficientAllowance` error if there are not enough tokens allowed
        /// for the caller to withdraw from `from`.
        ///
        /// 授予转账，提币出来
        #[ink(message)]
        pub fn transfer_from(
            &mut self,
            from: AccountId,
            to: AccountId,
            value: Balance,
        ) -> Result<()> {
            let caller = self.env().caller();
            // 检查是否授予转账
            let allowance = self.allowance_impl(&from, &caller);
            if allowance < value {
                return Err(Error::InsufficientAllowance)
            }
            // 转账代币
            self.transfer_from_to(&from, &to, value)?;
            self.allowances
                .insert((&from, &caller), &(allowance - value));
            Ok(())
        }

        /// Transfers `value` amount of tokens from the caller's account to account `to`.
        ///
        /// On success a `Transfer` event is emitted.
        ///
        /// # Errors
        ///
        /// Returns `InsufficientBalance` error if there are not enough tokens on
        /// the caller's account balance.
        fn transfer_from_to(
            &mut self,
            from: &AccountId,
            to: &AccountId,
            value: Balance,
        ) -> Result<()> {
            let from_balance = self.balance_of_impl(from);
            if from_balance < value {
                return Err(Error::InsufficientBalance)
            }

            self.balances.insert(from, &(from_balance - value));
            let to_balance = self.balance_of_impl(to);
            self.balances.insert(to, &(to_balance + value));
            self.env().emit_event(Transfer {
                from: Some(*from),
                to: Some(*to),
                value,
            });
            Ok(())
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        use ink::primitives::{
            Clear,
            Hash,
        };

        type Event = <Erc20 as ::ink::reflect::ContractEventBase>::Type;

        /// The default constructor does its job.
        #[ink::test]
        fn constructor_works() {
            // Constructor works.
            let _erc20 = Erc20::new(10000);
            let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
            
            assert_eq!(_erc20.total_supply(), 10000);
            assert_eq!(_erc20.balance_of(accounts.alice), 10000);

            let eniteed_events = ink::env::test::recorded_events().collect::<Vec<_>>();
            let event = &eniteed_events[0];

            let decoded = <Event as scale::Decode>::decode(&mut &event.data[..]).expect("decode error");

            match decoded {
                Event::Transfer(Transfer { from, to, value }) => {
                    assert!(from.is_none(),"mint from error");
                    assert_eq!(to, Some(accounts.alice),"mint to error");
                    assert_eq!(value, 10000,"mint value error");
                },
                _ => panic!("match invalid event")
            }
        }
        #[ink::test]
        fn transfer_should_work() {
            let mut _erc20 = Erc20::new(10000);
            let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
            let res = _erc20.transfer(accounts.bob,12);

            assert!(res.is_ok());
            assert_eq!(_erc20.balance_of(accounts.alice),10000-12);
            assert_eq!(_erc20.balance_of(accounts.bob),12);
        }   
        #[ink::test]
        fn invalid_transfer_should_work() {
            let mut _erc20 = Erc20::new(10000);
            let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
            let res = _erc20.transfer(accounts.bob,12);

            assert!(res.is_err());
            assert_eq!(res,Err(Error::InsufficientBalance));
        }     
    }

    // #[cfg(feature = "e2e-tests")]
    // mod e2e_tests {
    //     use super::*;
    //     use ink_e2e::build_message;

    //     type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

    //     #[ink_e2e::test]
    //     async fn e2e_transfer(mut client: ink_e2e::Client<C,E>) -> E2EResult<()> {
    //         let total_supply =123;
    //         let constructor = Erc20Ref::new(total_supply);
    //         let contract_acc_id = client.instantiate("erc20",&ink_e2e::alice(),constructor,0,None).await.expect("Failed to instantiate").account_id;
    //         let alice_acc = ink_e2e::account_id(ink_e2e::AccountKeyring::Alice);
    //         let bob_acc = ink_e2e::account_id(ink_e2e::AccountKeyring::Bob);

    //         let transfer_msg = build_message::<E2ERef>().call(|erc20| erc20.transfer(bob_acc,2));

    //         let res = client.call(&ink_e2e::alice(),transfer_msg,0,None).await;

    //         assert!(res.is_ok());
    //         let balance_of_msg = build_message::<E2ERef>(contract_acc_id.clone()).call(|erc20| erc20.balance_of(alice_acc));

    //         let balance_of_alice = client.call_dry_run(&ink_e2e::alice(),&balance_of_msg,0,None).await;

    //         assert!(balance_of_alice.return_value(),121);

    //         Ok(())
    //     }
    // }
}