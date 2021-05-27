//! # Sub Module
//!
//! - [`Config`]
//! - [`Call`]
//!
//! Create Sub tokens from NFT.
//!
//! ### Terminology
//!
//! * **Sub Token:** Lock NFT to this module then create new collection and tokens.
//! * **Recover:** Restore Sub Token to NFT.
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! * `create` - Transfer NFT to this module, and then create a new collection.
//! * `recover` - Transfer the locked NFT to the account that has all the sub tokens, and destroy the sub tokens.
//! * `mint_non_fungible` -  Mint one or a batch of SubNFTs 
//! * `mint_fungible` - Mint some SubFTs
//!
//! [`Call`]: ./enum.Call.html
//! [`Config`]: ./trait.Config.html

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure,
};
use frame_system::ensure_signed;
use sp_runtime::{traits::AccountIdConversion, ModuleId};
use sp_std::vec::Vec;

use pallet_collection::CollectionInterface;
use pallet_nft::NFTInterface;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

const PALLET_ID: ModuleId = ModuleId(*b"SubToken");

pub trait Config: frame_system::Config {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
    type Collection: CollectionInterface<Self::Hash, Self::AccountId>;
    type NFT: NFTInterface<Self::Hash, Self::AccountId>;
}

decl_storage! {
    trait Store for Module<T: Config> as SubNFTModule {
        /// The set of SubToken creators. subtoken_collection => creator
        pub SubTokenCreator get(fn sub_token_creator): map hasher(blake2_128_concat) T::Hash => T::AccountId;
        /// Record the collection_id of the SubToken corresponding to the locked NFT subtoken_collection => nft(collection_id, start_idx)
        pub SubTokens get(fn sub_tokens): map hasher(blake2_128_concat) T::Hash => (T::Hash, u128);
    }
}

decl_event!(
    /// Events for this module.
    pub enum Event<T>
    where
        Hash = <T as frame_system::Config>::Hash,
    {
        /// A SubCollection created. \[sub_collection_id\]
        SubCollectionCreated(Hash),

        /// Locked NFT was recovered. \[collection_id, token_id\]
        TokenRecovered(Hash, u128),

        /// One or a batch of SubNFTs were minted.  \[sub_collectio_idn, start_idx, end_idx\]
        SubNonFungibleTokenMinted(Hash, u128, u128),

        /// Some SubFTs were minted. \[sub_collection\]
        SubFungibleTokenMinted(Hash),
    }
);

decl_error! {
    /// Errors inform users that something went wrong.
    pub enum Error for Module<T: Config> {
        /// Collection does not exist.
        CollectionNotFound,
        /// SubToken does not exist.
        SubTokenNotFound,
        /// NFT does not exist.
        TokenNotFound,
        /// No permission to perform this operation.
        PermissionDenied,
        /// SubTokens cannot be burned at the time of recover
        BurnedtokensExistent,
    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;
        /// Lock NFT to this pallet and create a new collection.
        /// 
        /// The dispatch origin of this call must be _Signed_.
        /// 
        /// Parameters:
        /// - `collection_id`: The collection in which NFT is located.
        /// - `start_idx`: NFT's Index
        /// - `is_fungible`: SubToken is FT or not.
        #[weight = 10_000]
        pub fn create(origin, collection_id: T::Hash, start_idx: u128, is_fungible: bool) -> DispatchResult {
            let who = ensure_signed(origin)?;
            // transfer function will ensure collection and token exist so don't need to re-write ensure code.
            T::NFT::_transfer_non_fungible(who.clone(), Self::account_id(), collection_id, start_idx, 1)?;

            let token = T::NFT::get_nft_token(collection_id, start_idx);
            let sub_token_collection_id = T::Collection::_create_collection(Self::account_id(), token.uri, is_fungible)?;

            SubTokenCreator::<T>::insert(sub_token_collection_id, &who);
            SubTokens::<T>::insert(sub_token_collection_id, (collection_id, start_idx));

            // (token owner, collection_id, token_id, subtoken_collection, subtoken_type)
            Self::deposit_event(RawEvent::SubCollectionCreated(sub_token_collection_id));

            Ok(())
        }

        /// Burn all SubTokens and restore to NFT.
        /// 
        /// The dispatch origin of this call must be _Signed_.
        ///
        /// Parameters:
        /// - `sub_token_collection_id`: The collection where subtokens are located.
        #[weight = 10_000]
        pub fn recover(origin, sub_token_collection_id: T::Hash) -> DispatchResult {
            // when collection total_supply equals 0 and burn_amount equals 0, only creator can recover
            // if someone's balance is equal with subtoken collection total supply and burned amount equals 0, it can be recovered
            ensure!(
                T::Collection::collection_exist(sub_token_collection_id),
                Error::<T>::CollectionNotFound
            );
            ensure!(SubTokens::<T>::contains_key(sub_token_collection_id), Error::<T>::SubTokenNotFound);

            let who = ensure_signed(origin)?;
            let collection = T::Collection::get_collection(sub_token_collection_id);

            let balance = T::NFT::get_balance(&sub_token_collection_id, &who);
            ensure!(balance == collection.total_supply, Error::<T>::PermissionDenied);

            let burned_amount = T::NFT::get_burned_amount(&sub_token_collection_id);
            ensure!(burned_amount == 0, Error::<T>::BurnedtokensExistent);

            if collection.total_supply == 0 {
                ensure!(Self::sub_token_creator(sub_token_collection_id) == who, Error::<T>::PermissionDenied);
            }

            let (collection_id, start_idx) = Self::sub_tokens(sub_token_collection_id);
            // <pallet_nft::Module<T>>::transfer_non_fungible(frame_system::RawOrigin::Signed(Self::account_id()).into(), who.clone(), collection_id, start_idx, 1)?;
            T::NFT::_transfer_non_fungible(Self::account_id(), who, collection_id, start_idx, 1)?;

            SubTokenCreator::<T>::remove(sub_token_collection_id);
            SubTokens::<T>::remove(sub_token_collection_id);

            T::Collection::destory_collection(&collection_id);
            // <pallet_collection::Collections<T>>::remove(collection_id);

            if collection.total_supply != 0 {
                // <pallet_nft::Tokens<T>>::remove_prefix(sub_token_collection_id);
                T::NFT::destory_collection(&sub_token_collection_id);
            }

            // (collection_id, token_id)
            Self::deposit_event(RawEvent::TokenRecovered(collection_id, start_idx));

            Ok(())
        }

        /// Mint one or a batch of SubNFTs.
        ///
        /// The dispatch origin of this call must be _Signed_.
        /// 
        /// Parameters:
        /// - `receiver`: The address that accepts minted tokens.
        /// - `sub_token_collection_id`: The collection where the minted SubNFTs is located
        /// - `uri`: Uri representing the detailed information of SubNFT.
        /// - `amount`: How many tokens to mint.
        #[weight = 10_000]
        pub fn mint_non_fungible(origin, receiver: T::AccountId, sub_token_collection_id: T::Hash, uri: Vec<u8>,  amount: u128,) -> DispatchResult {
            ensure!(
                T::Collection::collection_exist(sub_token_collection_id),
                Error::<T>::CollectionNotFound
            );
            ensure!(SubTokens::<T>::contains_key(sub_token_collection_id), Error::<T>::SubTokenNotFound);


            let who = ensure_signed(origin)?;
            let collection = T::Collection::get_collection(sub_token_collection_id);
            ensure!(collection.owner == Self::account_id(), Error::<T>::PermissionDenied);
            ensure!(Self::sub_token_creator(sub_token_collection_id)==who, Error::<T>::PermissionDenied);

            let (start_idx, end_idx) = T::NFT::_mint_non_fungible(receiver, sub_token_collection_id, amount, uri, &collection)?;

            Self::deposit_event(RawEvent::SubNonFungibleTokenMinted(
                sub_token_collection_id,
                start_idx,
                end_idx,
            ));

            Ok(())
        }

        /// Mint some FTs
        ///
        /// The dispatch origin of this call must be _Signed_.
        /// 
        /// Parameters:
        /// - `receiver`: The address that accepts minted tokens.
        /// - `sub_token_collection_id`: The collection where the minted FTs is located
        /// - `amount`: How many tokens to mint.
        #[weight = 10_000]
        pub fn mint_fungible(origin, receiver: T::AccountId,  sub_token_collection_id: T::Hash, amount: u128,) -> DispatchResult {
            ensure!(
                T::Collection::collection_exist(sub_token_collection_id),
                Error::<T>::CollectionNotFound
            );
            ensure!(SubTokens::<T>::contains_key(sub_token_collection_id), Error::<T>::SubTokenNotFound);

            let who = ensure_signed(origin)?;
            let collection = T::Collection::get_collection(sub_token_collection_id);

            ensure!(collection.owner == Self::account_id(), Error::<T>::PermissionDenied);
            ensure!(Self::sub_token_creator(sub_token_collection_id)==who, Error::<T>::PermissionDenied);

            T::NFT::_mint_fungible(receiver, sub_token_collection_id, amount, &collection)?;

            Self::deposit_event(RawEvent::SubFungibleTokenMinted(sub_token_collection_id));

            Ok(())
        }
    }
}

impl<T: Config> Module<T> {
    /// Account of this pallet.
    pub fn account_id() -> T::AccountId {
        PALLET_ID.into_account()
    }
}
