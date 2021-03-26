#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "128"]
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure,
    traits::Get,
};
use frame_system::ensure_signed;
use sp_runtime::{traits::AccountIdConversion, ModuleId};

use pallet_collection;
use pallet_nft;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

const PALLET_ID: ModuleId = ModuleId(*b"GraphNFT");

pub trait Config: frame_system::Config + pallet_collection::Config + pallet_nft::Config {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
}

decl_storage! {
    trait Store for Module<T: Config> as GraphModule {
        GraphCreator get(fn graph_creator): map hasher(blake2_128_concat)  T::Hash => T::AccountId;
    }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Config>::AccountId,
        Hash = <T as frame_system::Config>::Hash,
    {
        // (creator, graph_id)
        GraphCreated(AccountId, Hash),
    }
);

decl_error! {
    pub enum Error for Module<T: Config> {
        NoneValue,
        StorageOverflow,
        GraphNotFound
    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        #[weight = 10_000]
        pub fn create_graph(origin, uri: Vec<u8>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let graph_id = <pallet_collection::Module<T>>::_create_collection(Self::account_id(), uri, false)?;

            GraphCreator::<T>::insert(graph_id, &who);

            Self::deposit_event(RawEvent::GraphCreated(who, graph_id));

            Ok(())
        }

        #[weight = 10_000]
        pub fn mint(origin, receiver: T::AccountId, graph_id: T::Hash, uri: Vec<u8>) -> DispatchResult {
            ensure!(GraphCreator::<T>::contains_key, Error::<T>::GraphNotFound);

            let who = ensure_signed(origin)?;

            ensure!(&Self::graph_creator(graph_id), &who);

            let collection = <pallet_collection::Collections<T>>::get(graph_id);
            <pallet_nft::Module<T>>::_mint_non_fungible(receiver, graph_id, 1, uri, &collection);

            Ok(())
        }

		// link to other token
    }
}

impl<T: Config> Module<T> {
    pub fn account_id() -> T::AccountId {
        PALLET_ID.into_account()
    }
}
