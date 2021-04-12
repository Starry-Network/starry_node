#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure,
};
use frame_system::ensure_signed;
use pallet_collection::{CollectionInfo, CollectionInterface, TokenType};
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

pub trait Config: frame_system::Config {
    /// Because this pallet emits events, it depends on the runtime's definition of an event.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
    type Collection: CollectionInterface<Self::Hash, Self::AccountId>;
}

decl_storage! {
    trait Store for Module<T: Config> as NFTModule {
        // collection_id => nft_id
        pub LastTokenId get(fn last_token_id): map hasher(blake2_128_concat) T::Hash => u128;

        // (collection_id, address) => balance;
        pub AddressBalances get (fn address_balances): map hasher(blake2_128_concat) (T::Hash, T::AccountId) => u128;

        // (collection_id, start_idx) => nft_info
        pub Tokens get(fn tokens): double_map hasher(blake2_128_concat) T::Hash, hasher(blake2_128_concat) u128 => TokenInfo<T::AccountId>;

        // collection_id => burned amount
        pub BurnedTokens get(fn burned_tokens): map hasher(blake2_128_concat) T::Hash => u128;
    }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Config>::AccountId,
        Hash = <T as frame_system::Config>::Hash,
    {
        // [receiver, collection_id, start_idx, end_idx, collection_total_supply]
        NonFungibleTokenMinted(AccountId, Hash, u128, u128, u128),

        // [receiver, collection_id, amount, collection_total_supply]
        FungibleTokenMinted(AccountId, Hash, u128, u128),

        // [sender, receiver, collection_id, start_idx, amount]
        NonFungibleTokenTransferred(AccountId, AccountId, Hash, u128, u128),

        // [sender, receiver, collection_id, amount]
        FungibleTokenTransferred(AccountId, AccountId, Hash, u128),

        // [sender, collection_id, start_idx, amount, total_supply]
        NonFungibleTokenBurned(AccountId, Hash, u128, u128, u128),
        // [sender, collection_id, amount, total_supply]
        FungibleTokenBurned(AccountId, Hash, u128, u128),
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
        WrongTokenType
    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {

        fn deposit_event() = default;

        #[weight = 10_000]
        pub fn mint_fungible(origin, receiver: T::AccountId, collection_id: T::Hash, amount: u128) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(
                T::Collection::collection_exist(collection_id),
                Error::<T>::CollectionNotFound
            );

            let collection = T::Collection::get_collection(collection_id);
            ensure!(collection.owner == who, Error::<T>::PermissionDenied);

            Self::_mint_fungible(receiver, collection_id, amount, &collection)?;

            Ok(())
        }

        #[weight = 10_000]
        pub fn mint_non_fungible(origin, receiver: T::AccountId, collection_id: T::Hash, uri: Vec<u8>, amount:u128) -> DispatchResult {
            ensure!(
                T::Collection::collection_exist(collection_id),
                Error::<T>::CollectionNotFound
            );

            let who = ensure_signed(origin)?;
            // ensure collection owner = origin
            let collection =T::Collection::get_collection(collection_id);
            ensure!(collection.owner == who, Error::<T>::PermissionDenied);

            Self::_mint_non_fungible(receiver, collection_id, amount, uri, &collection)?;

            Ok(())
        }

        #[weight = 10_000]
         pub fn transfer_non_fungible(origin, receiver: T::AccountId, collection_id: T::Hash, start_idx: u128, amount:u128) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Self::_transfer_non_fungible(who, receiver, collection_id, start_idx, amount)?;
            Ok(())
        }

        #[weight = 10_000]
         pub fn transfer_fungible(origin, receiver: T::AccountId, collection_id: T::Hash, amount:u128) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Self::_transfer_fungible(who, receiver, collection_id, amount)?;
            Ok(())
        }

        #[weight = 10_000]
        pub fn burn_non_fungible(origin, collection_id: T::Hash, start_idx:u128, amount:u128) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Self::_burn_non_fungible(who, collection_id, start_idx, amount)?;

            Ok(())
        }

        #[weight = 10_000]
        pub fn burn_fungible(origin, collection_id: T::Hash, amount:u128) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Self::_burn_fungible(who, collection_id, amount)?;

            Ok(())
        }
    }
}

pub trait NFTInterface<Hash, AccountId> {
    fn token_exist(collection_id: Hash, token_id: u128) -> bool;

    fn get_nft_token(collection_id: Hash, token_id: u128) -> TokenInfo<AccountId>;

    fn get_balance(collection_id: &Hash, who: &AccountId) -> u128;

    fn get_burned_amount(collection_id: &Hash) -> u128;

    fn destory_collection(collection_id: &Hash);

    fn _mint_non_fungible(
        receiver: AccountId,
        collection_id: Hash,
        amount: u128,
        uri: Vec<u8>,
        collection: &CollectionInfo<AccountId>,
    ) -> DispatchResult;

    fn _mint_fungible(
        receiver: AccountId,
        collection_id: Hash,
        amount: u128,
        collection: &CollectionInfo<AccountId>,
    ) -> DispatchResult;

    fn _transfer_non_fungible(
        who: AccountId,
        receiver: AccountId,
        collection_id: Hash,
        start_idx: u128,
        amount: u128,
    ) -> DispatchResult;

    fn _transfer_fungible(
        who: AccountId,
        receiver: AccountId,
        collection_id: Hash,
        amount: u128,
    ) -> DispatchResult;

    fn _burn_non_fungible(
        who: AccountId,
        collection_id: Hash,
        start_idx: u128,
        amount: u128,
    ) -> DispatchResult;

    fn _burn_fungible(who: AccountId, collection_id: Hash, amount: u128) -> DispatchResult;
}

impl<T: Config> NFTInterface<T::Hash, T::AccountId> for Module<T> {
    fn token_exist(collection_id: T::Hash, token_id: u128) -> bool {
        Tokens::<T>::contains_key(collection_id, token_id)
    }

    fn get_nft_token(collection_id: T::Hash, token_id: u128) -> TokenInfo<T::AccountId> {
        Self::tokens(collection_id, token_id)
    }

    fn get_balance(collection_id: &T::Hash, who: &T::AccountId) -> u128 {
        Self::address_balances((collection_id, who))
    }

    fn get_burned_amount(collection_id: &T::Hash) -> u128 {
        Self::burned_tokens(collection_id)
    }

    fn destory_collection(collection_id: &T::Hash) {
        Tokens::<T>::remove_prefix(collection_id)
    }

    fn _mint_non_fungible(
        receiver: T::AccountId,
        collection_id: T::Hash,
        amount: u128,
        uri: Vec<u8>,
        collection: &CollectionInfo<T::AccountId>,
    ) -> DispatchResult {
        ensure!(amount >= 1, Error::<T>::AmountLessThanOne);

        if let Some(token_type) = collection.token_type {
            ensure!(
                token_type == TokenType::NonFungible,
                Error::<T>::WrongTokenType
            );
        }

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
            owner: receiver.clone(),
            uri,
        };

        let owner_balance = Self::address_balances((collection_id, receiver.clone()))
            .checked_add(amount)
            .ok_or(Error::<T>::NumOverflow)?;

        // let new_total_supply =
        //     <pallet_collection::Module<T>>::add_total_supply(collection_id, amount)?;
        let new_total_supply = T::Collection::add_total_supply(collection_id, amount)?;

        LastTokenId::<T>::insert(collection_id, end_idx);
        AddressBalances::<T>::insert((collection_id, &receiver), owner_balance);
        Tokens::<T>::insert(collection_id, start_idx, token);

        // [receiver, collection_id, start_idx, end_idx, new_total_supply]
        Self::deposit_event(RawEvent::NonFungibleTokenMinted(
            receiver,
            collection_id,
            start_idx,
            end_idx,
            new_total_supply,
        ));

        Ok(())
    }

    fn _mint_fungible(
        receiver: T::AccountId,
        collection_id: T::Hash,
        amount: u128,
        collection: &CollectionInfo<T::AccountId>,
    ) -> DispatchResult {
        ensure!(amount >= 1, Error::<T>::AmountLessThanOne);

        if let Some(token_type) = collection.token_type {
            ensure!(
                token_type == TokenType::Fungible,
                Error::<T>::WrongTokenType
            );
        }

        let owner_balance = Self::address_balances((collection_id, &receiver))
            .checked_add(amount)
            .ok_or(Error::<T>::NumOverflow)?;

        let new_total_supply = T::Collection::add_total_supply(collection_id, amount)?;

        // let new_total_supply =
        //     <pallet_collection::Module<T>>::add_total_supply(collection_id, amount)?;

        AddressBalances::<T>::insert((collection_id, &receiver), owner_balance);

        // [receiver, collection_id, amount, collection_total_supply]
        Self::deposit_event(RawEvent::FungibleTokenMinted(
            receiver,
            collection_id,
            amount,
            new_total_supply,
        ));

        Ok(())
    }

    fn _transfer_non_fungible(
        who: T::AccountId,
        receiver: T::AccountId,
        collection_id: T::Hash,
        start_idx: u128,
        amount: u128,
    ) -> DispatchResult {
        ensure!(&who != &receiver, Error::<T>::ReceiverIsSender);
        ensure!(amount >= 1, Error::<T>::AmountLessThanOne);

        ensure!(
            T::Collection::collection_exist(collection_id),
            Error::<T>::CollectionNotFound
        );

        ensure!(
            Tokens::<T>::contains_key(collection_id, start_idx),
            Error::<T>::TokenNotFound
        );

        let collection = T::Collection::get_collection(collection_id);
        if let Some(token_type) = collection.token_type {
            ensure!(
                token_type == TokenType::NonFungible,
                Error::<T>::WrongTokenType
            );
        }

        let token = Self::tokens(collection_id, start_idx);
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

        let receiver_start_idx = &token
            .end_idx
            .checked_sub(amount - 1)
            .ok_or(Error::<T>::NumOverflow)?;

        let receiver_end_idx = &token.end_idx;

        let is_transfer_all = receiver_start_idx.clone() == start_idx;

        let sender_end_idx = if !is_transfer_all {
            receiver_start_idx
                .clone()
                .checked_sub(1)
                .ok_or(Error::<T>::NumOverflow)?
        } else {
            0
        };

        let receiver_token = TokenInfo {
            end_idx: *receiver_end_idx,
            owner: receiver.clone(),
            uri: token.uri.clone(),
        };

        AddressBalances::<T>::insert((collection_id, who.clone()), sender_balance);
        AddressBalances::<T>::insert((collection_id, receiver.clone()), receiver_balance);
        Tokens::<T>::insert(collection_id, receiver_start_idx, receiver_token);

        if !is_transfer_all {
            let sender_token = TokenInfo {
                end_idx: sender_end_idx,
                ..token
            };
            Tokens::<T>::insert(collection_id, start_idx, sender_token);
        }

        Self::deposit_event(RawEvent::NonFungibleTokenTransferred(
            who,
            receiver,
            collection_id,
            start_idx,
            amount,
        ));

        Ok(())
    }

    fn _transfer_fungible(
        who: T::AccountId,
        receiver: T::AccountId,
        collection_id: T::Hash,
        amount: u128,
    ) -> DispatchResult {
        ensure!(amount >= 1, Error::<T>::AmountLessThanOne);
        ensure!(&who != &receiver, Error::<T>::ReceiverIsSender);
        ensure!(
            T::Collection::collection_exist(collection_id),
            Error::<T>::CollectionNotFound
        );

        let collection = T::Collection::get_collection(collection_id);
        if let Some(token_type) = collection.token_type {
            ensure!(
                token_type == TokenType::Fungible,
                Error::<T>::WrongTokenType
            );
        }

        let sender_balance = Self::address_balances((collection_id, &who));
        ensure!(sender_balance >= amount, Error::<T>::AmountTooLarge);

        let sender_balance = sender_balance
            .checked_sub(amount)
            .ok_or(Error::<T>::NumOverflow)?;
        let receiver_balance = Self::address_balances((collection_id, &receiver))
            .checked_add(amount)
            .ok_or(Error::<T>::NumOverflow)?;

        AddressBalances::<T>::insert((collection_id, who.clone()), sender_balance);
        AddressBalances::<T>::insert((collection_id, receiver.clone()), receiver_balance);

        Self::deposit_event(RawEvent::FungibleTokenTransferred(
            who,
            receiver,
            collection_id,
            amount,
        ));

        Ok(())
    }

    fn _burn_non_fungible(
        who: T::AccountId,
        collection_id: T::Hash,
        start_idx: u128,
        amount: u128,
    ) -> DispatchResult {
        ensure!(amount >= 1, Error::<T>::AmountLessThanOne);

        ensure!(
            T::Collection::collection_exist(collection_id),
            Error::<T>::CollectionNotFound
        );
        ensure!(
            Tokens::<T>::contains_key(collection_id, start_idx),
            Error::<T>::TokenNotFound
        );

        let collection = T::Collection::get_collection(collection_id);
        if let Some(token_type) = collection.token_type {
            ensure!(
                token_type == TokenType::NonFungible,
                Error::<T>::WrongTokenType
            );
        }

        let token = Self::tokens(collection_id, start_idx);

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

        let burn_start_idx = &token
            .end_idx
            .checked_sub(amount - 1)
            .ok_or(Error::<T>::NumOverflow)?;

        let is_burn_all = burn_start_idx.clone() == start_idx;

        let new_end_idx = if !is_burn_all {
            burn_start_idx
                .clone()
                .checked_sub(1)
                .ok_or(Error::<T>::NumOverflow)?
        } else {
            0
        };

        let burn_amount = Self::burned_tokens(collection_id)
            .checked_add(amount)
            .ok_or(Error::<T>::NumOverflow)?;

        let new_total_supply = T::Collection::sub_total_supply(collection_id, amount)?;

        if is_burn_all {
            Tokens::<T>::remove(collection_id, start_idx);
        } else {
            let token = TokenInfo {
                end_idx: new_end_idx,
                ..token
            };
            Tokens::<T>::insert(collection_id, start_idx, token);
        }

        AddressBalances::<T>::insert((collection_id, who.clone()), balance);
        BurnedTokens::<T>::insert(collection_id, burn_amount);
        
        // [sender, amount, collection_id, start_idx]
        Self::deposit_event(RawEvent::NonFungibleTokenBurned(
            who,
            collection_id,
            start_idx,
            amount,
            new_total_supply,
        ));

        Ok(())
    }

    fn _burn_fungible(who: T::AccountId, collection_id: T::Hash, amount: u128) -> DispatchResult {
        ensure!(amount >= 1, Error::<T>::AmountLessThanOne);

        ensure!(
            T::Collection::collection_exist(collection_id),
            Error::<T>::CollectionNotFound
        );

        let collection = T::Collection::get_collection(collection_id);
        if let Some(token_type) = collection.token_type {
            ensure!(
                token_type == TokenType::Fungible,
                Error::<T>::WrongTokenType
            );
        }

        let balance = Self::address_balances((collection_id, &who));
        ensure!(balance >= amount, Error::<T>::AmountTooLarge);

        let balance = balance.checked_sub(amount).ok_or(Error::<T>::NumOverflow)?;
        let burn_amount = Self::burned_tokens(collection_id)
            .checked_add(amount)
            .ok_or(Error::<T>::NumOverflow)?;

        let new_total_supply = T::Collection::sub_total_supply(collection_id, amount)?;

        AddressBalances::<T>::insert((collection_id, who.clone()), balance);
        BurnedTokens::<T>::insert(collection_id, burn_amount);
        // [sender, amount, collection_id, start_idx]
        Self::deposit_event(RawEvent::FungibleTokenBurned(
            who,
            collection_id,
            amount,
            new_total_supply,
        ));

        Ok(())
    }
}
