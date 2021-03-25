#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure,
};
use frame_system::ensure_signed;
use sp_runtime::{traits::AccountIdConversion, ModuleId};
use sp_std::vec::Vec;

use pallet_collection;
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

const PALLET_ID: ModuleId = ModuleId(*b"SubToken");

pub trait Config: frame_system::Config + pallet_nft::Config + pallet_collection::Config {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
}

decl_storage! {
    trait Store for Module<T: Config> as SubModule {
        // subtoken_collection => creator
        pub SubTokenCreator get(fn sub_token_creator): map hasher(blake2_128_concat) T::Hash => T::AccountId;
        // subtoken_collection => nft(collection_id, start_idx)
        pub SubTokens get(fn sub_tokens): map hasher(blake2_128_concat) T::Hash => (T::Hash, u128);
    }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Config>::AccountId,
        Hash = <T as frame_system::Config>::Hash,
    {
        // (token owner, collection_id, token_id, subtoken_collection, subtoken_type)
        SubTokenCreated(AccountId, Hash, u128, Hash, bool),
    }
);

decl_error! {
    pub enum Error for Module<T: Config> {
        CollectionNotFound,
        SubTokenNotFound,
        TokenNotFound,
        PermissionDenied,
        AmountLessThanOne,
        WrongTokenType,
        NumOverflow,
        UriIsNone,
        AmountTooLarge,
        BurnedAmountShouldBeZero,
        BalanceInsufficient,
        BurnedtokensExistent,
    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        #[weight = 10_000]
        pub fn create(origin, collection_id: T::Hash, start_idx: u128, is_fungible: bool) -> DispatchResult {
            let who = ensure_signed(origin.clone())?;
            let token = <pallet_nft::Tokens<T>>::get((collection_id, start_idx));
            <pallet_nft::Module<T>>::_transfer_non_fungible(who.clone(), Self::account_id(), collection_id, start_idx, 1)?;
            let sub_token_collection_id = <pallet_collection::Module<T>>::_create_collection(Self::account_id(), token.uri, is_fungible)?;

            SubTokenCreator::<T>::insert(sub_token_collection_id, &who);
            SubTokens::<T>::insert(sub_token_collection_id, (collection_id, start_idx));

            // (token owner, collection_id, token_id, subtoken_collection, subtoken_type)
            Self::deposit_event(RawEvent::SubTokenCreated(who, collection_id, start_idx, sub_token_collection_id, is_fungible));

            Ok(())
        }

        #[weight = 10_000]
        pub fn recover(origin, sub_token_collection_id: T::Hash) -> DispatchResult {
            // when collection total_supply equals 0 and burn_amount equals 0, only creator can recover
            // if someone's balance is equal with subtoken collection total supply and burned amount equals 0, it can be recovered
            ensure!(
                <pallet_collection::Collections<T>>::contains_key(sub_token_collection_id),
                Error::<T>::CollectionNotFound
            );
            ensure!(SubTokens::<T>::contains_key(sub_token_collection_id), Error::<T>::SubTokenNotFound);

            let who = ensure_signed(origin)?;
            let collection = <pallet_collection::Collections<T>>::get(sub_token_collection_id);
            
            let balance = <pallet_nft::AddressBalances<T>>::get((sub_token_collection_id, &who));
            ensure!(balance == collection.total_supply, Error::<T>::PermissionDenied);

            let burned_amount = <pallet_nft::BurnedTokens<T>>::get(sub_token_collection_id);
            ensure!(burned_amount == 0, Error::<T>::BurnedtokensExistent);

            if collection.total_supply == 0 {
                ensure!(&Self::sub_token_creator(sub_token_collection_id) == &who, Error::<T>::PermissionDenied);
            }

            let (collection_id, start_idx) = Self::sub_tokens(sub_token_collection_id);
            <pallet_nft::Module<T>>::transfer_non_fungible(frame_system::RawOrigin::Signed(Self::account_id()).into(), who, collection_id, start_idx, 1)?;
            SubTokenCreator::<T>::remove(sub_token_collection_id);
            SubTokens::<T>::remove(sub_token_collection_id);
            
            Ok(())
        }
    }
}

impl<T: Config> Module<T> {
    pub fn account_id() -> T::AccountId {
        PALLET_ID.into_account()
    }
}
