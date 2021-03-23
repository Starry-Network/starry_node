#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure,
};
use frame_system::ensure_signed;
use sp_runtime::{traits::AccountIdConversion, ModuleId};
use sp_std::vec::Vec;

use pallet_nft;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct TokenInfo<AccountId> {
    pub end_idx: u128,
    pub owner: AccountId,
    pub uri: Vec<u8>,
}

// #[derive(Encode, Decode, Copy, Clone, PartialEq, Eq)]
#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq)]
pub enum SubTokenType {
    None,
    NonFungible,
    Fungible,
}

#[derive(Encode, Decode, Clone, Default, Eq, PartialEq)]
pub struct SubTokenInfo<AccountId> {
    pub owner: AccountId,
    pub total_supply: u128,
    pub sub_token_type: Option<SubTokenType>,
}

const PALLET_ID: ModuleId = ModuleId(*b"SubToken");

pub trait Config: frame_system::Config + pallet_nft::Config {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
}

pub type CollectionId = u128;
pub type StartIdx = u128;
pub type SubStartIdx = u128;
pub type MintAmount = u128;
pub type Balance = u128;
pub type SubTokenId = u128;

decl_storage! {
    trait Store for Module<T: Config> as SubModule {
        pub LockedTokens get(fn locked_tokens): map hasher(blake2_128_concat) (CollectionId, StartIdx) => SubTokenInfo<T::AccountId>;

        pub LastTokenId get(fn last_token_id): map hasher(blake2_128_concat) (CollectionId, StartIdx) => SubTokenId;

        pub SubTokens get(fn sub_tokens):
            double_map hasher(blake2_128_concat) (CollectionId, StartIdx), hasher(blake2_128_concat) SubStartIdx => TokenInfo<T::AccountId>;

        pub AddressBalances get(fn address_balances):
            double_map hasher(blake2_128_concat) (CollectionId, StartIdx), hasher(blake2_128_concat) T::AccountId => Balance;

        pub BurnedSubTokens get(fn burned_sub_tokens): map hasher(blake2_128_concat) (CollectionId,StartIdx) => u128;
    }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Config>::AccountId,
    {
        TokenLocked(CollectionId, StartIdx, AccountId),
        SubTokenMinted(CollectionId, StartIdx, MintAmount, AccountId),
    }
);

decl_error! {
    pub enum Error for Module<T: Config> {
        CollectionNotFound,
        TokenNotFound,
        PermissionDenied,
        AmountLessThanOne,
        WrongTokenType,
        NumOverflow,
        UriIsNone,
        AmountTooLarge,
        BurnedAmountShouldBeZero,
        BalanceInsufficient,
    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        #[weight = 10_000]
        pub fn lock(origin, collection_id: u128, start_idx: u128, is_non_fungible: bool) -> DispatchResult {
            ensure!(<pallet_nft::Collections<T>>::contains_key(collection_id), Error::<T>::CollectionNotFound);
            ensure!(<pallet_nft::Tokens<T>>::contains_key((collection_id, start_idx)), Error::<T>::TokenNotFound);

            let who = ensure_signed(origin.clone())?;
            let token = <pallet_nft::Tokens<T>>::get((collection_id, start_idx));

            ensure!(&token.owner == &who, Error::<T>::PermissionDenied);

            let sub_token_type = if is_non_fungible {
                SubTokenType::NonFungible
            } else { SubTokenType::Fungible };

            let sub_token_info = SubTokenInfo {
                owner: who.clone(),
                total_supply: 0,
                sub_token_type: Some(sub_token_type),
            };

            <pallet_nft::Module<T>>::transfer(origin.clone(), Self::account_id(), collection_id, start_idx)?;
            LockedTokens::<T>::insert((collection_id, start_idx), sub_token_info);

            Self::deposit_event(RawEvent::TokenLocked(collection_id, start_idx, who));
            Ok(())
        }

        #[weight = 10_000]
        pub fn mint_non_fungible(origin, collection_id: u128, start_idx: u128, amount: u128, uri: Vec<u8>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::_mint(who.clone(), collection_id, start_idx, amount, SubTokenType::NonFungible, Some(uri))?;
            Ok(())
        }

        #[weight = 10_000]
        pub fn mint_fungible(origin, collection_id: u128, start_idx: u128, amount: u128) -> DispatchResult {

            let who = ensure_signed(origin)?;

            let uri: Option<Vec<u8>> = None;

            Self::_mint(who.clone(), collection_id, start_idx, amount, SubTokenType::Fungible, uri)?;

            Ok(())
        }

        #[weight = 10_000]
        pub fn transfer_non_fungible(origin, receiver: T::AccountId, collection_id: u128, start_idx: u128, sub_token_start_idx: u128, amount:u128) -> DispatchResult {
            let who = ensure_signed(origin)?;
            // 检查是否存在
            // 检查subtoken类型
            // 检查权限
            // 检查转账数量
            ensure!(LockedTokens::<T>::contains_key((collection_id, start_idx)), Error::<T>::TokenNotFound);
            ensure!(SubTokens::<T>::contains_key((collection_id, start_idx), sub_token_start_idx), Error::<T>::TokenNotFound);
            ensure!(amount>=1, Error::<T>::AmountLessThanOne);

            let locked_token = Self::locked_tokens((collection_id, start_idx));

            if let Some(sub_token_type) = locked_token.sub_token_type {
                ensure!(sub_token_type == SubTokenType::NonFungible, Error::<T>::WrongTokenType);
            }

            let sub_token = Self::sub_tokens((collection_id, start_idx), sub_token_start_idx);
            ensure!(sub_token.owner == who, Error::<T>::PermissionDenied);

            let sub_token_amount = sub_token.end_idx.checked_sub(sub_token_start_idx).ok_or(Error::<T>::NumOverflow)?;
            let sub_token_amount = sub_token_amount.checked_add(1).ok_or(Error::<T>::NumOverflow)?;

            ensure!(sub_token_amount >= amount, Error::<T>::AmountTooLarge);

            // 开始转账

            let sender_balance = Self::address_balances((collection_id, start_idx), &who)
                .checked_sub(amount)
                .ok_or(Error::<T>::NumOverflow)?;

            let receiver_balance = Self::address_balances((collection_id, start_idx), &receiver)
                .checked_add(amount)
                .ok_or(Error::<T>::NumOverflow)?;

            let sender_start_idx = start_idx
                .checked_add(amount)
                .ok_or(Error::<T>::NumOverflow)?;

            let receiver_end_idx = sender_start_idx
                .checked_sub(1)
                .ok_or(Error::<T>::NumOverflow)?;

            let receiver_token = TokenInfo {
                end_idx: receiver_end_idx,
                owner: receiver.clone(),
                uri: sub_token.uri.clone(),
            };

            let is_transfer_all = receiver_token.end_idx == sub_token.end_idx;

            AddressBalances::<T>::insert((collection_id, start_idx), &who, sender_balance);
            AddressBalances::<T>::insert((collection_id, start_idx), &receiver, receiver_balance);
            SubTokens::<T>::insert((collection_id, start_idx), sub_token_start_idx, receiver_token);

            if !is_transfer_all {
                SubTokens::<T>::insert((collection_id, start_idx),sender_start_idx, sub_token);
            }

            Ok(())
        }

        #[weight = 10_000]
        pub fn transfer_fungible(origin, receiver: T::AccountId, collection_id: u128, start_idx: u128, amount:u128) -> DispatchResult {
            // 检查是否存在
            // 检查转账数量
            // 检查subtoken类型
            // 检查转账数量是否大于余额
            ensure!(LockedTokens::<T>::contains_key((collection_id, start_idx)), Error::<T>::TokenNotFound);
            ensure!(amount>=1, Error::<T>::AmountLessThanOne);

            let locked_token = Self::locked_tokens((collection_id, start_idx));

            if let Some(sub_token_type) = locked_token.sub_token_type {
                ensure!(sub_token_type == SubTokenType::Fungible, Error::<T>::WrongTokenType);
            }

            let who = ensure_signed(origin)?;

            let sender_balance = Self::address_balances((collection_id, start_idx), &who);
            ensure!(sender_balance>=amount, Error::<T>::AmountTooLarge);

            let sender_balance = sender_balance.checked_sub(amount).ok_or(Error::<T>::NumOverflow)?;

            let receiver_balance = Self::address_balances((collection_id, start_idx), &receiver)
                .checked_add(amount)
                .ok_or(Error::<T>::NumOverflow)?;

            AddressBalances::<T>::insert((collection_id, start_idx), &who, sender_balance);
            AddressBalances::<T>::insert((collection_id, start_idx), &receiver, receiver_balance);

            Ok(())
        }

        #[weight = 10_000]
        pub fn burn_non_fungible(origin, collection_id: u128, start_idx: u128, sub_token_start_idx:u128, amount:u128) ->DispatchResult {
            // 检查是否存在
            // 检查数量
            // 检查类型
            // 检查权限
            let who = ensure_signed(origin)?;

            ensure!(LockedTokens::<T>::contains_key((collection_id, start_idx)), Error::<T>::TokenNotFound);
            ensure!(amount>=1, Error::<T>::AmountLessThanOne);

            let locked_token = Self::locked_tokens((collection_id, start_idx));

            if let Some(sub_token_type) = locked_token.sub_token_type {
                ensure!(sub_token_type == SubTokenType::NonFungible, Error::<T>::WrongTokenType);
            }

            let sub_token = Self::sub_tokens((collection_id, start_idx), sub_token_start_idx);
            ensure!(sub_token.owner == who, Error::<T>::PermissionDenied);

            let sub_token_amount = sub_token.end_idx.checked_sub(sub_token_start_idx).ok_or(Error::<T>::NumOverflow)?;
            let sub_token_amount = sub_token_amount.checked_add(1).ok_or(Error::<T>::NumOverflow)?;

            ensure!(sub_token_amount >= amount, Error::<T>::AmountTooLarge);

            let balance = Self::address_balances((collection_id, start_idx), &who)
                .checked_sub(amount)
                .ok_or(Error::<T>::NumOverflow)?;
            let new_start_idx = sub_token_start_idx.checked_add(amount).ok_or(Error::<T>::NumOverflow)?;
            let is_burn_all = &new_start_idx == &sub_token.end_idx;

            let new_total_supply = locked_token
                .total_supply
                .checked_add(amount)
                .ok_or(Error::<T>::NumOverflow)?;

            let sub_token_info = SubTokenInfo {
                total_supply: new_total_supply,
                ..locked_token
            };

            let burned_amount = Self::burned_sub_tokens((collection_id, start_idx))
                .checked_add(amount)
                .ok_or(Error::<T>::NumOverflow)?;

            LockedTokens::<T>::insert((collection_id, start_idx), sub_token_info);
            AddressBalances::<T>::insert((collection_id, start_idx), &who, balance);
            SubTokens::<T>::remove((collection_id, start_idx), sub_token_start_idx);
            BurnedSubTokens::insert((collection_id, start_idx), burned_amount);

            if !is_burn_all {
                SubTokens::<T>::insert((collection_id, start_idx), new_start_idx, sub_token);
            }

            Ok(())
        }

        #[weight = 10_000]
        pub fn burn_fungible(origin, collection_id: u128, start_idx: u128, amount: u128) -> DispatchResult {
            // 检查是否存在
            // 检查数量
            // 检查类型
            let who = ensure_signed(origin)?;

            ensure!(LockedTokens::<T>::contains_key((collection_id, start_idx)), Error::<T>::TokenNotFound);
            ensure!(amount >= 1, Error::<T>::AmountLessThanOne);

            let locked_token = Self::locked_tokens((collection_id, start_idx));

            if let Some(sub_token_type) = locked_token.sub_token_type {
                ensure!(sub_token_type == SubTokenType::Fungible, Error::<T>::WrongTokenType);
            }

            let balance = Self::address_balances((collection_id, start_idx), &who);
            ensure!(balance >= amount, Error::<T>::AmountTooLarge);

            let balance = balance.checked_sub(amount).ok_or(Error::<T>::NumOverflow)?;

            let new_total_supply = locked_token
                .total_supply
                .checked_add(amount)
                .ok_or(Error::<T>::NumOverflow)?;

            let sub_token_info = SubTokenInfo {
                total_supply: new_total_supply,
                ..locked_token
            };

            let burned_amount = Self::burned_sub_tokens((collection_id, start_idx))
                .checked_add(amount)
                .ok_or(Error::<T>::NumOverflow)?;
            
            LockedTokens::<T>::insert((collection_id, start_idx), sub_token_info);
            AddressBalances::<T>::insert((collection_id, start_idx), &who, balance);
            BurnedSubTokens::insert((collection_id, start_idx), burned_amount);

            Ok(())
        }
    
        #[weight = 10_000]
        pub fn unlock(origin,collection_id: u128, start_idx: u128) -> DispatchResult {
            let who = ensure_signed(origin)?;
            // 检查是否存在
            // burn为0
            // total_supply = user balance
            // delete storage
            // transfer to nft pallet
            ensure!(LockedTokens::<T>::contains_key((collection_id, start_idx)), Error::<T>::TokenNotFound);
            ensure!(Self::burned_sub_tokens((collection_id, start_idx)) == 0, Error::<T>::BurnedAmountShouldBeZero);
            let locked_token = Self::locked_tokens((collection_id, start_idx));
            let balance = Self::address_balances((collection_id, start_idx), &who);
            ensure!(locked_token.total_supply == balance, Error::<T>::BalanceInsufficient);

            if let Some(sub_token_type) = locked_token.sub_token_type {
                if sub_token_type == SubTokenType::NonFungible {
                    SubTokens::<T>::remove_prefix((collection_id, start_idx));
                    LastTokenId::remove((collection_id, start_idx));
                } 
            }

            AddressBalances::<T>::remove((collection_id, start_idx), &who);
            BurnedSubTokens::remove((collection_id, start_idx));
            LockedTokens::<T>::remove((collection_id, start_idx));
            <pallet_nft::Module<T>>::transfer(frame_system::RawOrigin::Signed(Self::account_id()).into(), who, collection_id, start_idx)?;

            Ok(())
        }

    }
}

impl<T: Config> Module<T> {
    pub fn account_id() -> T::AccountId {
        PALLET_ID.into_account()
    }

    fn _mint(
        who: T::AccountId,
        collection_id: u128,
        start_idx: u128,
        amount: u128,
        token_type: SubTokenType,
        uri: Option<Vec<u8>>,
    ) -> DispatchResult {
        ensure!(amount >= 1, Error::<T>::AmountLessThanOne);
        ensure!(
            LockedTokens::<T>::contains_key((collection_id, start_idx)),
            Error::<T>::TokenNotFound
        );

        let locked_token = Self::locked_tokens((collection_id, start_idx));
        ensure!(
            locked_token.owner == who.clone(),
            Error::<T>::PermissionDenied
        );

        if let Some(sub_token_type) = locked_token.sub_token_type {
            ensure!(sub_token_type == token_type, Error::<T>::WrongTokenType);
        }

        let new_total_supply = locked_token
            .total_supply
            .checked_add(amount)
            .ok_or(Error::<T>::NumOverflow)?;

        let sub_token_info = SubTokenInfo {
            total_supply: new_total_supply,
            ..locked_token
        };

        let owner_balance = Self::address_balances((collection_id, start_idx), who.clone())
            .checked_add(amount)
            .ok_or(Error::<T>::NumOverflow)?;

        if token_type == SubTokenType::NonFungible {
            // Err(Error::<T>::NoneValue)?
            ensure!(!uri.is_none(), Error::<T>::UriIsNone);
            if let Some(uri_value) = uri {
                let sub_start_idx = if LastTokenId::contains_key((collection_id, start_idx)) {
                    Self::last_token_id((collection_id, start_idx))
                        .checked_add(1)
                        .ok_or(Error::<T>::NumOverflow)?
                } else {
                    0
                };

                let end_idx = sub_start_idx
                    .checked_add(amount)
                    .ok_or(Error::<T>::NumOverflow)?;

                let end_idx = end_idx.checked_sub(1).ok_or(Error::<T>::NumOverflow)?;

                let token = TokenInfo {
                    end_idx: end_idx,
                    owner: who.clone(),
                    uri: uri_value,
                };

                LastTokenId::insert((collection_id, start_idx), end_idx);
                SubTokens::<T>::insert((collection_id, start_idx), sub_start_idx, token);
            }
        }

        AddressBalances::<T>::insert((collection_id, start_idx), who.clone(), owner_balance);
        LockedTokens::<T>::insert((collection_id, start_idx), sub_token_info);

        Ok(())
    }
}