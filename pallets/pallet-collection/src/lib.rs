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

#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct CollectionInfo<AccountId> {
    pub owner: AccountId,
    pub uri: Vec<u8>,
    pub total_supply: u128,
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
        pub fn create_collection(origin, uri: Vec<u8>) -> DispatchResult  {
            let who = ensure_signed(origin)?;

            let nonce = Nonce::try_mutate(|nonce| -> Result<u128, DispatchError> {
                *nonce = nonce.checked_add(1).ok_or(Error::<T>::NumOverflow)?;
                Ok(*nonce)
            })?;

            let collection_id = Self::generate_collection_id(nonce)?;

            let collection = CollectionInfo {
                owner: who.clone(),
                total_supply: 0,
                uri,
            };

            Collections::<T>::insert(collection_id, collection);

            Self::deposit_event(RawEvent::CollectionCreated(who, collection_id));

            Ok(())
        }
    }
}

impl<T: Config> Module<T> {
    pub fn generate_collection_id(nonce: u128) -> Result<T::Hash, DispatchError> {
        let seed = T::RandomnessSource::random_seed();
        let collection_id = T::Hashing::hash(&(PALLET_ID, seed, nonce).encode());

        Ok(collection_id)
    }

    // pub fn create_collection(who: T::AccountId, uri: Vec<u8>) -> Result<T::Hash, DispatchError> {
    //     let id = Self::generate_collection_id()?;

    //     let collection = CollectionInfo {
    //         owner: who,
    //         total_supply: 0,
    //         uri,
    //     };

    //     Collections::<T>::insert(id, collection);

    //     Ok(id)
    // }

    pub fn add_total_supply(collection_id: T::Hash, amount: u128) -> Result<u128, DispatchError> {
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

    pub fn sub_total_supply(collection_id: T::Hash, amount: u128) -> Result<u128, DispatchError> {
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
