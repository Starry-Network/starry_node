#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
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

pub trait Config: frame_system::Config {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
    type Collection: CollectionInterface<Self::Hash, Self::AccountId>;
    type NFT: NFTInterface<Self::Hash, Self::AccountId>;
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
        Hash = <T as frame_system::Config>::Hash,
    {
        // (subtoken_collection)
        SubCollectionCreated(Hash),
        // (collection_id, token_id)
        TokenRecovered(Hash, u128),
        // (sub_collection, start_idx, end_idx)
        SubNonFungibleTokenMinted(Hash, u128, u128),
        // sub_collection
        SubFungibleTokenMinted(Hash),
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
                ensure!(&Self::sub_token_creator(sub_token_collection_id) == &who, Error::<T>::PermissionDenied);
            }

            let (collection_id, start_idx) = Self::sub_tokens(sub_token_collection_id);
            // <pallet_nft::Module<T>>::transfer_non_fungible(frame_system::RawOrigin::Signed(Self::account_id()).into(), who.clone(), collection_id, start_idx, 1)?;
            T::NFT::_transfer_non_fungible(Self::account_id(), who.clone(), collection_id, start_idx, 1)?;

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
    pub fn account_id() -> T::AccountId {
        PALLET_ID.into_account()
    }
}
