#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::traits::{
    Currency,
    ExistenceRequirement::{AllowDeath, KeepAlive},
};
/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// https://substrate.dev/docs/en/knowledgebase/runtime/frame
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::{DispatchError, DispatchResult},
    ensure,
    traits::{Get, Randomness},
    Parameter,
};
use sp_runtime::traits::{Bounded, Zero};

use frame_system::{self as system, ensure_signed};

use sp_runtime::{
    traits::{AccountIdConversion, BlakeTwo256, Dispatchable, Hash},
    ModuleId,
};

use sp_std::{convert::TryInto, vec::Vec};

use sp_core::TypeId;

use pallet_nft::NFTInterface;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

const PALLET_ID: ModuleId = ModuleId(*b"NFTDAO!!");

type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[derive(Clone, Copy, Eq, PartialEq, Encode, Decode)]
pub struct DAOId(pub [u8; 32]);

impl TypeId for DAOId {
    const TYPE_ID: [u8; 4] = *b"dao!";
}

#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct DAOInfo<AccountId, BlockNumber, BalanceOf> {
    pub account_id: AccountId,
    pub escrow_id: AccountId,
    pub name: Vec<u8>,
    pub vote_period: BlockNumber,
    pub grace_period: BlockNumber,
    pub metadata: Vec<u8>,
    pub total_shares: u128,
    pub summoning_time: BlockNumber,
    pub dilution_bound: u128, // maximum multiplier a YES voter will be obligated to pay in case of mass ragequit (default = 3)
    pub proposal_deposit: BalanceOf,
    pub processing_reward: BalanceOf,
}

#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct Member {
    pub shares: u128,
    pub highest_index_yes_vote: u128,
}

#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq)]
pub enum ProposalStatus {
    Sponsored,
    Processed,
    DidPass,
    Cancelled,
}

#[derive(Encode, Decode, Default, Clone, PartialEq)]
struct Proposal<AccountId, BalanceOf, Hash, BlockNumber> {
    pub applicant: AccountId,
    pub proposer: AccountId,
    pub sponsor: Option<AccountId>,
    pub shares_requested: u128,
    pub tribute_offered: BalanceOf,
    pub tribute_nft: Option<(Hash, u128)>,
    pub tribute_nft_offered: u128,
    pub starting_period: BlockNumber,
    pub yes_votes: u128,
    pub no_votes: u128,
    pub details: Vec<u8>,
    pub status: Option<ProposalStatus>,
}

// #[derive(Encode, Decode, Copy, Clone, PartialEq, Eq)]
// pub enum TributeType {
//     NonFungible,
//     SemiFungible,
//     Fungible,
// }

/// Configure the pallet by specifying the parameters and types on which it depends.
pub trait Config: frame_system::Config {
    /// Because this pallet emits events, it depends on the runtime's definition of an event.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
    type Action: Parameter + Dispatchable<Origin = Self::Origin> + From<Call<Self>>;
    // type Action: Parameter + UnfilteredDispatchable<Origin=Self::Origin> + From<Call<Self>>;
    type RandomnessSource: Randomness<Self::Hash>;

    type Currency: Currency<Self::AccountId>;
    type NFT: NFTInterface<Self::Hash, Self::AccountId>;
}

// The pallet's runtime storage items.
// https://substrate.dev/docs/en/knowledgebase/runtime/storage
decl_storage! {
    // A unique name is used to ensure that the pallet's storage items are isolated.
    // This name may be updated, but each pallet in the runtime must use a unique name.
    // ---------------------------------vvvvvvvvvvvvvv
    trait Store for Module<T: Config> as NFTDAOModule {
        // Learn more about declaring storage items:
        // https://substrate.dev/docs/en/knowledgebase/runtime/storage#declaring-storage-items
        Something get(fn something): Option<u32>;
        Nonce get(fn get_nonce): u128;
        // dao account => dao info
        pub DAOs get(fn get_dao): map hasher(blake2_128_concat)  T::AccountId => DAOInfo<T::AccountId, T::BlockNumber, BalanceOf<T>>;
        // (dao account, member account) => member
        pub Members get(fn get_member): double_map hasher(blake2_128_concat) T::AccountId, hasher(blake2_128_concat) T::AccountId => Member;
        // dao account => dao escrows account
        pub Escrows get(fn get_escrow): map hasher(blake2_128_concat)  T::AccountId => T::AccountId;
        // dao account => proposal id
        pub LastProposalId get(fn last_proposal_id): map hasher(blake2_128_concat) T::AccountId  => Option<u128>;
        // (dao account, proposal id) => proposal
        // pub Proposals get(fn get_proposal): double_map hasher(blake2_128_concat) T::AccountId, hasher(blake2_128_concat) u128 => Proposal<T::AccountId, BalanceOf<T>, T::Hash, T::BlockNumber>;
    }
}

// Pallets use events to inform users when important changes are made.
// https://substrate.dev/docs/en/knowledgebase/runtime/events
decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Config>::AccountId,
    {
        /// Event documentation should end with an array that provides descriptive names for event
        /// parameters. [something, who]
        SomethingStored(u32, AccountId),
        // summoner_account, dao_account
        DAOCreated(AccountId, AccountId),
    }
);

// Errors inform users that something went wrong.
decl_error! {
    pub enum Error for Module<T: Config> {
        /// Error names should be descriptive.
        NoneValue,
        /// Errors should have helpful documentation associated with them.
        StorageOverflow,
        DecodeFailed,
        NumOverflow,
        ConvertFailed,
        DAONotFound,
        NFTIsEmpty,
    }
}

// Dispatchable functions allows users to interact with the pallet and invoke state changes.
// These functions materialize as "extrinsics", which are often compared to transactions.
// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        // Errors must be initialized if they are used by the pallet.
        type Error = Error<T>;

        // Events must be initialized if they are used by the pallet.
        fn deposit_event() = default;

        /// An example dispatchable that takes a singles value as a parameter, writes the value to
        /// storage and emits an event. This function must be dispatched by a signed extrinsic.
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn do_something(origin, something: u32) -> DispatchResult {
            // Check that the extrinsic was signed and get the signer.
            // This function will return an error if the extrinsic is not signed.
            // https://substrate.dev/docs/en/knowledgebase/runtime/origin
            let who = ensure_signed(origin)?;

            // Update storage.
            Something::put(something);

            // Emit an event.
            Self::deposit_event(RawEvent::SomethingStored(something, who));
            // Return a successful DispatchResult
            Ok(())
        }

        // submit a proposal
        // dao members can sponsor submited proposal, then the proposal will in quee
        // vote, after vote period, members can's vote
        // note: in vote period,can set member a "highestIndexYesVote", then they cannot ragequit until highest index proposal member voted YES on is processed
        // grace, in this period member's who vote No can rageguit dao
        // processing:  DAO protects members from extreme dilution: if the combo of a proposal and related Ragequitting would result in any one member suffering dilution of greater than 3x (dilution_bound), the proposal automatically fails.
        // complete
        #[weight = 10_000 ]
        pub fn create_dao(origin, name: Vec<u8>, vote_period: T::BlockNumber, grace_period: T::BlockNumber, metadata: Vec<u8>, shares_requested: u128, proposal_deposit: BalanceOf<T>, processing_reward: BalanceOf<T>, dilution_bound: u128 ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let dao_id = Self::dao_id(&who, &name)?;
            let dao_account = Self::dao_account_id(&dao_id);
            let escrow_id = Self::dao_escrow_id(&dao_id);

            let block_number = <system::Pallet<T>>::block_number();

            let dao = DAOInfo {
                account_id: dao_account.clone(),
                escrow_id: escrow_id.clone(),
                name,
                vote_period,
                grace_period,
                metadata,
                total_shares: shares_requested,
                summoning_time: block_number,
                dilution_bound,
                proposal_deposit,
                processing_reward
            };

            let member = Member {
                shares: shares_requested,
                highest_index_yes_vote: 0
            };

            DAOs::<T>::insert(&dao_account, dao);
            Escrows::<T>::insert(&dao_account, escrow_id);
            Members::<T>::insert(&dao_account, &who,  member);

            Self::deposit_event(RawEvent::DAOCreated(who, dao_account));

            Ok(())
        }

        #[weight = 10_000]
        pub fn submit_proposal(origin, dao_account: T::AccountId, applicant: T::AccountId, shares_requested: u128, tribute_offered: BalanceOf<T>, tribute_nft: Option<(T::Hash, u128)>, tribute_nft_offered: u128, details: Vec<u8>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(
                DAOs::<T>::contains_key(&dao_account),
                Error::<T>::DAONotFound
            );

            let escrow_id = Self::get_escrow(&dao_account);

            let proposal = Proposal {
                applicant,
                proposer: who.clone(),
                sponsor: None::<T::AccountId>,
                shares_requested,
                tribute_offered,
                tribute_nft,
                tribute_nft_offered,
                starting_period: 0,
                yes_votes:0,
                no_votes:0,
                details,
                status: None::<ProposalStatus>
            };


            let transfer_balance = tribute_offered == Zero::zero();
            let transfer_nft = tribute_nft_offered == Zero::zero();

            if !transfer_nft {
                if let Some((collection_id, token_id)) = tribute_nft {
                    T::NFT::_transfer_non_fungible(who.clone(), escrow_id.clone(), collection_id, token_id, tribute_nft_offered)?;
                } else {
                    Err(Error::<T>::NFTIsEmpty)?
                }
            }

            if !transfer_balance {
                T::Currency::transfer(&who, &escrow_id, tribute_offered, AllowDeath)?;
            }

            let proposal_id = Self::proposal_increment(&dao_account)?;

            Ok(())
        }


    }
}

impl<T: Config> Module<T> {
    pub fn account_id() -> T::AccountId {
        PALLET_ID.into_account()
    }

    // pub fn escrow_id(dao_id: DAOId) -> T::AccountId {
    //     dao_id.into_sub_account(b"escrow_id")
    // }

    // pub fn try_into_daoid(dao_account: &T::AccountId) -> Result<DAOId, DispatchError> {
    //     if let Some(id) = DAOId::try_from_account(dao_account) {
    //         Ok(id)
    //     } else {
    //         Err(Error::<T>::ConvertFailed)?
    //     }
    // }

    fn nonce_increment() -> Result<u128, DispatchError> {
        let nonce = Nonce::try_mutate(|nonce| -> Result<u128, DispatchError> {
            *nonce = nonce.checked_add(1).ok_or(Error::<T>::NumOverflow)?;
            Ok(*nonce)
        })?;

        Ok(nonce)
    }

    pub fn option_dao_account_id(
        summoner_address: &T::AccountId,
        name: &Vec<u8>,
    ) -> Result<T::AccountId, DispatchError> {
        let nonce = Self::nonce_increment()?;
        let seed = T::RandomnessSource::random_seed();

        let hash = T::Hashing::hash(&(name, seed).encode());
        let hash = T::Hashing::hash(&("awesome nft dao!", summoner_address, hash, nonce).encode());

        Ok(PALLET_ID.into_sub_account((hash).encode()))
    }

    pub fn dao_id(summoner_address: &T::AccountId, name: &Vec<u8>) -> Result<DAOId, DispatchError> {
        let nonce = Self::nonce_increment()?;
        let seed = T::RandomnessSource::random_seed();

        let hash = BlakeTwo256::hash(&(name, seed).encode());
        let hash = BlakeTwo256::hash(&("awesome nft dao!", summoner_address, hash, nonce).encode());

        let id: [u8; 32] = hash.into();

        Ok(DAOId(id))
    }

    pub fn dao_account_id(dao_id: &DAOId) -> T::AccountId {
        dao_id.into_account()
    }

    pub fn dao_escrow_id(dao_id: &DAOId) -> T::AccountId {
        dao_id.into_sub_account(b"escrow_id")
    }

    pub fn proposal_increment(dao_account: &T::AccountId) -> Result<u128, DispatchError> {
        if let Some(proposal_id) = Self::last_proposal_id(dao_account) {
            let proposal_id = proposal_id.checked_add(1).ok_or(Error::<T>::NumOverflow)?;
            Ok(proposal_id)
        } else {
            Ok(0)
        }
    }

    // pub fn dao_account_id(
    //     summoner_address: &T::AccountId,
    //     name: &Vec<u8>,
    // ) -> Result<T::AccountId, DispatchError> {
    //     let nonce = Self::nonce_increment()?;
    //     let seed = T::RandomnessSource::random_seed();

    //     let hash = BlakeTwo256::hash(&(name, seed).encode());
    //     let hash = BlakeTwo256::hash(&("awesome nft dao!", summoner_address, hash, nonce).encode());

    //     let id: [u8; 32] = hash.into();
    //     let dao_id: DAOId = DAOId(id);

    //     Ok(dao_id.into_account())
    // }

    pub fn run(data: Vec<u8>) -> Result<bool, DispatchError> {
        if let Ok(action) = T::Action::decode(&mut &data[..]) {
            // Ok(action.dispatch(frame_system::RawOrigin::Root.into()).is_ok())
            let self_origin = frame_system::RawOrigin::Signed(Self::account_id()).into();
            // Ok(action.dispatch_bypass_filter(seld_origin).is_ok())
            Ok(action.dispatch(self_origin).is_ok())
        } else {
            Err(Error::<T>::DecodeFailed)?
        }
    }
}
