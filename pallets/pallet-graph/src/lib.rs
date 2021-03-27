#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "128"]
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::{DispatchError, DispatchResult},
    ensure,
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
// ToDo: 1. repalce ParentToChild to many to many 2. when link to other nft, rember remove it from parent to child
decl_storage! {
    trait Store for Module<T: Config> as GraphModule {
        // Child(collection_id, token_id) => Parent(collection_id, token_id)
        pub ChildToParent get(fn child_to_parent): map hasher(blake2_128_concat) (T::Hash, u128) => (T::Hash, u128);
        // Parent(collection_id, token_id) => Child(collection_id, token_id)
        // pub ParentToChild get(fn parent_to_child): map hasher(blake2_128_concat) (T::Hash, u128) => Vec<(T::Hash, u128)>;
        pub ParentToChild get(fn parent_to_child): double_map hasher(blake2_128_concat) (T::Hash, u128), hasher(blake2_128_concat) (T::Hash, u128) => ();

        // (parent_token, child_collection_id) => balance
        pub ParentBalance get(fn parent_balance): double_map hasher(blake2_128_concat) (T::Hash, u128), hasher(blake2_128_concat) T::Hash => u128;
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
        // (child_collection_id, child_token_id, parent_collection_id, parent_token_id)
        NonFungibleTokenLinked(Hash, u128, Hash, u128),
        // (who, fungible_collection, parent_collection_id, parent_token_id, amount)
        FungibleTokenLinkedByUser(AccountId, Hash, Hash, u128, u128),
        // (who, child_collection_id, child_token_id, fungible_collection, parent_collection_id, parent_token_id, amount)
        FungibleTokenLinkedByChild(AccountId, Hash, u128, Hash, Hash, u128, u128),
        // (who, collection_id, token_id)
        NonFungibleTokenRecovered(AccountId, Hash, u128),
        // (who, child_collection_id, child_token_id, fungible_collection, amount)
        FungibleTokenRecovered(AccountId, Hash, u128, Hash, u128),
    }
);

decl_error! {
    pub enum Error for Module<T: Config> {
        PermissionDenied,
        AmountTooLarge,
        NumOverflow,
        ParentCollectionNotFound,
        TokenNotFound,
        ChildHadNoBalance,
        RootTokenNotFound,
        CanNotLinkAncestorToDescendant,
        CanNotRecoverNormalToken,
        CanNotRecoverParentToken,
    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        // link to other token
        #[weight = 10_000]
        pub fn link_non_fungible(origin, child_collection_id: T::Hash, child_token_id: u128, parent_collection_id: T::Hash, parent_token_id: u128) -> DispatchResult {
            ensure!(
                <pallet_collection::Collections<T>>::contains_key(parent_collection_id),
                Error::<T>::ParentCollectionNotFound
            );
            ensure!(
                <pallet_nft::Tokens<T>>::contains_key(parent_collection_id, parent_token_id),
                Error::<T>::TokenNotFound
            );


            let who = ensure_signed(origin.clone())?;
            let have_parent = ChildToParent::<T>::contains_key((child_collection_id, child_token_id));

            if have_parent {
                // if token in ChildToParent, it's owner is graph pallet.
                let root_token_owner = Self::find_root_owner(child_collection_id, child_token_id)?;
                ensure!(&root_token_owner == &who, Error::<T>::PermissionDenied);
            } else {
                // token's owner should be user
                <pallet_nft::Module<T>>::transfer_non_fungible(origin, Self::account_id(), child_collection_id, child_token_id, 1)?;
            }

            // if parent token's owner is user, it can be a root token, so don't check
            let parent_token = <pallet_nft::Module<T>>::tokens(parent_collection_id, parent_token_id);

            if parent_token.owner == Self::account_id() {
                let child_is_parent_ancestor = Self::is_ancestor((child_collection_id, child_token_id), (parent_collection_id, parent_token_id))?;
                ensure!(
                    !child_is_parent_ancestor,
                    Error::<T>::CanNotLinkAncestorToDescendant
                );
            }

            if have_parent {
                let (old_parent_collection_id, old_parent_token_id) = Self::child_to_parent((child_collection_id, child_token_id));
                ParentToChild::<T>::remove((old_parent_collection_id, old_parent_token_id), (child_collection_id, child_token_id));
            }

            ChildToParent::<T>::insert((child_collection_id, child_token_id), (parent_collection_id, parent_token_id));
            ParentToChild::<T>::insert((parent_collection_id, parent_token_id), (child_collection_id, child_token_id), ());

            Self::deposit_event(RawEvent::NonFungibleTokenLinked(
                child_collection_id,
                child_token_id,
                parent_collection_id,
                parent_token_id,
            ));

            Ok(())
        }

        #[weight = 10_000]
        // if use polkadot.js can use Option<(T::Hash,u128)> normal, will change to it
        // child_collection_id: Option<T::Hash>, child_token_id: Option<u128>
        // pub fn link_fungible(origin, child_token: Option<(T::Hash,u128)>, fungible_collection_id: T::Hash, parent_collection_id: T::Hash, parent_token_id: u128, amount: u128) -> DispatchResult {
        pub fn link_fungible(origin, child_collection_id: Option<T::Hash>, child_token_id: Option<u128>, fungible_collection_id: T::Hash, parent_collection_id: T::Hash, parent_token_id: u128, amount: u128) -> DispatchResult {

            let who = ensure_signed(origin.clone())?;

            ensure!(
                <pallet_collection::Collections<T>>::contains_key(parent_collection_id),
                Error::<T>::ParentCollectionNotFound
            );
            ensure!(
                <pallet_nft::Tokens<T>>::contains_key(parent_collection_id, parent_token_id),
                Error::<T>::TokenNotFound
            );

            let transfer_from_user = child_collection_id.is_none() || child_token_id.is_none();
            // let transfer_from_user = child_token.is_none();
            let parent_balance = Self::parent_balance((parent_collection_id, parent_token_id), fungible_collection_id).checked_add(amount).ok_or(Error::<T>::NumOverflow)?;

            if transfer_from_user {
                <pallet_nft::Module<T>>::transfer_fungible(origin, Self::account_id(), fungible_collection_id, amount)?;
                ParentBalance::<T>::insert((parent_collection_id, parent_token_id), fungible_collection_id, parent_balance);

                Self::deposit_event(RawEvent::FungibleTokenLinkedByUser(who,fungible_collection_id,parent_collection_id,parent_token_id,amount));
            }
            else {
                if let (Some(child_collection_id), Some(child_token_id)) = (child_collection_id, child_token_id) {
                // if let Some((child_collection_id, child_token_id)) = child_token {

                    ensure!(ParentBalance::<T>::contains_key((child_collection_id, child_token_id), fungible_collection_id), Error::<T>::ChildHadNoBalance);

                    let child_balance = Self::parent_balance((child_collection_id, child_token_id), fungible_collection_id);
                    ensure!(child_balance>=amount, Error::<T>::AmountTooLarge);
                    let child_balance = child_balance.checked_sub(amount).ok_or(Error::<T>::NumOverflow)?;

                    let root_token_owner = Self::find_root_owner(child_collection_id, child_token_id)?;
                    ensure!(&root_token_owner == &who, Error::<T>::PermissionDenied);

                    ParentBalance::<T>::insert((child_collection_id, child_token_id), fungible_collection_id, child_balance);
                    ParentBalance::<T>::insert((parent_collection_id, parent_token_id), fungible_collection_id, parent_balance);

                    if child_balance == 0 {
                        ParentBalance::<T>::remove((child_collection_id, child_token_id), fungible_collection_id);
                    }

                    Self::deposit_event(RawEvent::FungibleTokenLinkedByChild(who, child_collection_id, child_token_id,
                        fungible_collection_id,
                        parent_collection_id,
                        parent_token_id,
                        amount
                    ));
                }
            }

            Ok(())
        }

        #[weight = 10_000]
        pub fn recover_non_fungible(origin, collection_id: T::Hash, token_id: u128) -> DispatchResult {
            let who = ensure_signed(origin.clone())?;

            ensure!(
                ChildToParent::<T>::contains_key((collection_id, token_id)),
                Error::<T>::CanNotRecoverNormalToken
            );

            // only child token can be recovered
            let mut maybe_children = ParentToChild::<T>::iter_prefix_values((collection_id, token_id));
            ensure!(
                maybe_children.next().is_none(),
                Error::<T>::CanNotRecoverParentToken
            );

            let root_token_owner = Self::find_root_owner(collection_id, token_id)?;

            ensure!(&root_token_owner == &who, Error::<T>::PermissionDenied);

            <pallet_nft::Module<T>>::transfer_non_fungible(frame_system::RawOrigin::Signed(Self::account_id()).into(), who.clone(), collection_id, token_id, 1)?;

            ChildToParent::<T>::remove((collection_id, token_id));

            Self::deposit_event(RawEvent::NonFungibleTokenRecovered(who, collection_id, token_id));

            Ok(())
        }

        #[weight = 10_000]
        pub fn recover_fungible(origin, child_collection_id: T::Hash, child_token_id: u128, fungible_collection_id: T::Hash, amount: u128) -> DispatchResult {
            let who = ensure_signed(origin.clone())?;

            let child_balance = Self::parent_balance((child_collection_id, child_token_id), fungible_collection_id);
            ensure!(child_balance >= amount, Error::<T>::AmountTooLarge);

            let root_token_owner = Self::find_root_owner(child_collection_id, child_token_id)?;
            ensure!(&root_token_owner == &who, Error::<T>::PermissionDenied);

            let child_balance = child_balance.checked_sub(amount).ok_or(Error::<T>::NumOverflow)?;

            <pallet_nft::Module<T>>::transfer_fungible(frame_system::RawOrigin::Signed(Self::account_id()).into(), who.clone(), fungible_collection_id, amount)?;
            ParentBalance::<T>::insert((child_collection_id, child_token_id), fungible_collection_id, child_balance);

            if child_balance == 0 {
                ParentBalance::<T>::remove((child_collection_id, child_token_id), fungible_collection_id);
            }

            Self::deposit_event(RawEvent::FungibleTokenRecovered(who, child_collection_id, child_token_id, fungible_collection_id, amount));

            Ok(())
        }

    }
}

impl<T: Config> Module<T> {
    pub fn account_id() -> T::AccountId {
        PALLET_ID.into_account()
    }
    fn find_root_owner(
        child_collection_id: T::Hash,
        child_token_id: u128,
    ) -> Result<T::AccountId, DispatchError> {
        // root token: owner isn't equal with pallet account
        // if can't find parent token in pallet_nft, it may be burned.
        let token = <pallet_nft::Module<T>>::tokens(child_collection_id, child_token_id);
        if token.owner != Self::account_id() {
            Ok(token.owner)
        } else {
            let (parent_collection_id, parent_token_id) =
                Self::child_to_parent((child_collection_id, child_token_id));
            ensure!(
                <pallet_nft::Tokens<T>>::contains_key(parent_collection_id, parent_token_id),
                Error::<T>::RootTokenNotFound
            );
            Self::find_root_owner(parent_collection_id, parent_token_id)
        }
    }

    fn is_ancestor(
        maybe_ancestor_token: (T::Hash, u128),
        maybe_descendant_token: (T::Hash, u128),
    ) -> Result<bool, DispatchError> {
        let (ancestor_collection_id, ancestor_token_id) = maybe_ancestor_token;
        let (descendant_collection_id, descendant_token_id) = maybe_descendant_token;

        // if can't find descendant token's parent, it walks to the end and the token may be a root token.
        let have_parent =
            ChildToParent::<T>::contains_key((descendant_collection_id, descendant_token_id));

        if have_parent {
            let (parent_collection_id, parent_token_id) =
                Self::child_to_parent((descendant_collection_id, descendant_token_id));

            // if parent token not in pallet_nft, it may be burned.
            ensure!(
                <pallet_nft::Tokens<T>>::contains_key(parent_collection_id, parent_token_id),
                Error::<T>::RootTokenNotFound
            );

            // check whether token's parent equal with ancestor token
            let is_equal = parent_collection_id == ancestor_collection_id
                && parent_token_id == ancestor_token_id;

            if is_equal {
                Ok(true)
            } else {
                Self::is_ancestor(
                    (ancestor_collection_id, ancestor_token_id),
                    (parent_collection_id, parent_token_id),
                )
            }
        } else {
            Ok(false)
        }
    }
}
