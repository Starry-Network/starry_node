// submit a proposal
// dao members can sponsor submited proposal, then the proposal will in quee
// vote, after vote period, members can's vote
// note: in vote period,can set member a "highestIndexYesVote", then they cannot ragequit until highest index proposal member voted YES on is processed
// grace, in this period member's who vote No can rageguit dao
// processing:  DAO protects members from extreme dilution: if the combo of a proposal and related Ragequitting would result in any one member suffering dilution of greater than 3x (dilution_bound), the proposal automatically fails.
// complete

//! # NFTDAO Module
//!
//! A pallet that refers to Moloch and can use NFT as tribute.
//!
//! ### Terminology
//!
//! * **Pool:** It can be exchanged with some FTs, and the price can be automatically discovered through bancor curve.
//! * **Action:** After the proposal is passed, the operation of the dao account on the chain.
//! * **Proposal Queue:** Only proposals in the queue can be voted.
//! * **Sponsor:** In order to prevent spam proposals, a proposal must be sponsored to enter the queue.
//! * **Vote:** Yes or not, only members of dao can vote.
//! * **Grace:** You can ragequit after the voting period.
//! * **Ragequit:** Burn shares and exchange for corresponding assets.
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! * `create_dao` - Create a new DAO.
//! * `submit_proposal` - Submit a proposal, regardless of whether it is a member of dao can perform this operation.
//! * `cancel_proposal` - The proposal can be cancelled before it is sponsored.
//! * `sponsor_proposal` - Sponsor a proposal and make it into the queue.
//! * `vote_proposal` - DAO members can vote on proposals.
//! * `process_proposal` - After the grace period, the proposal needs to be processed.
//! * `ragequit` - Burn shares and exchange for corresponding assets..
//!
//! [`Call`]: ./enum.Call.html
//! [`Config`]: ./trait.Config.html

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::traits::{Currency, ExistenceRequirement::AllowDeath};

use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::{DispatchError, DispatchResult},
    ensure,
    traits::Randomness,
    Parameter,
};
use sp_runtime::traits::{CheckedDiv, CheckedMul, CheckedSub, SaturatedConversion, Zero};

use frame_system::{self as system, ensure_signed};

use sp_runtime::{
    traits::{AccountIdConversion, BlakeTwo256, Dispatchable, Hash},
    ModuleId,
};

use sp_std::{cmp::max, vec::Vec};

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

/// DAO's details
#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct DAOInfo<AccountId, BlockNumber, Balance> {
    pub account_id: AccountId,
    pub escrow_id: AccountId,
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

/// Member's details
#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct Member {
    pub shares: u128,
    pub highest_index_yes_vote: u128,
}

/// Proposal's details
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
    pub sponsored: bool,
    pub processed: bool,
    pub did_pass: bool,
    pub cancelled: bool,
    pub executed: bool,
    pub max_total_shares_at_yes_vote: u128,
}

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
        /// A self-increasing number used to create a dao account.
        Nonce get(fn get_nonce): u128;
        /// A set of daos. dao account => dao info
        pub DAOs get(fn dao): map hasher(blake2_128_concat)  T::AccountId => DAOInfo<T::AccountId, T::BlockNumber, BalanceOf<T>>;
        /// A set of members in the DAO. (dao account, member account) => member
        pub Members get(fn member): double_map hasher(blake2_128_concat) T::AccountId, hasher(blake2_128_concat) T::AccountId => Member;
        /// A set of dao's escrow account. dao account => dao escrows account
        pub Escrows get(fn escrow): map hasher(blake2_128_concat)  T::AccountId => T::AccountId;
        /// The last proposalId in each DAO. dao account => proposal id
        pub LastProposalId get(fn last_proposal_id): map hasher(blake2_128_concat) T::AccountId  => Option<u128>;
        /// A set of proposals. (dao account, proposal id) => proposal
        pub Proposals get(fn proposal): double_map hasher(blake2_128_concat) T::AccountId, hasher(blake2_128_concat) u128 => Option<Proposal<T::AccountId, BalanceOf<T>, T::Hash>>;
        /// The last proposal queue index in each DAO. dao account => proposal in queue index
        pub LastQueueIndex get(fn last_queue_index): map hasher(blake2_128_concat)  T::AccountId => Option<u128>;
        /// Proposal ids in the queue. (dao account, proposal queue index) => proposal id
        pub ProposalQueues get(fn proposal_queue): double_map hasher(blake2_128_concat) T::AccountId, hasher(blake2_128_concat) u128 => u128;
        /// A set of members who voted on the proposal (dao account, proposal queue index), member account => ()
        pub VoteMembers get(fn vote_member): double_map hasher(blake2_128_concat) (T::AccountId, u128), hasher(blake2_128_concat) T::AccountId => ();

    }
}

decl_event!(
    /// Events for this module.
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Config>::AccountId,
    {
        /// A DAO was created. \[summoner_account, dao_account, escrow_id\]
        DAOCreated(AccountId, AccountId, AccountId),
        /// A proposal submitted. \[proposal_id\]
        ProposalSubmitted(u128),
        /// A proposal cancelled. \[proposal_id\]
        ProposalCanceled(u128),
        /// A proposal was sponsored. \[queue_index, starting_period\]
        ProposalSponsored(u128, u128),
        /// A member has voted a proposal. \[proposal_id, member_shares(votes)\]
        ProposalVoted(u128, u128),
        /// The action in a proposal was executed. \[proposal_id, executed\]
        ProposalExecuted(u128, bool),
        /// A proposal was processed. \[proposal_id, did_pass\]
        ProposalProcessed(u128, bool),
        /// A member performed the ragequited operation \[burn_shares\]
        MemberRagequited(u128),
    }
);

decl_error! {
    /// Errors inform users that something went wrong.
    pub enum Error for Module<T: Config> {
        /// No permission to perform this operation.
        PermissionDenied,
        /// Have no enough shares.
        InsufficientShares,
        /// Decode action failed.
        DecodeFailed,
        /// Number is too large or less than zero.
        NumOverflow,
        /// DAO does not exist.
        DAONotFound,
        /// Deposit for proposal can't less than proposal reward.
        DepositSmallerThanReward,
        /// The minimum value of period duration is 1.
        PeriodDurationShouldLargeThanZero,
        /// The minimum value of voting duration is 1.
        VotingDurationShouldLargeThanZero,
        /// The minimum value of grace period is 1.
        GracePeriodShouldLargeThanZero,
        /// The minimum value of dilution_bound is 1.
        DilutionBoundShouldLargeThanZero,
        /// Account is not a member of dao.
        NotDAOMember,
        /// Proposal does not exist.
        ProposalNotFound,
        /// The proposal has already been sponsored.
        SponsoredProposal,
        /// The proposal has already been cancelled.
        CancelledProposal,
        /// Not the right time.
        ExpiredPeriod,
        /// A member can only vote once on the proposal.
        MemberAlreadyVoted,
        /// Not the right time to processe proposal.
        NotReadyToProcessed,
        /// The proposal has already been processed.
        ProcessedProposal,
        /// Old proposals should be disposed of before processing a proposal.
        PrevProposalUnprocessed,
        CanNotRagequit,
        /// The minimum value of BurnShares is 1.
        BurnSharesShouldLargeThanZero,
    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        // Errors must be initialized if they are used by the pallet.
        type Error = Error<T>;

        // Events must be initialized if they are used by the pallet.
        fn deposit_event() = default;

        /// Create a new DAO.
        ///
        /// The dispatch origin of this call must be _Signed_.
        ///
        /// Parameters:
        /// - `metadata`: The details of DAO, can be a ipfs CID.
        /// - `period_duration`: The duration of each period.
        /// - `voting_period`: How many periods does voting last.
        /// - `grace_period`: How many periods does grace last.
        /// - `shares_requested`: How many shares does the summoner need.
        /// - `shares_requested`: How many shares does the summoner need.
        /// - `proposal_deposit`: How many asset does the sponner need.
        /// - `processing_reward`: How much is the reward for processing the proposal.
        /// - `dilution_bound`: Value that protects members from extreme dilution, default is 3.
        #[weight = 10_000 ]
        pub fn create_dao(origin, metadata: Vec<u8>, period_duration: u128, voting_period: u128, grace_period: u128, shares_requested: u128, proposal_deposit: BalanceOf<T>, processing_reward: BalanceOf<T>, dilution_bound: u128 ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(proposal_deposit >= processing_reward, Error::<T>::DepositSmallerThanReward);
            ensure!(period_duration > Zero::zero(), Error::<T>::PeriodDurationShouldLargeThanZero);
            ensure!(voting_period > Zero::zero(), Error::<T>::VotingDurationShouldLargeThanZero);
            ensure!(grace_period > Zero::zero(), Error::<T>::GracePeriodShouldLargeThanZero);
            ensure!(dilution_bound > Zero::zero(), Error::<T>::DilutionBoundShouldLargeThanZero);

            let dao_id = Self::dao_id(&who, &metadata)?;
            let dao_account = Self::dao_account_id(&dao_id);
            let escrow_id = Self::dao_escrow_id(&dao_id);

            let block_number = <system::Pallet<T>>::block_number();

            let dao = DAOInfo {
                account_id: dao_account.clone(),
                escrow_id: escrow_id.clone(),
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
            Escrows::<T>::insert(&dao_account, &escrow_id);
            Members::<T>::insert(&dao_account, &who, member);

            Self::deposit_event(RawEvent::DAOCreated(who, dao_account, escrow_id));

            Ok(())
        }

        /// Submit a new proposal.
        ///
        /// The dispatch origin of this call must be _Signed_.
        ///
        /// Parameters:
        /// - `dao_account`: Account for submitting proposals to that dao.
        /// - `applicant`: Who will act after the proposal is passed.
        /// - `shares_requested`: Share to applicant after the proposal passed.
        /// - `tribute_offered`: Tribute to dao from the person who submitted the proposal.
        /// - `tribute_nft`: Tribute NFT to dao from the person who submitted the proposal.
        /// - `details`: The detailed information of the proposal, which can be ipfs cid with title and description.
        /// - `action`: Operations performed by DAO, such as transfers or purchases.
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
            }

            let proposal_id = Self::proposal_id_increment(&dao_account)?;

            LastProposalId::<T>::insert(&dao_account, &proposal_id);
            Proposals::<T>::insert(&dao_account, &proposal_id, proposal);

            // emit event
            Self::deposit_event(RawEvent::ProposalSubmitted(proposal_id));

            Ok(())
        }

        /// Cancel the proposal and return the tribute.
        ///
        /// The dispatch origin of this call must be _Signed_.
        ///
        /// Parameters:
        /// - `dao_account`: The account of the dao where the proposal is to be cancelled.
        /// - `proposal_id`: The id of proposal.
        #[weight = 10_000]
        pub fn cancel_proposal(origin, dao_account: T::AccountId, proposal_id: u128) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(DAOs::<T>::contains_key(&dao_account), Error::<T>::DAONotFound);

            if let Some(proposal) = Self::proposal(&dao_account, proposal_id) {
                ensure!(who == proposal.proposer, Error::<T>::PermissionDenied);

                ensure!(!&proposal.sponsored, Error::<T>::SponsoredProposal);
                ensure!(!&proposal.cancelled, Error::<T>::CancelledProposal);

                let proposal = Proposal{
                    cancelled: true,
                    ..proposal
                };

                let escrow_id = Self::escrow(&dao_account);

                Proposals::<T>::insert(&dao_account, &proposal_id, &proposal);

                if let Some((collection_id, token_id)) = proposal.tribute_nft {
                    T::NFT::_transfer_non_fungible(escrow_id.clone(), who.clone(), collection_id, token_id, 1)?;
                    // UserNFT::<T>::remove((&dao_account, &escrow_id), (collection_id, token_id));
                }

                if !(proposal.tribute_offered == Zero::zero()) {
                    T::Currency::transfer(&escrow_id, &who, proposal.tribute_offered, AllowDeath)?;
                }
                // emit event
                Self::deposit_event(RawEvent::ProposalCanceled(proposal_id));

            } else {
                return Err(Error::<T>::ProposalNotFound.into())
            }

            Ok(())

        }

        ///  Sponsor a proposal and make it into the queue.
        ///
        /// The dispatch origin of this call must be _Signed_.
        ///
        /// Parameters:
        /// - `dao_account`: The account of the dao where the proposal is to be sponsored.
        /// - `proposal_id`: The id of proposal.
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

                T::Currency::transfer(&who, &escrow_id, dao.proposal_deposit, AllowDeath)?;

                Proposals::<T>::insert(&dao_account, &proposal_id, proposal);
                ProposalQueues::<T>::insert(&dao_account, &queue_index, &proposal_id);
                LastQueueIndex::<T>::insert(&dao_account, &queue_index);

                // emit event
                Self::deposit_event(RawEvent::ProposalSponsored(queue_index, starting_period));

            } else {
                return Err(Error::<T>::ProposalNotFound.into());
            }

            Ok(())
        }

        ///  Vote for the proposal.
        ///
        /// The dispatch origin of this call must be _Signed_.
        ///
        /// Parameters:
        /// - `dao_account`: The account of the dao where the proposal is to be voted.
        /// - `proposal_index`: Proposal indexing in the queue.
        /// - `yes`: true is yes, false is not
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

                ensure!(!has_voting_period_expired, Error::<T>::ExpiredPeriod);

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
                        highest_index_yes_vote: proposal_index,
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

                VoteMembers::<T>::insert((&dao_account, &proposal_index), &who, ());
                // emit event
                Self::deposit_event(RawEvent::ProposalVoted(proposal_id, *member_shares));

            } else {
                return Err(Error::<T>::ProposalNotFound.into());
            }

            Ok(())
        }

        ///  Processing proposal.
        ///
        /// If the proposal fails, the tribute will be returned. After processing
        /// the proposal, part of the funds sponsored by the sponsor will be
        /// given to the processor as a reward, and part will be returned to the sponsor.
        ///
        /// If the proposal includes an action, ProposalExecuted event will be emitted after the action is executed.
        /// The ProposalProcessed event will be emitted after the proposal is processed.
        ///
        /// Note that, like a stack, the proposal that can be processed must be 
        /// the one at the beginning of the queue. If you do not process 
        /// the first one, you will get an error.
        ///
        /// The dispatch origin of this call must be _Signed_.
        ///
        /// Parameters:
        /// - `dao_account`: The account of the dao where the proposal is to be processed.
        /// - `proposal_index`: Proposal indexing in the queue.
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

                let passed_period = starting_period.checked_add(*voting_period).ok_or(Error::<T>::NumOverflow)?;
                let passed_period = passed_period.checked_add(*grace_period).ok_or(Error::<T>::NumOverflow)?;

                ensure!(current_period >= passed_period, Error::<T>::NotReadyToProcessed);

                ensure!(!&proposal.processed, Error::<T>::ProcessedProposal);

                let prev_proposal_unprocessed = if proposal_index == Zero::zero() {
                    false
                } else {
                    let prev_index = &proposal_index.checked_sub(1).ok_or(Error::<T>::NumOverflow)?;
                    let prev_id = Self::proposal_queue(&dao.account_id, prev_index);
                    if let Some(prev_proposal) = Self::proposal(&dao.account_id, prev_id) {
                        prev_proposal.processed
                    } else {
                        return Err(Error::<T>::ProposalNotFound.into())
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
                let did_pass = if proposal.yes_votes > proposal.no_votes {
                    dilution > &proposal.max_total_shares_at_yes_vote
                } else {
                    false
                };

                let tribute_nft = &proposal.tribute_nft;
                let tribute_offered = &proposal.tribute_offered;

                if did_pass {
                    let proposal = Proposal {
                        did_pass: true,
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

                let new_total_shares = &dao.total_shares.checked_add(*shares_requested).ok_or(Error::<T>::NumOverflow)?;

                let dao = DAOInfo {
                    total_shares: *new_total_shares,
                    ..dao
                };

                if let Some((collection_id, token_id)) = *tribute_nft {
                    T::NFT::_transfer_non_fungible(escrow_id.clone(), dao_account.clone(), collection_id, token_id, 1)?;
                }

                if !(tribute_offered == &Zero::zero()) {
                    T::Currency::transfer(&escrow_id, &dao_account, *tribute_offered, AllowDeath)?;
                }

                DAOs::<T>::insert(&dao_account, dao);
                Members::<T>::insert(&dao_account, &proposal.applicant, &member);

                if let Some(action_data) = &proposal.action {
                    let executed = Self::run(dao_account.clone(), action_data).is_ok();
                    let proposal = Proposal {
                        executed,
                        ..proposal.clone()
                    };
                    Proposals::<T>::insert(&dao_account, &proposal_id, &proposal);

                    Self::deposit_event(RawEvent::ProposalExecuted(proposal_id, executed));
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

                // emit event
                Self::deposit_event(RawEvent::ProposalProcessed(proposal_id, did_pass));

                let proposal_deposit = &dao.proposal_deposit;
                let processing_reward = &dao.processing_reward;
                let back_to_sponsor = proposal_deposit.checked_sub(processing_reward).ok_or(Error::<T>::NumOverflow)?;

                T::Currency::transfer(&escrow_id, &who, *processing_reward, AllowDeath)?;
                T::Currency::transfer(&escrow_id, &proposal.proposer, back_to_sponsor, AllowDeath)?;
            }
            else {
                return Err(Error::<T>::ProposalNotFound.into());
            }

            Ok(())
        }

        /// Members can burn shares and obtain corresponding assets.
        ///
        /// The dispatch origin of this call must be _Signed_.
        ///
        /// Parameters:
        /// - `dao_account`: The account of the dao where the proposal is to be ragequited.
        /// - `shares_to_burn`: How many shares to burn.
        #[weight = 10_000]
        pub fn ragequit(origin, dao_account: T::AccountId, shares_to_burn: u128) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(shares_to_burn > Zero::zero(), Error::<T>::BurnSharesShouldLargeThanZero);
            ensure!(DAOs::<T>::contains_key(&dao_account), Error::<T>::DAONotFound);
            ensure!(Members::<T>::contains_key(&dao_account, &who), Error::<T>::PermissionDenied);

            let member = Self::member(&dao_account, &who);

            ensure!(member.shares >= shares_to_burn, Error::<T>::InsufficientShares);
            ensure!(ProposalQueues::<T>::contains_key(&dao_account, &member.highest_index_yes_vote), Error::<T>::ProposalNotFound);

            let proposal_id = Self::proposal_queue(&dao_account, &member.highest_index_yes_vote);

            if let Some(proposal) = Self::proposal(&dao_account, &proposal_id) {
                ensure!(proposal.processed, Error::<T>::CanNotRagequit);
            }

            let dao = Self::dao(&dao_account);

            let shares = &member.shares.checked_sub(shares_to_burn).ok_or(Error::<T>::NumOverflow)?;
            let new_total_shares = &dao.total_shares.checked_sub(shares_to_burn).ok_or(Error::<T>::NumOverflow)?;
            let total_shares = &dao.total_shares;

            let member = Member {
                shares: *shares,
                ..member
            };

            let dao = DAOInfo {
                total_shares: *new_total_shares,
                ..dao
            };

            let dao_balance: BalanceOf<T> = T::Currency::free_balance(&dao_account);

            let total_shares_as_balance = (*total_shares).saturated_into::<BalanceOf<T>>();
            let shares_to_burn_as_balance = shares_to_burn.saturated_into::<BalanceOf<T>>();

            let splited = &dao_balance.checked_mul(&shares_to_burn_as_balance).ok_or(Error::<T>::NumOverflow)?;
            let withdraw_balance = splited.checked_div(&total_shares_as_balance).ok_or(Error::<T>::NumOverflow)?;

            T::Currency::transfer(&dao_account, &who, withdraw_balance, AllowDeath)?;

            Members::<T>::insert(&dao_account, &who, member);
            DAOs::<T>::insert(&dao_account, dao);

            // emit event
            Self::deposit_event(RawEvent::MemberRagequited(shares_to_burn));

            Ok(())
        }
    }
}

impl<T: Config> Module<T> {
    /// Account of this pallet.
    pub fn account_id() -> T::AccountId {
        PALLET_ID.into_account()
    }

    /// nonce plus one
    fn nonce_increment() -> Result<u128, DispatchError> {
        let nonce = Nonce::try_mutate(|nonce| -> Result<u128, DispatchError> {
            *nonce = nonce.checked_add(1).ok_or(Error::<T>::NumOverflow)?;
            Ok(*nonce)
        })?;

        Ok(nonce)
    }

    /// Hash related information.
    fn _dao_id(summoner_address: &T::AccountId, details: &[u8], nonce: u128) -> [u8; 32] {
        let seed = T::RandomnessSource::random_seed();

        let hash = BlakeTwo256::hash(&(details, seed).encode());
        let hash = BlakeTwo256::hash(&("awesome nft dao!", summoner_address, hash, nonce).encode());

        hash.into()
    }
    /// Convert the hash value to DAOId.
    pub fn dao_id(summoner_address: &T::AccountId, details: &[u8]) -> Result<DAOId, DispatchError> {
        let nonce = Self::nonce_increment()?;
        let id = Self::_dao_id(summoner_address, details, nonce);

        Ok(DAOId(id))
    }
    /// Convert the DAOId to dao account.
    pub fn dao_account_id(dao_id: &DAOId) -> T::AccountId {
        dao_id.into_account()
    }

    /// Use dao account to generate escrow id (a sub account).
    pub fn dao_escrow_id(dao_id: &DAOId) -> T::AccountId {
        // dao_id.into_sub_account(b"escrow_id")
        let hash = BlakeTwo256::hash(&("a escrow id", dao_id).encode());
        let id: [u8; 32] = hash.into();
        let escrow_id = DAOId(id);
        escrow_id.into_account()
    }

    /// id plus one
    pub fn proposal_id_increment(dao_account: &T::AccountId) -> Result<u128, DispatchError> {
        if let Some(proposal_id) = Self::last_proposal_id(dao_account) {
            let proposal_id = proposal_id.checked_add(1).ok_or(Error::<T>::NumOverflow)?;
            Ok(proposal_id)
        } else {
            Ok(0)
        }
    }

    /// index plus one
    pub fn queue_index_increment(dao_account: &T::AccountId) -> Result<u128, DispatchError> {
        if let Some(queue_index) = Self::last_queue_index(dao_account) {
            let queue_index = queue_index.checked_add(1).ok_or(Error::<T>::NumOverflow)?;
            Ok(queue_index)
        } else {
            Ok(0)
        }
    }

    /// dao in the period of the current block
    pub fn get_current_period(
        dao: &DAOInfo<T::AccountId, T::BlockNumber, BalanceOf<T>>,
    ) -> Result<u128, DispatchError> {
        let summoning_time = &dao.summoning_time;
        let u128_summoning_time = (*summoning_time).saturated_into::<u128>();

        let now = <system::Pallet<T>>::block_number().saturated_into::<u128>();
        let period = now
            .checked_sub(u128_summoning_time)
            .ok_or(Error::<T>::NumOverflow)?;

        let period_duration = &dao.period_duration;
        let period = period
            .checked_div(*period_duration)
            .ok_or(Error::<T>::NumOverflow)?;
        Ok(period)
    }

    /// Calculate the period for the proposal to start voting
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
                None => return Err(Error::<T>::ProposalNotFound.into()),
                Some(proposal) => proposal.starting_period,
            }
        } else {
            0
        };

        Ok(max(current_period, period))
    }

    /// increase the number of votes with shares.
    pub fn add_vote(now_vote: u128, shares: u128) -> Result<u128, DispatchError> {
        let added_vote = now_vote
            .checked_add(shares)
            .ok_or(Error::<T>::NumOverflow)?;
        Ok(added_vote)
    }

    /// Let dao perform an operation (call).
    pub fn run(dao_account: T::AccountId, action_data: &[u8]) -> Result<bool, DispatchError> {
        if let Ok(action) = T::Action::decode(&mut &action_data[..]) {
            // Ok(action.dispatch(frame_system::RawOrigin::Root.into()).is_ok())
            let dao = frame_system::RawOrigin::Signed(dao_account).into();
            // Ok(action.dispatch_bypass_filter(seld_origin).is_ok())
            Ok(action.dispatch(dao).is_ok())
        } else {
            Err(Error::<T>::DecodeFailed.into())
        }
    }
}
