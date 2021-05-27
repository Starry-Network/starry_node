//! # NFT Module
//! 
//! - [`Config`]
//! - [`Call`]
//!
//! Create a batch of NonFungible or Fungible Tokens.
//! 
//! ### Terminology
//! 
//! * **Mint:** Mint one or a batch of NFTs or some FTs (SemiFts)
//! * **transfer:** Transfer one or a batch of tokens from one account to another account
//! * **Burn:** Destroy one or a batch of tokens from an account. This is an irreversible operation.
//! * **Fungible Token:** Fungible or semi-fungible token
//! * **Non-fungible asset:** Unique or have some copies of the token.
//! 
//! ## Interface
//! 
//! ### Dispatchable Functions
//! 
//! * `mint_fungible` - Mint some FTs
//! * `mint_non_fungible` - Mint one or a batch of NFTs
//! * `transfer_fungible` - Transfer some FTs to another account
//! * `transfer_non_fungible` - Transfer one or a batch of NFTs to another account
//! * `burn_fungible` - Destroy some FTs by owner
//! * `burn_non_fungible` - Destroy one or a batch of NFTs NFTs by owner
//! 
//! [`Call`]: ./enum.Call.html
//! [`Config`]: ./trait.Config.html

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::{DispatchError, DispatchResult},
    ensure,
};
use frame_system::ensure_signed;
use pallet_collection::{CollectionInfo, CollectionInterface, TokenType};
use sp_std::vec::Vec;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// Details of a NFT
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
        /// The set of collection last token id. collection_id => nft_id
        pub LastTokenId get(fn last_token_id): map hasher(blake2_128_concat) T::Hash => u128;

        /// Account balance in collection. (collection_id, address) => balance;
        pub AddressBalances get (fn address_balances): map hasher(blake2_128_concat) (T::Hash, T::AccountId) => u128;

        /// The set of minted NFTs. (collection_id, start_idx) => nft_info
        pub Tokens get(fn tokens): double_map hasher(blake2_128_concat) T::Hash, hasher(blake2_128_concat) u128 => TokenInfo<T::AccountId>;

        /// The set of Collection burned count. collection_id => burned amount
        pub BurnedTokens get(fn burned_tokens): map hasher(blake2_128_concat) T::Hash => u128;
    }
}

decl_event!(
    /// Events for this module.
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Config>::AccountId,
        Hash = <T as frame_system::Config>::Hash,
    {
        /// One or a batch of NFTs were Minted. \[collection_id, start_idx, end_idx\]
        NonFungibleTokenMinted(Hash, u128, u128),

        /// Some FTs were minted. \[collection_id\]
        FungibleTokenMinted(Hash),

        /// One or a batch of NFTs were transfered to another account. \[receiver, collection_id\]
        NonFungibleTokenTransferred(AccountId, Hash),

        // some FTs were transfered to another account. \[receiver, collection_id\]
        FungibleTokenTransferred(AccountId, Hash),

        // One or a batch of NFTs were burned. \[sender, collection_id\]
        NonFungibleTokenBurned(AccountId, Hash),

        // Some FTs were burned.  \[sender, collection_id\]
        FungibleTokenBurned(AccountId, Hash),
    }
);

decl_error! {
    /// Errors inform users that something went wrong.
    pub enum Error for Module<T: Config> {
        /// Number is too large or less than zero.
        NumOverflow,
        /// The minimum value is 1.
        AmountLessThanOne,
        /// Amount too large (amount is more than own).
        AmountTooLarge,
        /// No permission to perform this operation.
        PermissionDenied,
        /// Collection does not exist.
        CollectionNotFound,
        /// Token does not exist.
        TokenNotFound,
        /// The recipient cannot be the sender.
        ReceiverIsSender,
        /// Wrong token type, for example: cann't mint FTs in NFT Collection.
        WrongTokenType
    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {

        fn deposit_event() = default;

        /// Mint some FTs.
        ///
        /// The dispatch origin of this call must be _Signed_.
        ///
        /// Parameters:
        /// - `receiver`: The address that accepts minted tokens.
        /// - `collection_id`: The id of the collection whose token type is FT.
        /// - `amount`: How many tokens to mint
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

            Self::deposit_event(RawEvent::FungibleTokenMinted(collection_id));

            Ok(())
        }

        /// Mint one or a batch of NFTs.
        ///
        /// If mint a batch of NFTs, end_idx will be stored in TokenInfo.
        /// From start_idx to end_idx can be used to represent a batch of NFTs.
        ///
        /// The dispatch origin of this call must be _Signed_.
        ///
        /// Parameters:
        /// - `receiver`: The address that accepts minted tokens.
        /// - `collection_id`: The id of the collection whose token type is NFT.
        /// - `uri`: Used to get the detailed information of the collection such as name,
        /// description, cover_image, which can be the CID of ipfs or a URL.
        /// - `amount`: How many tokens to mint.
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

            let (start_idx, end_idx) = Self::_mint_non_fungible(receiver, collection_id, amount, uri, &collection)?;

            Self::deposit_event(RawEvent::NonFungibleTokenMinted(
                collection_id,
                start_idx,
                end_idx,
            ));

            Ok(())
        }

        /// Transfer some FTs to another account.
        ///
        /// The dispatch origin of this call must be _Signed_.
        ///
        /// Parameters:
        /// - `receiver`: The address that accepts transfered tokens.
        /// - `collection_id`: The id of the collection whose token type is FT and the
        /// token to be transferred is in this collection
        /// - `amount`: How many tokens to transfer.
        #[weight = 10_000]
        pub fn transfer_fungible(origin, receiver: T::AccountId, collection_id: T::Hash, amount:u128) -> DispatchResult {
           let who = ensure_signed(origin)?;

           Self::_transfer_fungible(who, receiver.clone(), collection_id, amount)?;

           Self::deposit_event(RawEvent::FungibleTokenTransferred(receiver, collection_id));

           Ok(())
       }

        /// Transfer one or a batch of NFTs to another account.
        ///
        /// If you need to transfer a batch of NFTs, the nft id will be the starting index,
        /// note that the number of transfers cannot exceed (end_idx - start_idx) + 1.
        ///
        /// The dispatch origin of this call must be _Signed_.
        ///
        /// Parameters:
        /// - `receiver`: The address that accepts transfered tokens.
        /// - `collection_id`: The id of the collection whose token type is NFT and the
        /// token to be transferred is in this collection.
        /// - `start_idx`: The index of the token or a batch of tokens to be transferred.
        /// - `amount`: How many tokens to transfer.
        #[weight = 10_000]
         pub fn transfer_non_fungible(origin, receiver: T::AccountId, collection_id: T::Hash, start_idx: u128, amount: u128) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Self::_transfer_non_fungible(who, receiver.clone(), collection_id, start_idx, amount)?;

            Self::deposit_event(RawEvent::NonFungibleTokenTransferred(
                receiver,
                collection_id,
            ));

            Ok(())
        }

        /// Burn some FTs to another account.
        ///
        /// The dispatch origin of this call must be _Signed_.
        ///
        /// Parameters:
        /// - `collection_id`: The id of the collection whose token type is FT and the
        /// token to be burned is in this collection
        /// - `amount`: How many tokens to burn.
        #[weight = 10_000]
        pub fn burn_fungible(origin, collection_id: T::Hash, amount:u128) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Self::_burn_fungible(who.clone(), collection_id, amount)?;

            Self::deposit_event(RawEvent::FungibleTokenBurned(who, collection_id));

            Ok(())
        }

        /// Burn one or a batch of NFTS.
        ///
        /// The dispatch origin of this call must be _Signed_.
        ///
        /// Parameters:
        /// - `collection_id`: The id of the collection whose token type is NFT and the
        /// token to be burned is in this collection
        /// - `start_idx`: The index of the token or a batch of tokens to be burned.
        /// - `amount`: How many tokens to burn.
        #[weight = 10_000]
        pub fn burn_non_fungible(origin, collection_id: T::Hash, start_idx: u128, amount: u128) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Self::_burn_non_fungible(who.clone(), collection_id, start_idx, amount)?;

            Self::deposit_event(RawEvent::NonFungibleTokenBurned(who, collection_id));

            Ok(())
        }
    }
}

pub trait NFTInterface<Hash, AccountId> {
    /// Check whether the token exists by collection_id and token_id.
    fn token_exist(collection_id: Hash, token_id: u128) -> bool;
    /// Get token by collection_id and token_id.
    fn get_nft_token(collection_id: Hash, token_id: u128) -> TokenInfo<AccountId>;
    /// Get the balance of an account in a collection.
    fn get_balance(collection_id: &Hash, who: &AccountId) -> u128;
    /// Get the count of tokens burned in a collection.
    fn get_burned_amount(collection_id: &Hash) -> u128;
    /// Destory a collection by collection_id.
    fn destory_collection(collection_id: &Hash);
    /// Mint NFTs
    fn _mint_non_fungible(
        receiver: AccountId,
        collection_id: Hash,
        amount: u128,
        uri: Vec<u8>,
        collection: &CollectionInfo<AccountId>,
    ) -> Result<(u128, u128), DispatchError>;
    /// Mint FTs
    fn _mint_fungible(
        receiver: AccountId,
        collection_id: Hash,
        amount: u128,
        collection: &CollectionInfo<AccountId>,
    ) -> DispatchResult;
    /// Transfer NFTs to another account.
    fn _transfer_non_fungible(
        who: AccountId,
        receiver: AccountId,
        collection_id: Hash,
        start_idx: u128,
        amount: u128,
    ) -> DispatchResult;
    /// Transfer FTs to another account.
    fn _transfer_fungible(
        who: AccountId,
        receiver: AccountId,
        collection_id: Hash,
        amount: u128,
    ) -> DispatchResult;
    /// burn NFTs.
    fn _burn_non_fungible(
        who: AccountId,
        collection_id: Hash,
        start_idx: u128,
        amount: u128,
    ) -> DispatchResult;
    /// burn FTs.
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
    ) -> Result<(u128, u128), DispatchError> {
        // Result<Hash, DispatchError>;
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
            end_idx,
            owner: receiver.clone(),
            uri,
        };

        let owner_balance = Self::address_balances((collection_id, receiver.clone()))
            .checked_add(amount)
            .ok_or(Error::<T>::NumOverflow)?;

        // let new_total_supply =
        //     <pallet_collection::Module<T>>::add_total_supply(collection_id, amount)?;
        T::Collection::add_total_supply(collection_id, amount)?;

        LastTokenId::<T>::insert(collection_id, end_idx);
        AddressBalances::<T>::insert((collection_id, &receiver), owner_balance);
        Tokens::<T>::insert(collection_id, start_idx, token);

        // [receiver, collection_id, start_idx, end_idx, new_total_supply]
        // Self::deposit_event(RawEvent::NonFungibleTokenMinted(
        //     collection_id,
        //     start_idx,
        //     end_idx,
        // ));

        Ok((start_idx, end_idx))
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

        T::Collection::add_total_supply(collection_id, amount)?;

        // let new_total_supply =
        //     <pallet_collection::Module<T>>::add_total_supply(collection_id, amount)?;

        AddressBalances::<T>::insert((collection_id, &receiver), owner_balance);

        Ok(())
    }

    fn _transfer_non_fungible(
        who: T::AccountId,
        receiver: T::AccountId,
        collection_id: T::Hash,
        start_idx: u128,
        amount: u128,
    ) -> DispatchResult {
        ensure!(who != receiver, Error::<T>::ReceiverIsSender);
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
        ensure!(token.owner == who, Error::<T>::PermissionDenied);

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

        let is_transfer_all = receiver_token.end_idx == token.end_idx;

        AddressBalances::<T>::insert((collection_id, who), sender_balance);
        AddressBalances::<T>::insert((collection_id, receiver), receiver_balance);
        Tokens::<T>::insert(collection_id, start_idx, receiver_token);

        if !is_transfer_all {
            Tokens::<T>::insert(collection_id, sender_start_idx, token);
        }

        Ok(())
    }

    fn _transfer_fungible(
        who: T::AccountId,
        receiver: T::AccountId,
        collection_id: T::Hash,
        amount: u128,
    ) -> DispatchResult {
        ensure!(amount >= 1, Error::<T>::AmountLessThanOne);
        ensure!(who != receiver, Error::<T>::ReceiverIsSender);
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

        AddressBalances::<T>::insert((collection_id, who), sender_balance);
        AddressBalances::<T>::insert((collection_id, receiver), receiver_balance);

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

        ensure!(token.owner == who, Error::<T>::PermissionDenied);

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

        let burn_amount = Self::burned_tokens(collection_id)
            .checked_add(amount)
            .ok_or(Error::<T>::NumOverflow)?;
        let is_burn_all = new_start_idx == token.end_idx;

        T::Collection::sub_total_supply(collection_id, amount)?;

        AddressBalances::<T>::insert((collection_id, who), balance);
        Tokens::<T>::remove(collection_id, start_idx);
        BurnedTokens::<T>::insert(collection_id, burn_amount);

        if !is_burn_all {
            Tokens::<T>::insert(collection_id, new_start_idx, token);
        }

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

        T::Collection::sub_total_supply(collection_id, amount)?;

        AddressBalances::<T>::insert((collection_id, who), balance);
        BurnedTokens::<T>::insert(collection_id, burn_amount);

        Ok(())
    }
}
