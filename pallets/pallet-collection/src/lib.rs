//! # Collection Module
//!
//! - [`Config`]
//! - [`Call`]
//!
//! Collection is used to represent a series of NonFungible or Fungible Tokens,
//! which can also be understood as a folder. It is one of the basic modules.
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! * `create_collection` - Create a collection to represent NFT/FT.
//!
//! [`Call`]: ./enum.Call.html
//! [`Config`]: ./trait.Config.html

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};

use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::{DispatchError, DispatchResult},
    traits::Randomness,
};
use frame_system::ensure_signed;
use sp_runtime::{traits::Hash, ModuleId};
use sp_std::vec::Vec;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// Used to indicate the type of tokens in the Collection.
#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq)]
pub enum TokenType {
    /// NFT type
    NonFungible,
    /// FT type
    Fungible,
}

/// Details of a collection.
#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct CollectionInfo<AccountId> {
    pub owner: AccountId,
    pub uri: Vec<u8>,
    pub total_supply: u128,
    pub token_type: Option<TokenType>,
}

const PALLET_ID: ModuleId = ModuleId(*b"Collecti");

pub trait Config: frame_system::Config {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
    type RandomnessSource: Randomness<Self::Hash>;
}

decl_storage! {
    trait Store for Module<T: Config> as TemplateModule {
        /// Increment number used to create collection_id.
        pub Nonce get(fn get_nonce): u128;
        /// The set of collection.
        pub Collections get(fn collections): map hasher(blake2_128_concat) T::Hash => CollectionInfo<T::AccountId>;
    }
}

decl_event!(
    /// Events for this module.
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Config>::AccountId,
        Hash = <T as frame_system::Config>::Hash,
    {
        /// A collection was created. \[who, collection_id\]
        CollectionCreated(AccountId, Hash),
    }
);

decl_error! {
    /// Errors for this module.
    pub enum Error for Module<T: Config> {
        /// Nonce is too large to cause overflow.
        NumOverflow,
    }
}

decl_module! {
    /// The module declaration.
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        // Used for handling module events.
        fn deposit_event() = default;

        /// Create a new collection.
        ///
        /// The dispatch origin of this call must be _Signed_.
        ///
        /// Parameters:
        /// - `uri`: Used to get the detailed information of the collection such as name, description, cover_image, which can be the CID of ipfs or a URL.
        /// - `is_fungible`: Is FT or not.
        #[weight = 10_000]
        pub fn create_collection(origin, uri: Vec<u8>, is_fungible: bool) -> DispatchResult  {
            let who = ensure_signed(origin)?;
            let collection_id = Self::_create_collection(who.clone(), uri, is_fungible)?;

            Self::deposit_event(RawEvent::CollectionCreated(who, collection_id));

            Ok(())
        }
    }
}

pub trait CollectionInterface<Hash, AccountId> {
    /// Check whether the collection exists by collection_id.
    fn collection_exist(collection_id: Hash) -> bool;
    /// Get a collection by collection_id.
    fn get_collection(collection_id: Hash) -> CollectionInfo<AccountId>;
    /// Generate collection_id through nonce.
    fn generate_collection_id(nonce: u128) -> Result<Hash, DispatchError>;
    /// nonce plus one.
    fn nonce_increment() -> Result<u128, DispatchError>;
    /// create a collection.
    fn _create_collection(
        who: AccountId,
        uri: Vec<u8>,
        is_fungible: bool,
    ) -> Result<Hash, DispatchError>;
    /// destory a collection by collection_id.
    fn destory_collection(collection_id: &Hash);
    /// Increase a certain amount of of collection total_supply by collection_id.
    fn add_total_supply(collection_id: Hash, amount: u128) -> Result<u128, DispatchError>;
    /// Reduce a certain amount of collection total_supply by collection_id.
    fn sub_total_supply(collection_id: Hash, amount: u128) -> Result<u128, DispatchError>;
}

impl<T: Config> CollectionInterface<T::Hash, T::AccountId> for Module<T> {
    fn collection_exist(collection_id: T::Hash) -> bool {
        Collections::<T>::contains_key(collection_id)
    }

    fn get_collection(collection_id: T::Hash) -> CollectionInfo<T::AccountId> {
        Self::collections(collection_id)
    }

    fn generate_collection_id(nonce: u128) -> Result<T::Hash, DispatchError> {
        let seed = T::RandomnessSource::random_seed();
        let collection_id = T::Hashing::hash(&(PALLET_ID, seed, nonce).encode());

        Ok(collection_id)
    }

    fn nonce_increment() -> Result<u128, DispatchError> {
        let nonce = Nonce::try_mutate(|nonce| -> Result<u128, DispatchError> {
            *nonce = nonce.checked_add(1).ok_or(Error::<T>::NumOverflow)?;
            Ok(*nonce)
        })?;

        Ok(nonce)
    }

    fn _create_collection(
        who: T::AccountId,
        uri: Vec<u8>,
        is_fungible: bool,
    ) -> Result<T::Hash, DispatchError> {
        let nonce = Self::nonce_increment()?;

        let collection_id = Self::generate_collection_id(nonce)?;

        let token_type = if is_fungible {
            Some(TokenType::Fungible)
        } else {
            Some(TokenType::NonFungible)
        };

        let collection = CollectionInfo {
            owner: who.clone(),
            total_supply: 0,
            uri,
            token_type,
        };

        Collections::<T>::insert(collection_id, collection);

        Ok(collection_id)
    }

    fn destory_collection(collection_id: &T::Hash) {
        Collections::<T>::remove(collection_id)
    }

    fn add_total_supply(collection_id: T::Hash, amount: u128) -> Result<u128, DispatchError> {
        let collection = Self::collections(collection_id);

        let total_supply = collection
            .total_supply
            .checked_add(amount)
            .ok_or(Error::<T>::NumOverflow)?;

        let new_collection = CollectionInfo {
            total_supply,
            ..collection
        };

        Collections::<T>::insert(collection_id, new_collection);

        Ok(total_supply)
    }

    fn sub_total_supply(collection_id: T::Hash, amount: u128) -> Result<u128, DispatchError> {
        let collection = Self::collections(collection_id);

        let total_supply = collection
            .total_supply
            .checked_sub(amount)
            .ok_or(Error::<T>::NumOverflow)?;

        let new_collection = CollectionInfo {
            total_supply,
            ..collection
        };

        Collections::<T>::insert(collection_id, new_collection);

        Ok(total_supply)
    }
}
