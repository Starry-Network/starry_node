// submit a proposal
// dao members can sponsor submited proposal, then the proposal will in quee
// vote, after vote period, members can's vote
// note: in vote period,can set member a "highestIndexYesVote", then they cannot ragequit until highest index proposal member voted YES on is processed
// grace, in this period member's who vote No can rageguit dao
// processing:  DAO protects members from extreme dilution: if the combo of a proposal and related Ragequitting would result in any one member suffering dilution of greater than 3x (dilution_bound), the proposal automatically fails.
// complete
#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::traits::{
    Currency,
    ExistenceRequirement::{AllowDeath, KeepAlive},
};

use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::{DispatchError, DispatchResult},
    ensure,
    traits::{Get, Randomness},
    Parameter,
};
use sp_runtime::traits::{CheckedDiv, CheckedMul, CheckedSub, Zero};

use frame_system::{self as system, ensure_signed};

use sp_runtime::{
    traits::{AccountIdConversion, BlakeTwo256, Dispatchable, Hash},
    ModuleId,
};

use sp_std::{cmp::max, convert::TryInto, vec::Vec};

use sp_core::TypeId;

use pallet_nft::NFTInterface;
use pallet_nft::TokenInfo;

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
pub struct DAOInfo<AccountId, BlockNumber, Balance> {
    pub account_id: AccountId,
    pub escrow_id: AccountId,
    pub name: Vec<u8>,
    pub period_duration: u128,
    pub voting_period: u128,
    pub grace_period: u128,
    pub metadata: Vec<u8>,
    pub total_shares: u128,
    pub summoning_time: BlockNumber,
    pub dilution_bound: u128, // maximum multiplier a YES voter will be obligated to pay in case of mass ragequit (default = 3)
    pub proposal_deposit: Balance,
    pub processing_reward: Balance,
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
pub struct Proposal<AccountId, Balance, Hash> {
    pub applicant: AccountId,
    pub proposer: AccountId,
    pub sponsor: Option<AccountId>,
    pub shares_requested: u128,
    pub tribute_offered: Balance,
    pub tribute_nft: Option<(Hash, u128)>,
    pub starting_period: u128,
    // aye, nay
    pub yes_votes: u128,
    pub no_votes: u128,
    pub details: Vec<u8>,
    pub action: Option<Vec<u8>>,
    // pub status: Option<ProposalStatus>,
    pub sponsored: bool,
    pub processed: bool,
    pub did_pass: bool,
    pub cancelled: bool,
    pub executed: bool,
    pub max_total_shares_at_yes_vote: u128,
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

decl_storage! {
    trait Store for Module<T: Config> as NFTDAOModule {
        Nonce get(fn get_nonce): u128;
        // dao account => dao info
        pub DAOs get(fn dao): map hasher(blake2_128_concat)  T::AccountId => DAOInfo<T::AccountId, T::BlockNumber, BalanceOf<T>>;
        // (dao account, member account) => member
        pub Members get(fn member): double_map hasher(blake2_128_concat) T::AccountId, hasher(blake2_128_concat) T::AccountId => Member;
        // dao account => dao escrows account
        pub Escrows get(fn escrow): map hasher(blake2_128_concat)  T::AccountId => T::AccountId;
        // user currency balance: dao account , user account => balance
        pub UserCurrencyBalance get(fn user_currency_balance): double_map hasher(blake2_128_concat) T::AccountId, hasher(blake2_128_concat) T::AccountId => BalanceOf<T>;
        // user nft: (dao account, user account), collection  => (token id)
        // pub UserNFT get(fn user_nft): double_map hasher(blake2_128_concat) (T::AccountId, T::AccountId), hasher(blake2_128_concat) (T::Hash, u128)=> ();
        // dao account => proposal id
        pub LastProposalId get(fn last_proposal_id): map hasher(blake2_128_concat) T::AccountId  => Option<u128>;
        // (dao account, proposal id) => proposal
        pub Proposals get(fn proposal): double_map hasher(blake2_128_concat) T::AccountId, hasher(blake2_128_concat) u128 => Option<Proposal<T::AccountId, BalanceOf<T>, T::Hash>>;
        //  dao account => proposal in queue index
        pub LastQueueIndex get(fn last_queue_index): map hasher(blake2_128_concat)  T::AccountId => Option<u128>;
        // (dao account, proposal queue index) => proposal id
        pub ProposalQueues get(fn proposal_queue): double_map hasher(blake2_128_concat) T::AccountId, hasher(blake2_128_concat) u128 => u128;
        // (dao account, proposal queue index), member account => ()
        pub VoteMembers get(fn vote_member): double_map hasher(blake2_128_concat) (T::AccountId, u128), hasher(blake2_128_concat) T::AccountId => ();

    }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Config>::AccountId,
    {
        // summoner_account, dao_account
        DAOCreated(AccountId, AccountId),
    }
);

decl_error! {
    pub enum Error for Module<T: Config> {
        PermissionDenied,
        DecodeFailed,
        NumOverflow,
        ConvertFailed,
        DAONotFound,
        NFTIsEmpty,
        DepositSmallerThanReward,
        ValueShouldLargeThanZero,
        NotDAOMember,
        ProposalNotFound,
        InsufficientBalances,
        InsufficientShares,
        SponsoredProposal,
        CancelledProposal,
        CanNotSponsorProposal,
        ExpiredPeriod,
        MemberAlreadyVoted,
        NotReadyToProcessed,
        ProcessedProposal,
        NoneStatus,
        PrevProposalUnprocessed,
        CanNotRagequit,
    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        // Errors must be initialized if they are used by the pallet.
        type Error = Error<T>;

        // Events must be initialized if they are used by the pallet.
        fn deposit_event() = default;

        #[weight = 10_000 ]
        pub fn create_dao(origin, name: Vec<u8>, period_duration: u128, voting_period: u128, grace_period: u128, metadata: Vec<u8>, shares_requested: u128, proposal_deposit: BalanceOf<T>, processing_reward: BalanceOf<T>, dilution_bound: u128 ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(proposal_deposit >= processing_reward, Error::<T>::DepositSmallerThanReward);
            ensure!(period_duration > Zero::zero(), Error::<T>::ValueShouldLargeThanZero);
            ensure!(voting_period > Zero::zero(), Error::<T>::ValueShouldLargeThanZero);
            ensure!(grace_period > Zero::zero(), Error::<T>::ValueShouldLargeThanZero);
            ensure!(dilution_bound > Zero::zero(), Error::<T>::ValueShouldLargeThanZero);

            let dao_id = Self::dao_id(&who, &name)?;
            let dao_account = Self::dao_account_id(&dao_id);
            let escrow_id = Self::dao_escrow_id(&dao_id);

            let block_number = <system::Pallet<T>>::block_number();

            let dao = DAOInfo {
                account_id: dao_account.clone(),
                escrow_id: escrow_id.clone(),
                name,
                period_duration,
                voting_period,
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
            Members::<T>::insert(&dao_account, &who, member);

            Self::deposit_event(RawEvent::DAOCreated(who, dao_account));

            Ok(())
        }

        #[weight = 10_000]
        pub fn submit_proposal(origin, dao_account: T::AccountId, applicant: T::AccountId, shares_requested: u128, tribute_offered: BalanceOf<T>, tribute_nft: Option<(T::Hash, u128)>,  details: Vec<u8>, action: Option<Vec<u8>>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(
                DAOs::<T>::contains_key(&dao_account),
                Error::<T>::DAONotFound
            );

            let escrow_id = Self::escrow(&dao_account);

            // let starting_period: T::BlockNumber = Zero::zero();
            let starting_period: u128 = 0;

            let proposal = Proposal {
                applicant,
                proposer: who.clone(),
                sponsor: None::<T::AccountId>,
                shares_requested,
                tribute_offered,
                tribute_nft,
                starting_period,
                yes_votes:0,
                no_votes:0,
                details,
                action,
                // status: None::<ProposalStatus>,
                sponsored: false,
                processed: false,
                did_pass: false,
                cancelled: false,
                executed: false,
                max_total_shares_at_yes_vote: 0
            };

            if let Some((collection_id, token_id)) = tribute_nft {
                T::NFT::_transfer_non_fungible(who.clone(), escrow_id.clone(), collection_id, token_id, 1)?;
                // UserNFT::<T>::insert((&dao_account, &escrow_id), (collection_id, token_id), ());
            }

            if !(tribute_offered == Zero::zero()) {
                T::Currency::transfer(&who, &escrow_id, tribute_offered, AllowDeath)?;
                // UserCurrencyBalance::<T>::insert(&dao_account, &escrow_id, tribute_offered);
            }

            let proposal_id = Self::proposal_id_increment(&dao_account)?;

            LastProposalId::<T>::insert(&dao_account, &proposal_id);
            Proposals::<T>::insert(&dao_account, &proposal_id, proposal);

            // emit event

            Ok(())
        }

        #[weight = 10_000]
        pub fn cancel_proposal(origin, dao_account: T::AccountId, proposal_id: u128) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(DAOs::<T>::contains_key(&dao_account), Error::<T>::DAONotFound);

            if let Some(proposal) = Self::proposal(&dao_account, proposal_id) {
                ensure!(&who == &proposal.proposer, Error::<T>::PermissionDenied);

                ensure!(!&proposal.sponsored, Error::<T>::SponsoredProposal);
                ensure!(!&proposal.cancelled, Error::<T>::CancelledProposal);

                let proposal = Proposal{
                    cancelled: true,
                    ..proposal
                };

                let escrow_id = Self::escrow(&dao_account);

                Proposals::<T>::insert(&dao_account, &proposal_id, &proposal);

                if let Some((collection_id, token_id)) = proposal.clone().tribute_nft {
                    T::NFT::_transfer_non_fungible(escrow_id.clone(), who.clone(), collection_id, token_id, 1)?;
                    // UserNFT::<T>::remove((&dao_account, &escrow_id), (collection_id, token_id));
                }

                if !(&proposal.tribute_offered == &Zero::zero()) {
                    T::Currency::transfer(&escrow_id, &who, proposal.clone().tribute_offered, AllowDeath)?;
                }
                // emit event

            } else {
                Err(Error::<T>::ProposalNotFound)?
            }

            Ok(())

        }

        #[weight = 10_000]
        pub fn sponsor_proposal(origin, dao_account: T::AccountId, proposal_id: u128) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(DAOs::<T>::contains_key(&dao_account), Error::<T>::DAONotFound);
            ensure!(Members::<T>::contains_key(&dao_account, &who), Error::<T>::NotDAOMember);

            let dao = Self::dao(&dao_account);
            let escrow_id = Self::escrow(&dao_account);
            // let block_number = <system::Pallet<T>>::block_number();

            if let Some(proposal) = Self::proposal(&dao_account, proposal_id) {
                ensure!(!&proposal.sponsored, Error::<T>::SponsoredProposal);
                ensure!(!&proposal.cancelled, Error::<T>::CancelledProposal);

                let queue_index = Self::queue_index_increment(&dao_account)?;
                let starting_period = Self::calculate_starting_period(&dao)?;

                let proposal = Proposal {
                    sponsor: Some(who.clone()),
                    sponsored: true,
                    starting_period,
                    ..proposal
                };

                T::Currency::transfer(&who, &escrow_id, dao.clone().proposal_deposit, AllowDeath)?;

                Proposals::<T>::insert(&dao_account, &proposal_id, proposal);
                // UserCurrencyBalance::<T>::insert(&dao_account, &escrow_id, dao.proposal_deposit);
                ProposalQueues::<T>::insert(&dao_account, &queue_index, &proposal_id);
                LastQueueIndex::<T>::insert(&dao_account, &queue_index);

                // emit event

            } else {
                Err(Error::<T>::ProposalNotFound)?
            }

            Ok(())
        }

        #[weight = 10_000]
        pub fn vote_proposal(origin, dao_account: T::AccountId, proposal_index: u128, yes: bool) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(DAOs::<T>::contains_key(&dao_account), Error::<T>::DAONotFound);
            ensure!(ProposalQueues::<T>::contains_key(&dao_account, &proposal_index), Error::<T>::ProposalNotFound);
            ensure!(Members::<T>::contains_key(&dao_account, &who), Error::<T>::PermissionDenied);
            ensure!(!VoteMembers::<T>::contains_key((&dao_account, &proposal_index), &who), Error::<T>::MemberAlreadyVoted);

            let proposal_id = Self::proposal_queue(&dao_account, &proposal_index);

            if let Some(proposal) = Self::proposal(&dao_account, &proposal_id) {
                let dao = Self::dao(&dao_account);

                let current_period = Self::get_current_period(&dao)?;
                let starting_period = &proposal.starting_period;
                let voting_period = &dao.voting_period;
                let has_voting_period_expired =  current_period >= starting_period.checked_add(*voting_period).ok_or(Error::<T>::NumOverflow)?;

                ensure!(has_voting_period_expired, Error::<T>::ExpiredPeriod);

                let member = Self::member(&dao_account, &who);
                let member_shares = &member.shares;

                if yes {
                    let old_yes_votes = &proposal.yes_votes;
                    let yes_votes = Self::add_vote(*old_yes_votes, *member_shares)?;
                    let total_shares = &dao.total_shares;

                    let proposal = if total_shares > &proposal.max_total_shares_at_yes_vote {
                        Proposal {
                            yes_votes,
                            max_total_shares_at_yes_vote: *total_shares,
                            ..proposal
                        }
                    } else {
                        Proposal {
                            yes_votes,
                            ..proposal
                        }
                    };

                    let member = Member {
                        highest_index_yes_vote: proposal_index.clone(),
                        ..member
                    };

                    Members::<T>::insert(&dao_account, &who, member);
                    Proposals::<T>::insert(&dao_account, &proposal_id, proposal);

                } else {
                    let old_no_votes = &proposal.no_votes;
                    let no_votes = Self::add_vote(*old_no_votes, *member_shares)?;
                    let proposal = Proposal {
                        no_votes,
                        ..proposal
                    };
                    Proposals::<T>::insert(&dao_account, &proposal_id, proposal);
                }
                // emit event

            } else {
                Err(Error::<T>::ProposalNotFound)?
            }
            Ok(())
        }

        #[weight = 10_000]
        pub fn process_proposal(origin, dao_account: T::AccountId, proposal_index: u128) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(DAOs::<T>::contains_key(&dao_account), Error::<T>::DAONotFound);
            ensure!(ProposalQueues::<T>::contains_key(&dao_account, &proposal_index), Error::<T>::ProposalNotFound);

            let proposal_id = Self::proposal_queue(&dao_account, &proposal_index);

            if let Some(proposal) = Self::proposal(&dao_account, &proposal_id) {
                let dao = Self::dao(&dao_account);
                let escrow_id = Self::escrow(&dao_account);

                let current_period = Self::get_current_period(&dao)?;
                let starting_period = &proposal.starting_period;
                let voting_period = &dao.voting_period;
                let grace_period = &dao.grace_period;

                let passed_period = starting_period.checked_sub(voting_period).ok_or(Error::<T>::NumOverflow)?;
                let passed_period = passed_period.checked_sub(*grace_period).ok_or(Error::<T>::NumOverflow)?;

                ensure!(current_period >= passed_period, Error::<T>::NotReadyToProcessed);

                ensure!(!&proposal.processed, Error::<T>::ProcessedProposal);

                let prev_proposal_unprocessed = if &proposal_index == &Zero::zero() {
                    false
                } else {
                    let prev_index = &proposal_index.checked_sub(1).ok_or(Error::<T>::NumOverflow)?;
                    let prev_id = Self::proposal_queue(&dao.account_id, prev_index);
                    if let Some(prev_proposal) = Self::proposal(&dao.account_id, prev_id) {
                        prev_proposal.processed
                    } else {
                        Err(Error::<T>::ProposalNotFound)?
                    }
                };

                ensure!(!prev_proposal_unprocessed, Error::<T>::PrevProposalUnprocessed);

                let proposal = Proposal {
                    processed: true,
                    ..proposal
                };

                Proposals::<T>::insert(&dao_account, &proposal_id, &proposal);

                let dilution_bound = &dao.dilution_bound;
                let dilution = &dao.total_shares.checked_mul(*dilution_bound).ok_or(Error::<T>::NumOverflow)?;
                let did_pass = if &proposal.yes_votes > &proposal.no_votes {
                    dilution < &proposal.max_total_shares_at_yes_vote
                } else {
                    false
                };

                let tribute_nft = &proposal.tribute_nft;
                let tribute_offered = &proposal.tribute_offered;

                if did_pass {
                    let proposal = Proposal {
                        processed: true,
                        ..proposal.clone()
                    };
                    Proposals::<T>::insert(&dao_account, &proposal_id, &proposal);

                    let shares_requested = &proposal.shares_requested;
                    let member = if Members::<T>::contains_key(&dao_account, &proposal.applicant) {
                        let old_member = Self::member(&dao_account, &proposal.applicant);
                        let shares = old_member.shares.checked_add(*shares_requested).ok_or(Error::<T>::NumOverflow)?;
                        Member {
                            shares,
                            ..old_member
                        }
                    } else {
                        Member {
                            shares: *shares_requested,
                            highest_index_yes_vote: 0
                    }
                };

                if let Some((collection_id, token_id)) = *tribute_nft {
                    T::NFT::_transfer_non_fungible(escrow_id.clone(), dao_account.clone(), collection_id, token_id, 1)?;
                }

                if !(tribute_offered == &Zero::zero()) {
                    T::Currency::transfer(&escrow_id, &dao_account, *tribute_offered, AllowDeath)?;
                }

                Members::<T>::insert(&dao_account, &proposal.applicant, &member);

                if let Some(action_data) = &proposal.action {
                    let executed = Self::run(dao_account.clone(), action_data).is_ok();
                    let proposal = Proposal {
                        executed,
                        ..proposal.clone()
                    };
                    Proposals::<T>::insert(&dao_account, &proposal_id, &proposal);
                }


                } else {
                    // back tribute
                    if let Some((collection_id, token_id)) = *tribute_nft {
                        T::NFT::_transfer_non_fungible(escrow_id.clone(), proposal.clone().proposer, collection_id, token_id, 1)?;
                    }
                    if !(tribute_offered == &Zero::zero()) {
                        T::Currency::transfer(&escrow_id, &proposal.proposer, *tribute_offered, AllowDeath)?;
                    }
                }

                let proposal_deposit = &dao.proposal_deposit;
                let processing_reward = &dao.processing_reward;
                let back_to_sponsor = proposal_deposit.checked_sub(processing_reward).ok_or(Error::<T>::NumOverflow)?;

                T::Currency::transfer(&escrow_id, &who, *processing_reward, AllowDeath)?;
                T::Currency::transfer(&escrow_id, &proposal.proposer, back_to_sponsor, AllowDeath)?;

                // emit event
            }
            else {
                Err(Error::<T>::ProposalNotFound)?
            }

            Ok(())
        }

        #[weight = 10_000]
        pub fn ragequit(origin, dao_account: T::AccountId, shares_to_burn: u128) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(DAOs::<T>::contains_key(&dao_account), Error::<T>::DAONotFound);
            ensure!(Members::<T>::contains_key(&dao_account, &who), Error::<T>::PermissionDenied);

            let member = Self::member(&dao_account, &who);

            ensure!(&member.shares >= &shares_to_burn, Error::<T>::InsufficientShares);
            ensure!(ProposalQueues::<T>::contains_key(&dao_account, &member.highest_index_yes_vote), Error::<T>::ProposalNotFound);

            let proposal_id = Self::proposal_queue(&dao_account, &member.highest_index_yes_vote);

            if let Some(proposal) = Self::proposal(&dao_account, &proposal_id) {
                ensure!(proposal.processed, Error::<T>::CanNotRagequit);
            }

            let dao = Self::dao(&dao_account);

            let shares = &member.shares.checked_sub(shares_to_burn.clone()).ok_or(Error::<T>::NumOverflow)?;
            let total_shares = &dao.total_shares.checked_sub(shares_to_burn.clone()).ok_or(Error::<T>::NumOverflow)?;

            let member = Member {
                shares: *shares,
                ..member
            };

            let dao = DAOInfo {
                total_shares: *total_shares,
                ..dao
            };

            let dao_balance: BalanceOf<T> = T::Currency::free_balance(&dao_account);
            let total_shares_as_balance = Self::u128_to_balance(*total_shares)?;
            let shares_to_burn_as_balance = Self::u128_to_balance(shares_to_burn)?;

            let splited = &dao_balance.checked_div(&total_shares_as_balance).ok_or(Error::<T>::NumOverflow)?;
            let withdraw_balance = splited.checked_mul(&shares_to_burn_as_balance).ok_or(Error::<T>::NumOverflow)?;

            T::Currency::transfer(&Self::account_id(), &who, withdraw_balance, AllowDeath)?;

            Members::<T>::insert(&dao_account, &who, member);
            DAOs::<T>::insert(&dao_account, dao);

            // emit event

            Ok(())
        }
    }
}

impl<T: Config> Module<T> {
    pub fn account_id() -> T::AccountId {
        PALLET_ID.into_account()
    }

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

    pub fn proposal_id_increment(dao_account: &T::AccountId) -> Result<u128, DispatchError> {
        if let Some(proposal_id) = Self::last_proposal_id(dao_account) {
            let proposal_id = proposal_id.checked_add(1).ok_or(Error::<T>::NumOverflow)?;
            Ok(proposal_id)
        } else {
            Ok(0)
        }
    }

    pub fn queue_index_increment(dao_account: &T::AccountId) -> Result<u128, DispatchError> {
        if let Some(queue_index) = Self::last_queue_index(dao_account) {
            let queue_index = queue_index.checked_add(1).ok_or(Error::<T>::NumOverflow)?;
            Ok(queue_index)
        } else {
            Ok(0)
        }
    }

    pub fn blocknumber_to_u128(input: T::BlockNumber) -> Result<u128, DispatchError> {
        if let Some(blocknumber) = TryInto::<u128>::try_into(input).ok() {
            Ok(blocknumber)
        } else {
            Err(Error::<T>::ConvertFailed)?
        }
    }

    pub fn u128_to_balance(input: u128) -> Result<BalanceOf<T>, DispatchError> {
        if let Some(balance) = input.try_into().ok() {
            Ok(balance)
        } else {
            Err(Error::<T>::ConvertFailed)?
        }
    }

    pub fn get_current_period(
        dao: &DAOInfo<T::AccountId, T::BlockNumber, BalanceOf<T>>,
    ) -> Result<u128, DispatchError> {
        let summoning_time = &dao.summoning_time;
        let u128_summoning_time = Self::blocknumber_to_u128(*summoning_time)?;

        let now = Self::blocknumber_to_u128(<system::Pallet<T>>::block_number())?;
        let period = now
            .checked_sub(u128_summoning_time)
            .ok_or(Error::<T>::NumOverflow)?;

        let period_duration = &dao.period_duration;
        let period = period
            .checked_div(*period_duration)
            .ok_or(Error::<T>::NumOverflow)?;
        Ok(period)
    }

    pub fn calculate_starting_period(
        dao: &DAOInfo<T::AccountId, T::BlockNumber, BalanceOf<T>>,
    ) -> Result<u128, DispatchError> {
        let current_period = Self::get_current_period(&dao)?;
        let next_queue_index = Self::queue_index_increment(&dao.account_id)?;

        let period = if next_queue_index != 0 {
            let last_index = next_queue_index
                .checked_sub(1)
                .ok_or(Error::<T>::NumOverflow)?;
            let id = Self::proposal_queue(&dao.account_id, last_index);

            match Self::proposal(&dao.account_id, id) {
                None => Err(Error::<T>::ProposalNotFound)?,
                Some(proposal) => proposal.starting_period,
            }
        } else {
            0
        };

        Ok(max(current_period, period))
    }

    pub fn add_vote(now_vote: u128, shares: u128) -> Result<u128, DispatchError> {
        let added_vote = now_vote
            .checked_div(shares)
            .ok_or(Error::<T>::NumOverflow)?;
        Ok(added_vote)
    }

    pub fn run(dao_account: T::AccountId, action_data: &Vec<u8>) -> Result<bool, DispatchError> {
        if let Ok(action) = T::Action::decode(&mut &action_data[..]) {
            // Ok(action.dispatch(frame_system::RawOrigin::Root.into()).is_ok())
            let dao = frame_system::RawOrigin::Signed(dao_account).into();
            // Ok(action.dispatch_bypass_filter(seld_origin).is_ok())
            Ok(action.dispatch(dao).is_ok())
        } else {
            Err(Error::<T>::DecodeFailed)?
        }
    }
}
