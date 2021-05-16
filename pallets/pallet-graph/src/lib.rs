//! # Graph Module
//!
//! Combine different tokens together.
//!
//! ### Terminology
//!
//! * **Link:** Link a token to another token and the linked token must be NFT.
//! * **Parent NFT:** Linked NFT.
//! * **Child Token:** Link to the parent's token.
//! * **Ancestor NFT:** NFT located before parent NFT or parent NFT.
//! * **Root NFT:** Graph token's starting NFT.
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! * `link_non_fungible` - Link a NFT to another NFT.
//! * `link_fungible` - Link some FTs to NFT.
//! * `recover_non_fungible` - Transfer a child NFT to root_owner.
//! * `recover_fungible` - Transfer some child FTs to root_owner.
//! * `burn_fungible` - Destroy some FTs by owner
//! * `burn_non_fungible` - Destroy one or a batch of NFTs NFTs by owner
//!
//! [`Call`]: ./enum.Call.html
//! [`Config`]: ./trait.Config.html

#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "128"]
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::{DispatchError, DispatchResult},
    ensure,
};
use frame_system::ensure_signed;
use sp_runtime::{traits::AccountIdConversion, ModuleId};

use pallet_collection::CollectionInterface;
use pallet_nft::NFTInterface;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

const PALLET_ID: ModuleId = ModuleId(*b"GraphNFT");

// pub trait Config: frame_system::Config + pallet_collection::Config + pallet_nft::Config {
pub trait Config: frame_system::Config {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
    type Collection: CollectionInterface<Self::Hash, Self::AccountId>;
    type NFT: NFTInterface<Self::Hash, Self::AccountId>;
}

decl_storage! {
    trait Store for Module<T: Config> as GraphModule {
        /// The link relationship between child NFT and parent NFT. Child(collection_id, token_id) => Parent(collection_id, token_id)
        pub ChildToParent get(fn child_to_parent): map hasher(blake2_128_concat) (T::Hash, u128) => (T::Hash, u128);
        // The set of linked to parent NFT. [(Parent(collection_id, token_id), Child(collection_id, token_id))...]
        pub ParentToChild get(fn parent_to_child): double_map hasher(blake2_128_concat) (T::Hash, u128), hasher(blake2_128_concat) (T::Hash, u128) => ();
        // How many tokens are linked to the parent. (parent_token, child_collection_id) => balance
        pub ParentBalance get(fn parent_balance): double_map hasher(blake2_128_concat) (T::Hash, u128), hasher(blake2_128_concat) T::Hash => u128;
    }
}

decl_event!(
    /// Events for this module.
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Config>::AccountId,
    {
        /// NFT was linked to another NFT. \[who\]
        NonFungibleTokenLinked(AccountId),
        /// FTs were linked to NFT from user. \[who\]
        FungibleTokenLinkedByUser(AccountId),
        /// FTs were linked to NFT from child NFT. \[who\]
        FungibleTokenLinkedByChild(AccountId),
        /// Child NFT was transferred to root_owner. \[who\]
        NonFungibleTokenRecovered(AccountId),
        // Child FTs were transferred to root_owner. \[who\]
        FungibleTokenRecovered(AccountId),
    }
);

decl_error! {
    /// Errors inform users that something went wrong.
    pub enum Error for Module<T: Config> {
        /// No permission to perform this operation.
        PermissionDenied,
        /// Amount is more than own.
        AmountTooLarge,
        /// Number is too large or less than zero.
        NumOverflow,
        /// Collection does not exist.
        ParentCollectionNotFound,
        /// Token does not exist.
        TokenNotFound,
        /// The parent does not have this child FT balance.
        ChildHadNoBalance,
        /// Root NFT does not exist.
        RootTokenNotFound,
        /// Can't link ancestor NFT to descendant NFT.
        CanNotLinkAncestorToDescendant,
        /// Child token does not exist.
        ChildTokenNotFound,
        /// Can't recover a parent NFT.
        CanNotRecoverParentToken,
    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        /// Link a NFT to another NFT.
        ///
        /// Note: Ancestor NFT cannot be linked to descendant NFT.
        /// After linking to an NFT, the owner of the child NFT will become the root_owner
        ///
        /// The dispatch origin of this call must be _Signed_.
        ///
        /// Parameters:
        /// - `child_collection_id`: The collection in which child NFT is located.
        /// - `child_token_id`: The index of the child NFT.
        /// - `parent_collection_id`: The collection in which parebt NFT is located.
        /// - `parent_token_id`: The index of the parent NFT.
        #[weight = 10_000]
        pub fn link_non_fungible(origin, child_collection_id: T::Hash, child_token_id: u128, parent_collection_id: T::Hash, parent_token_id: u128) -> DispatchResult {
            ensure!(
                T::Collection::collection_exist(parent_collection_id),
                Error::<T>::ParentCollectionNotFound
            );
            ensure!(
                T::NFT::token_exist(parent_collection_id, parent_token_id),
                Error::<T>::TokenNotFound
            );

            let who = ensure_signed(origin)?;
            let have_parent = ChildToParent::<T>::contains_key((child_collection_id, child_token_id));

            if have_parent {
                // if token in ChildToParent, it's owner is graph pallet.
                let root_token_owner = Self::find_root_owner(child_collection_id, child_token_id)?;
                ensure!(&root_token_owner == &who, Error::<T>::PermissionDenied);
            } else {
                // token's owner should be user
                T::NFT::_transfer_non_fungible(who.clone(), Self::account_id(), child_collection_id, child_token_id, 1)?;
            }

            // if parent token's owner is user, it can be a root token, so don't check
            let parent_token = T::NFT::get_nft_token(parent_collection_id, parent_token_id);

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

            Self::deposit_event(RawEvent::NonFungibleTokenLinked(who));

            Ok(())
        }

        /// Link a FTs to NFT.
        ///
        /// If there is no child_collection_id and child_token_id, then FTs will be transferred from the user.
        ///
        /// The dispatch origin of this call must be _Signed_.
        ///
        /// Parameters:
        /// - `child_collection_id`: The collection in which child NFT is located.
        /// - `child_token_id`: The index of the child NFT.
        /// - `fungible_collection_id`: The collection in which FTs were located.
        /// - `parent_collection_id`: The collection in which parebt NFT is located.
        /// - `parent_token_id`: The index of the parent NFT.
        /// - `amount`: Amount of FTs to link to parent NFT.
        #[weight = 10_000]
        pub fn link_fungible(origin, child_collection_id: Option<T::Hash>, child_token_id: Option<u128>, fungible_collection_id: T::Hash, parent_collection_id: T::Hash, parent_token_id: u128, amount: u128) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(
               T::Collection::collection_exist(parent_collection_id),
                Error::<T>::ParentCollectionNotFound
            );
            ensure!(
                T::NFT::token_exist(parent_collection_id, parent_token_id),
                Error::<T>::TokenNotFound
            );

            let transfer_from_user = child_collection_id.is_none() || child_token_id.is_none();
            // let transfer_from_user = child_token.is_none();
            let parent_balance = Self::parent_balance((parent_collection_id, parent_token_id), fungible_collection_id).checked_add(amount).ok_or(Error::<T>::NumOverflow)?;

            if transfer_from_user {
                // <pallet_nft::Module<T>>::transfer_fungible(origin, Self::account_id(), fungible_collection_id, amount)?;
                T::NFT::_transfer_fungible(who.clone(), Self::account_id(), fungible_collection_id, amount)?;
                ParentBalance::<T>::insert((parent_collection_id, parent_token_id), fungible_collection_id, parent_balance);

                Self::deposit_event(RawEvent::FungibleTokenLinkedByUser(who));
            }
            else {
                if let (Some(child_collection_id), Some(child_token_id)) = (child_collection_id, child_token_id) {
                // if let Some((child_collection_id, child_token_id)) = child_token {

                    ensure!(ParentBalance::<T>::contains_key((child_collection_id, child_token_id), fungible_collection_id), Error::<T>::ChildHadNoBalance);

                    let child_balance = Self::parent_balance((child_collection_id, child_token_id), fungible_collection_id);
                    ensure!(child_balance >= amount, Error::<T>::AmountTooLarge);
                    let child_balance = child_balance.checked_sub(amount).ok_or(Error::<T>::NumOverflow)?;

                    let root_token_owner = Self::find_root_owner(child_collection_id, child_token_id)?;
                    ensure!(&root_token_owner == &who, Error::<T>::PermissionDenied);

                    ParentBalance::<T>::insert((child_collection_id, child_token_id), fungible_collection_id, child_balance);
                    ParentBalance::<T>::insert((parent_collection_id, parent_token_id), fungible_collection_id, parent_balance);

                    if child_balance == 0 {
                        ParentBalance::<T>::remove((child_collection_id, child_token_id), fungible_collection_id);
                    }

                    Self::deposit_event(RawEvent::FungibleTokenLinkedByChild(who));
                }
            }

            Ok(())
        }

        /// Transfer the child NFT from graph token to root_owner.
        ///
        /// Only child token can be recovered.
        ///
        /// The dispatch origin of this call must be _Signed_.
        ///
        /// Parameters:
        /// - `collection_id`: The collection id of the child NFT to be recovered.
        /// - `token_id`: The index of the child NFT to be recovered.
        #[weight = 10_000]
        pub fn recover_non_fungible(origin, collection_id: T::Hash, token_id: u128) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(
                ChildToParent::<T>::contains_key((collection_id, token_id)),
                Error::<T>::ChildTokenNotFound
            );

            // only child token can be recovered
            let mut maybe_children = ParentToChild::<T>::iter_prefix_values((collection_id, token_id));
            ensure!(
                maybe_children.next().is_none(),
                Error::<T>::CanNotRecoverParentToken
            );

            let root_token_owner = Self::find_root_owner(collection_id, token_id)?;

            ensure!(&root_token_owner == &who, Error::<T>::PermissionDenied);

            // <pallet_nft::Module<T>>::transfer_non_fungible(frame_system::RawOrigin::Signed(Self::account_id()).into(), who.clone(), collection_id, token_id, 1)?;
            T::NFT::_transfer_non_fungible(Self::account_id(), who.clone(), collection_id, token_id, 1)?;

            ChildToParent::<T>::remove((collection_id, token_id));

            Self::deposit_event(RawEvent::NonFungibleTokenRecovered(who));

            Ok(())
        }

        /// Transfer the child FTs from graph token to root_owner.
        ///
        /// The dispatch origin of this call must be _Signed_.
        ///
        /// Parameters:
        /// - `child_collection_id`: The collection_id of the child NFT where the FTs are located.
        /// - `child_token_id`: The index of the child NFT where the FTs are located.
        /// - `fungible_collection_id`: FT's collection_id.
        /// - `amount`: Amount to be recovered.
        #[weight = 10_000]
        pub fn recover_fungible(origin, child_collection_id: T::Hash, child_token_id: u128, fungible_collection_id: T::Hash, amount: u128) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let child_balance = Self::parent_balance((child_collection_id, child_token_id), fungible_collection_id);
            ensure!(child_balance >= amount, Error::<T>::AmountTooLarge);

            let root_token_owner = Self::find_root_owner(child_collection_id, child_token_id)?;
            ensure!(&root_token_owner == &who, Error::<T>::PermissionDenied);

            let child_balance = child_balance.checked_sub(amount).ok_or(Error::<T>::NumOverflow)?;

            // <pallet_nft::Module<T>>::transfer_fungible(frame_system::RawOrigin::Signed(Self::account_id()).into(), who.clone(), fungible_collection_id, amount)?;
            T::NFT::_transfer_fungible(Self::account_id(), who.clone(), fungible_collection_id, amount)?;
            ParentBalance::<T>::insert((child_collection_id, child_token_id), fungible_collection_id, child_balance);

            if child_balance == 0 {
                ParentBalance::<T>::remove((child_collection_id, child_token_id), fungible_collection_id);
            }

            Self::deposit_event(RawEvent::FungibleTokenRecovered(who));

            Ok(())
        }

    }
}

impl<T: Config> Module<T> {
    /// Account of this pallet.
    pub fn account_id() -> T::AccountId {
        PALLET_ID.into_account()
    }
    /// Find the root owner of this NFT.
    fn find_root_owner(
        child_collection_id: T::Hash,
        child_token_id: u128,
    ) -> Result<T::AccountId, DispatchError> {
        // root token: owner isn't equal with pallet account
        // if can't find parent token in pallet_nft, it may be burned.
        let token = T::NFT::get_nft_token(child_collection_id, child_token_id);
        if token.owner != Self::account_id() {
            Ok(token.owner)
        } else {
            let (parent_collection_id, parent_token_id) =
                Self::child_to_parent((child_collection_id, child_token_id));
            ensure!(
                T::NFT::token_exist(parent_collection_id, parent_token_id),
                Error::<T>::RootTokenNotFound
            );
            Self::find_root_owner(parent_collection_id, parent_token_id)
        }
    }

    /// Recursively check whether this NFT is an ancestor NFT.
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
                T::NFT::token_exist(parent_collection_id, parent_token_id),
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
