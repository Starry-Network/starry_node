#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure,
};
use frame_system::ensure_signed;
use pallet_collection;
use sp_std::vec::Vec;

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

pub trait Config: frame_system::Config + pallet_collection::Config {
    /// Because this pallet emits events, it depends on the runtime's definition of an event.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
}

decl_storage! {
    trait Store for Module<T: Config> as NFTModule {
        // collection_id => nft_id
        pub LastTokenId get(fn last_token_id): map hasher(blake2_128_concat) T::Hash => u128;

        // (collection_id, address) => balance;
        pub AddressBalances get (fn address_balances): map hasher(blake2_128_concat) (T::Hash, T::AccountId) => u128;

        // (collection_id, start_idx) => nft_info
        pub Tokens get(fn tokens): map hasher(blake2_128_concat) (T::Hash, u128) =>  TokenInfo<T::AccountId>;
    }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Config>::AccountId,
        Hash = <T as frame_system::Config>::Hash,
    {
        // [sender, collection_id, start_idx, end_idx, collection_total_supply]
        TokenMinted(AccountId, Hash, u128, u128, u128),

        // [sender, receiver, amount, collection_id, start_idx]
        TokenTransferred(AccountId, AccountId, u128, Hash, u128),

        // [sender, amount, collection_id, start_idx, total_supply]
        TokenBurned(AccountId, u128, Hash, u128, u128),
    }
);

// Errors inform users that something went wrong.
decl_error! {
    pub enum Error for Module<T: Config> {
        NumOverflow,
        AmountLessThanOne,
        AmountTooLarge,
        PermissionDenied,
        CollectionNotFound,
        TokenNotFound,
        ReceiverIsSender,
    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {

        fn deposit_event() = default;

        #[weight = 10_000]
        pub fn mint(origin, collection_id: T::Hash, uri: Vec<u8>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Self::_mint(who, collection_id, 1, uri)?;

            Ok(())
        }

        #[weight = 10_000]
        pub fn batch_mint(origin, collection_id: T::Hash, uri: Vec<u8>, amount: u128) -> DispatchResult {
            ensure!(amount >= 1, Error::<T>::AmountLessThanOne);

            let who = ensure_signed(origin)?;

            Self::_mint(who, collection_id, amount, uri)?;

            Ok(())
        }

        #[weight = 10_000]
        pub fn transfer(origin, receiver: T::AccountId, collection_id: T::Hash, start_idx: u128) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Self::_transfer(who, receiver, collection_id, start_idx, 1)?;
            Ok(())
        }

        #[weight = 10_000]
        pub fn batch_transfer(origin, receiver: T::AccountId, collection_id: T::Hash, start_idx: u128, amount: u128) -> DispatchResult {
            ensure!(amount >= 1, Error::<T>::AmountLessThanOne);

            let who = ensure_signed(origin)?;

            Self::_transfer(who, receiver, collection_id, start_idx, amount)?;
            Ok(())
        }

        #[weight = 10_000]
        pub fn burn(origin, collection_id: T::Hash, start_idx:u128) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Self::_burn(who, collection_id, start_idx, 1)?;

            Ok(())
        }

        #[weight = 10_000]
        pub fn batch_burn(origin, collection_id: T::Hash, start_idx: u128, amount: u128) -> DispatchResult {
            ensure!(amount >= 1, Error::<T>::AmountLessThanOne);

            let who = ensure_signed(origin)?;

            Self::_burn(who, collection_id, start_idx, amount)?;

            Ok(())
        }

    }
}

impl<T: Config> Module<T> {
    fn _mint(
        who: T::AccountId,
        collection_id: T::Hash,
        amount: u128,
        uri: Vec<u8>,
    ) -> DispatchResult {
        ensure!(
            <pallet_collection::Collections<T>>::contains_key(collection_id),
            Error::<T>::CollectionNotFound
        );

        let start_idx = if LastTokenId::<T>::contains_key(collection_id) {
            Self::last_token_id(collection_id)
                .checked_add(1)
                .ok_or(Error::<T>::NumOverflow)?
        } else {
            0
        };

        let end_idx = start_idx
            .checked_add(amount)
            .ok_or(Error::<T>::NumOverflow)?;
        let end_idx = end_idx.checked_sub(1).ok_or(Error::<T>::NumOverflow)?;

        let token = TokenInfo {
            end_idx: end_idx,
            owner: who.clone(),
            uri,
        };

        let owner_balance = Self::address_balances((collection_id, who.clone()))
            .checked_add(amount)
            .ok_or(Error::<T>::NumOverflow)?;

        let new_total_supply =
            <pallet_collection::Module<T>>::add_total_supply(collection_id, amount)?;
        LastTokenId::<T>::insert(collection_id, end_idx);
        AddressBalances::<T>::insert((collection_id, who.clone()), owner_balance);
        Tokens::<T>::insert((collection_id, start_idx), token);

        // [sender, collection_id, start_idx, end_idx, new_total_supply]
        Self::deposit_event(RawEvent::TokenMinted(
            who,
            collection_id,
            start_idx,
            end_idx,
            new_total_supply,
        ));

        Ok(())
    }

    pub fn _transfer(
        who: T::AccountId,
        receiver: T::AccountId,
        collection_id: T::Hash,
        start_idx: u128,
        amount: u128,
    ) -> DispatchResult {
        ensure!(&who != &receiver, Error::<T>::ReceiverIsSender);
        ensure!(
            <pallet_collection::Collections<T>>::contains_key(collection_id),
            Error::<T>::CollectionNotFound
        );
        ensure!(
            Tokens::<T>::contains_key((collection_id, start_idx)),
            Error::<T>::TokenNotFound
        );

        let token = Self::tokens((collection_id, start_idx));
        ensure!(&token.owner == &who, Error::<T>::PermissionDenied);

        if amount > 1 {
            let token_amount = &token
                .end_idx
                .checked_sub(start_idx)
                .ok_or(Error::<T>::NumOverflow)?;
            let token_amount = &token_amount.checked_add(1).ok_or(Error::<T>::NumOverflow)?;

            ensure!(token_amount >= &amount, Error::<T>::AmountTooLarge);
        }

        let sender_balance = Self::address_balances((collection_id, &who))
            .checked_sub(amount)
            .ok_or(Error::<T>::NumOverflow)?;
        let receiver_balance = Self::address_balances((collection_id, &receiver))
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
            uri: token.uri.clone(),
        };

        let is_transfer_all = &receiver_token.end_idx == &token.end_idx;

        AddressBalances::<T>::insert((collection_id, who.clone()), sender_balance);
        AddressBalances::<T>::insert((collection_id, receiver.clone()), receiver_balance);
        Tokens::<T>::insert((collection_id, start_idx), receiver_token);

        if !is_transfer_all {
            Tokens::<T>::insert((collection_id, sender_start_idx), token);
        }

        Self::deposit_event(RawEvent::TokenTransferred(
            who,
            receiver,
            amount,
            collection_id,
            start_idx,
        ));

        Ok(())
    }

    fn _burn(
        who: T::AccountId,
        collection_id: T::Hash,
        start_idx: u128,
        amount: u128,
    ) -> DispatchResult {
        ensure!(
            <pallet_collection::Collections<T>>::contains_key(collection_id),
            Error::<T>::CollectionNotFound
        );
        ensure!(
            Tokens::<T>::contains_key((collection_id, start_idx)),
            Error::<T>::TokenNotFound
        );

        let token = Self::tokens((collection_id, start_idx));

        ensure!(&token.owner == &who, Error::<T>::PermissionDenied);

        if amount > 1 {
            let token_amount = &token
                .end_idx
                .checked_sub(start_idx)
                .ok_or(Error::<T>::NumOverflow)?;
            let token_amount = &token_amount.checked_add(1).ok_or(Error::<T>::NumOverflow)?;

            ensure!(token_amount >= &amount, Error::<T>::AmountTooLarge);
        }
        let balance = Self::address_balances((collection_id, &who))
            .checked_sub(amount)
            .ok_or(Error::<T>::NumOverflow)?;
        let new_start_idx = start_idx
            .checked_add(amount)
            .ok_or(Error::<T>::NumOverflow)?;

        let is_burn_all = &new_start_idx == &token.end_idx;

        let new_total_supply =
            <pallet_collection::Module<T>>::sub_total_supply(collection_id, amount)?;

        AddressBalances::<T>::insert((collection_id, who.clone()), balance);
        Tokens::<T>::remove((collection_id, start_idx));

        if !is_burn_all {
            Tokens::<T>::insert((collection_id, new_start_idx), token);
        }

        // [sender, amount, collection_id, start_idx]
        Self::deposit_event(RawEvent::TokenBurned(
            who,
            amount,
            collection_id,
            start_idx,
            new_total_supply,
        ));

        Ok(())
    }
}
