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

#[derive(Encode, Decode, Clone, Default, Eq, PartialEq)]
pub struct SubTokenInfo<AccountId> {
    pub owner: AccountId,
    pub total_supply: u128,
    pub sub_token_type: Option<SubTokenType>,
}

const PALLET_ID: ModuleId = ModuleId(*b"SubToken");

pub trait Config: frame_system::Config + pallet_nft::Config + pallet_collection::Config {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
}

decl_storage! {
    trait Store for Module<T: Config> as SubModule {
        // subtoken_collection => creator
        pub SubTokenCreator get(fn token_owner): map hasher(blake2_128_concat) T::Hash => T::AccountId;
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
        // (token owner, collection_id, token_id, subtoken_collection)
        TokenReceived(AccountId, Hash, u128, Hash),
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
        pub fn receive(origin, collection_id: T::Hash, start_idx: u128) -> DispatchResult {
            let who = ensure_signed(origin.clone())?;
            let token = <pallet_nft::Tokens<T>>::get((collection_id, start_idx));
            <pallet_nft::Module<T>>::transfer(origin, Self::account_id(), collection_id, start_idx)?;
            let subtoken_collection_id = <pallet_collection::Module<T>>::_create_collection(Self::account_id(),token.uri)?;

            SubTokenCreator::<T>::insert(subtoken_collection_id, &who);
            SubTokens::<T>::insert(subtoken_collection_id, (collection_id, start_idx));

            // (token owner, collection_id, token_id, subtoken_collection)
            Self::deposit_event(RawEvent::TokenReceived(who, collection_id, start_idx, subtoken_collection_id));

            Ok(())
        }

    }
}

impl<T: Config> Module<T> {
    pub fn account_id() -> T::AccountId {
        PALLET_ID.into_account()
    }
}
