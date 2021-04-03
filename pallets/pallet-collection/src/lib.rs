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

#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq)]
pub enum TokenType {
    NonFungible,
    Fungible,
}

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
        pub Nonce get(fn get_nonce): u128;
        pub Collections get(fn collections): map hasher(blake2_128_concat) T::Hash => CollectionInfo<T::AccountId>;
    }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Config>::AccountId,
        Hash = <T as frame_system::Config>::Hash,
    {
        CollectionCreated(AccountId, Hash),
    }
);

decl_error! {
    pub enum Error for Module<T: Config> {
        NumOverflow,
    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

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
    fn collection_exist(collection_id: Hash) -> bool;
    fn get_collection(collection_id: Hash) -> CollectionInfo<AccountId>;
    fn generate_collection_id(nonce: u128) -> Result<Hash, DispatchError>;
    fn nonce_increment() -> Result<u128, DispatchError>;
    fn _create_collection(
        who: AccountId,
        uri: Vec<u8>,
        is_fungible: bool,
    ) -> Result<Hash, DispatchError>;
    fn destory_collection(collection_id: &Hash);
    fn add_total_supply(collection_id: Hash, amount: u128) -> Result<u128, DispatchError>;
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
