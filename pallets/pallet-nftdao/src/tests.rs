use crate::{mock::*, Error};
use codec::Encode;
use frame_support::traits::Currency;
use frame_support::{assert_noop, assert_ok};
use sp_core::H256;
use sp_runtime::traits::{BlakeTwo256, Hash};

// use pallet_collection::CollectionInterface;

#[test]
fn run() {
    new_test_ext().execute_with(|| {
        let value = 2;
        let preimage = Call::Template(<pallet_template::Call<Test>>::do_something(value)).encode();
        let _h = BlakeTwo256::hash(&preimage[..]);

        let is_ok = DaoModule::run(1, &preimage).unwrap();
        assert_eq!(is_ok, true);
        assert_eq!(Template::something(), Some(value));
        // assert_ok!(DaoModule::create_dao(Origin::signed(1), vec![2,3,3], 1, 1, vec![2,3,3],10));
    });
}

#[test]
fn create_dao() {
    new_test_ext().execute_with(|| {
        let alice_address = 1;
        let alice = Origin::signed(alice_address);

        assert_ok!(DaoModule::create_dao(
            alice,
            METADATA,
            PERIOD_DURATION,
            VOTING_PERIOD,
            GRACE_PERIOD,
            SHARES_REQUESTED,
            PROPOSAL_DEPOSIT,
            PROCESSING_REWARD,
            DILUTION_BOUND
        ));

        let dao_account = get_last_dao_account(&alice_address, &METADATA);
        let dao = DaoModule::dao(&dao_account);
        assert_eq!(&dao.metadata, &METADATA);
        assert_eq!(&dao.period_duration, &PERIOD_DURATION);
    });
}

#[test]
fn create_dao_failed() {
    new_test_ext().execute_with(|| {
        let alice_address = 1;
        let alice = Origin::signed(alice_address);
        assert_noop!(
            DaoModule::create_dao(
                alice.clone(),
                METADATA,
                PERIOD_DURATION,
                VOTING_PERIOD,
                GRACE_PERIOD,
                SHARES_REQUESTED,
                WRONG_PROPOSAL_DEPOSIT,
                PROCESSING_REWARD,
                DILUTION_BOUND
            ),
            Error::<Test>::DepositSmallerThanReward
        );

        assert_noop!(
            DaoModule::create_dao(
                alice.clone(),
                METADATA,
                WRONG_PERIOD_DURATION,
                VOTING_PERIOD,
                GRACE_PERIOD,
                SHARES_REQUESTED,
                PROPOSAL_DEPOSIT,
                PROCESSING_REWARD,
                DILUTION_BOUND
            ),
            Error::<Test>::PeriodDurationShouldLargeThanZero
        );
        assert_noop!(
            DaoModule::create_dao(
                alice.clone(),
                METADATA,
                PERIOD_DURATION,
                WRONG_VOTING_PERIOD,
                GRACE_PERIOD,
                SHARES_REQUESTED,
                PROPOSAL_DEPOSIT,
                PROCESSING_REWARD,
                DILUTION_BOUND
            ),
            Error::<Test>::VotingDurationShouldLargeThanZero
        );
        assert_noop!(
            DaoModule::create_dao(
                alice.clone(),
                METADATA,
                PERIOD_DURATION,
                VOTING_PERIOD,
                WRONG_GRACE_PERIOD,
                SHARES_REQUESTED,
                PROPOSAL_DEPOSIT,
                PROCESSING_REWARD,
                DILUTION_BOUND
            ),
            Error::<Test>::GracePeriodShouldLargeThanZero
        );
        assert_noop!(
            DaoModule::create_dao(
                alice.clone(),
                METADATA,
                PERIOD_DURATION,
                VOTING_PERIOD,
                GRACE_PERIOD,
                SHARES_REQUESTED,
                PROPOSAL_DEPOSIT,
                PROCESSING_REWARD,
                WRONG_DILUTION_BOUND
            ),
            Error::<Test>::DilutionBoundShouldLargeThanZero
        );
    });
}

#[test]
fn submit_proposal() {
    new_test_ext().execute_with(|| {
        let alice_address = 1;
        let alice = Origin::signed(alice_address);

        let new_dao_account =
            create_a_dao(&alice_address, PROPOSAL_DEPOSIT, PROPOSAL_DEPOSIT);

        let shares_requested = 1;
        let tribute_offered = 0;
        let tribute_nft = None::<(H256, u128)>;
        let details = Vec::new();
        let action = Some(Vec::new());

        assert_ok!(DaoModule::submit_proposal(
            alice.clone(),
            new_dao_account.clone(),
            alice_address,
            shares_requested,
            tribute_offered,
            tribute_nft,
            details,
            action
        ));

        let proposal = DaoModule::proposal(&new_dao_account, 0).unwrap();
        assert_eq!(&proposal.proposer, &alice_address);
        assert_eq!(proposal.shares_requested, 1);
    });
}
#[test]
fn submit_proposal_and_tribute() {
    new_test_ext().execute_with(|| {
        let alice_address = 1;
        let alice = Origin::signed(alice_address);

        let new_dao_account =
            create_a_dao(&alice_address, PROPOSAL_DEPOSIT, PROPOSAL_DEPOSIT);
        let escrow_id = DaoModule::escrow(&new_dao_account);

        let _ = Balances::deposit_creating(&alice_address, 100);
        assert_eq!(Balances::free_balance(&alice_address), 100);

        let shares_requested = 1;
        let tribute_offered = 1;
        let tribute_nft = None::<(H256, u128)>;
        let details = Vec::new();
        let action = Some(Vec::new());

        assert_ok!(DaoModule::submit_proposal(
            alice.clone(),
            new_dao_account.clone(),
            alice_address,
            shares_requested,
            tribute_offered,
            tribute_nft,
            details,
            action
        ));

        assert_eq!(Balances::free_balance(&alice_address), 99);
        assert_eq!(Balances::free_balance(&escrow_id), 1);
    });
}

#[test]
fn submit_proposal_and_tribute_nft() {
    new_test_ext().execute_with(|| {
        let alice_address = 1;
        let alice = Origin::signed(alice_address);

        let new_dao_account =
            create_a_dao(&alice_address, PROPOSAL_DEPOSIT, PROPOSAL_DEPOSIT);
        let escrow_id = DaoModule::escrow(&new_dao_account);

        let shares_requested = 1;
        let tribute_offered = 0;
        let details = Vec::new();
        let action = Some(Vec::new());

        let token = mint_a_nft(&alice_address);
        assert_eq!(
            <pallet_nft::Module<Test>>::address_balances((token.0, &alice_address)),
            1
        );

        assert_ok!(DaoModule::submit_proposal(
            alice.clone(),
            new_dao_account.clone(),
            alice_address,
            shares_requested,
            tribute_offered,
            Some(token.clone()),
            details,
            action
        ));

        assert_eq!(
            <pallet_nft::Module<Test>>::address_balances((token.0, &alice_address)),
            0
        );
        assert_eq!(
            <pallet_nft::Module<Test>>::address_balances((token.0, &escrow_id)),
            1
        );
    });
}

#[test]
fn cancel_proposal_and_back_tribute() {
    new_test_ext().execute_with(|| {
        let alice_address = 1;
        let alice = Origin::signed(alice_address);

        let new_dao_account =
            create_a_dao(&alice_address, PROPOSAL_DEPOSIT, PROPOSAL_DEPOSIT);
        let escrow_id = DaoModule::escrow(&new_dao_account);

        let _ = Balances::deposit_creating(&alice_address, 100);
        assert_eq!(Balances::free_balance(&alice_address), 100);

        let shares_requested = 1;
        let tribute_offered = 1;
        let tribute_nft = None::<(H256, u128)>;
        let details = Vec::new();
        let action = Some(Vec::new());

        assert_ok!(DaoModule::submit_proposal(
            alice.clone(),
            new_dao_account.clone(),
            alice_address,
            shares_requested,
            tribute_offered,
            tribute_nft,
            details,
            action
        ));

        assert_eq!(Balances::free_balance(&alice_address), 99);
        assert_eq!(Balances::free_balance(&escrow_id), 1);

        assert_ok!(DaoModule::cancel_proposal(
            alice.clone(),
            new_dao_account.clone(),
            0
        ));

        assert_eq!(Balances::free_balance(&alice_address), 100);
        assert_eq!(Balances::free_balance(&escrow_id), 0);
    });
}

#[test]
fn sponsor_proposal() {
    new_test_ext().execute_with(|| {
        let alice_address = 1;
        let alice = Origin::signed(alice_address);

        let _ = Balances::deposit_creating(&alice_address, 100);

        let new_dao_account =
            create_a_dao(&alice_address, PROPOSAL_DEPOSIT, PROPOSAL_DEPOSIT);

        let shares_requested = 1;
        let tribute_offered = 0;
        let tribute_nft = None::<(H256, u128)>;
        let details = Vec::new();
        let action = Some(Vec::new());

        assert_ok!(DaoModule::submit_proposal(
            alice.clone(),
            new_dao_account.clone(),
            alice_address,
            shares_requested,
            tribute_offered,
            tribute_nft,
            details,
            action
        ));
        assert_eq!(Balances::free_balance(&alice_address), 100);
        assert_ok!(DaoModule::sponsor_proposal(
            alice,
            new_dao_account.clone(),
            0
        ));
        let proposal = DaoModule::proposal(&new_dao_account, 0).unwrap();
        let sponsor = &proposal.sponsor.unwrap();
        let sponsored = &proposal.sponsored;
        assert_eq!(sponsor, &alice_address);
        assert_eq!(sponsored, &true);
        assert_eq!(Balances::free_balance(&alice_address), 99);
    });
}

#[test]
fn sponsor_proposal_failed() {
    new_test_ext().execute_with(|| {
        let alice_address = 1;
        let alice = Origin::signed(alice_address);

        let bob_address = 2;
        let bob = Origin::signed(bob_address);

        let new_dao_account =
            create_a_dao(&alice_address, PROPOSAL_DEPOSIT, PROPOSAL_DEPOSIT);
        // let escrow_id = DaoModule::escrow(&new_dao_account);

        let shares_requested = 1;
        let tribute_offered = 0;
        let tribute_nft = None::<(H256, u128)>;
        let details = Vec::new();
        let action = Some(Vec::new());

        assert_ok!(DaoModule::submit_proposal(
            alice.clone(),
            new_dao_account.clone(),
            alice_address,
            shares_requested,
            tribute_offered,
            tribute_nft,
            details,
            action
        ));

        assert_noop!(
            DaoModule::sponsor_proposal(bob, new_dao_account.clone(), 0),
            Error::<Test>::NotDAOMember
        );

        assert_eq!(
            DaoModule::sponsor_proposal(alice.clone(), new_dao_account.clone(), 0).is_ok(),
            false
        );

        assert_ok!(DaoModule::cancel_proposal(
            alice.clone(),
            new_dao_account.clone(),
            0
        ));

        let _ = Balances::deposit_creating(&alice_address, 100);

        assert_noop!(
            DaoModule::sponsor_proposal(alice.clone(), new_dao_account.clone(), 0),
            Error::<Test>::CancelledProposal
        );
    });
}

#[test]
fn vote_proposal() {
    new_test_ext().execute_with(|| {
        let alice_address = 1;
        let alice = Origin::signed(alice_address);

        let _ = Balances::deposit_creating(&alice_address, 100);

        let new_dao_account =
            create_a_dao(&alice_address, PROPOSAL_DEPOSIT, PROPOSAL_DEPOSIT);

        let shares_requested = 1;
        let tribute_offered = 0;
        let tribute_nft = None::<(H256, u128)>;
        let details = Vec::new();
        let action = Some(Vec::new());

        assert_ok!(DaoModule::submit_proposal(
            alice.clone(),
            new_dao_account.clone(),
            alice_address,
            shares_requested,
            tribute_offered,
            tribute_nft,
            details,
            action
        ));
        assert_ok!(DaoModule::sponsor_proposal(
            alice.clone(),
            new_dao_account.clone(),
            0
        ));

        let proposal_index = 0;
        assert_ok!(DaoModule::vote_proposal(
            alice.clone(),
            new_dao_account.clone(),
            proposal_index,
            true
        ));

        let proposal = DaoModule::proposal(&new_dao_account, 0).unwrap();

        assert_eq!(proposal.yes_votes, 1);
    });
}

#[test]
fn vote_proposal_failed() {
    new_test_ext().execute_with(|| {
        let alice_address = 1;
        let alice = Origin::signed(alice_address);

        let bob_address = 2;
        let bob = Origin::signed(bob_address);

        let _ = Balances::deposit_creating(&alice_address, 100);

        let new_dao_account =
            create_a_dao(&alice_address, PROPOSAL_DEPOSIT, PROPOSAL_DEPOSIT);

        let shares_requested = 1;
        let tribute_offered = 0;
        let tribute_nft = None::<(H256, u128)>;
        let details = Vec::new();
        let action = Some(Vec::new());

        assert_ok!(DaoModule::submit_proposal(
            alice.clone(),
            new_dao_account.clone(),
            alice_address,
            shares_requested,
            tribute_offered,
            tribute_nft,
            details,
            action
        ));
        assert_ok!(DaoModule::sponsor_proposal(
            alice.clone(),
            new_dao_account.clone(),
            0
        ));

        let proposal_index = 0;

        // vote duration is 2 blocks
        System::set_block_number(System::block_number() + 2);

        assert_noop!(
            DaoModule::vote_proposal(alice.clone(), new_dao_account.clone(), proposal_index, true),
            Error::<Test>::ExpiredPeriod
        );

        System::set_block_number(System::block_number() - 2);

        assert_noop!(
            DaoModule::vote_proposal(alice.clone(), 2, proposal_index, true),
            Error::<Test>::DAONotFound
        );
        assert_noop!(
            DaoModule::vote_proposal(alice.clone(), new_dao_account.clone(), 2, true),
            Error::<Test>::ProposalNotFound
        );
        assert_noop!(
            DaoModule::vote_proposal(bob, new_dao_account.clone(), proposal_index, true),
            Error::<Test>::PermissionDenied
        );
        assert_ok!(DaoModule::vote_proposal(
            alice.clone(),
            new_dao_account.clone(),
            proposal_index,
            true
        ));
        assert_noop!(
            DaoModule::vote_proposal(alice.clone(), new_dao_account.clone(), proposal_index, true),
            Error::<Test>::MemberAlreadyVoted
        );
    });
}

#[test]
fn process_proposal() {
    new_test_ext().execute_with(|| {
        let alice_address = 1;
        let alice = Origin::signed(alice_address);

        let _ = Balances::deposit_creating(&alice_address, 100);

        let new_dao_account =
            create_a_dao(&alice_address, PROPOSAL_DEPOSIT, PROPOSAL_DEPOSIT);

        let shares_requested = 1;
        let tribute_offered = 0;
        let tribute_nft = None::<(H256, u128)>;
        let details = Vec::new();
        let action = Some(Vec::new());

        assert_ok!(DaoModule::submit_proposal(
            alice.clone(),
            new_dao_account.clone(),
            alice_address,
            shares_requested,
            tribute_offered,
            tribute_nft,
            details,
            action
        ));
        assert_ok!(DaoModule::sponsor_proposal(
            alice.clone(),
            new_dao_account.clone(),
            0
        ));

        let proposal_index = 0;

        assert_ok!(DaoModule::vote_proposal(
            alice.clone(),
            new_dao_account.clone(),
            proposal_index,
            true
        ));

        System::set_block_number(System::block_number() + 4);

        assert_ok!(DaoModule::process_proposal(
            alice.clone(),
            new_dao_account.clone(),
            proposal_index
        ));

        let proposal = DaoModule::proposal(&new_dao_account, 0).unwrap();

        assert_eq!(proposal.did_pass, true);

        let member = DaoModule::member(&new_dao_account, &alice_address);
        assert_eq!(member.shares, 2);
    });
}

#[test]
fn process_proposal_failed() {
    new_test_ext().execute_with(|| {
        let alice_address = 1;
        let alice = Origin::signed(alice_address);

        let _ = Balances::deposit_creating(&alice_address, 100);

        let new_dao_account =
            create_a_dao(&alice_address, PROPOSAL_DEPOSIT, PROPOSAL_DEPOSIT);

        let shares_requested = 1;
        let tribute_offered = 0;
        let tribute_nft = None::<(H256, u128)>;
        let details = Vec::new();
        let action = Some(Vec::new());

        assert_ok!(DaoModule::submit_proposal(
            alice.clone(),
            new_dao_account.clone(),
            alice_address,
            shares_requested,
            tribute_offered,
            tribute_nft,
            details,
            action
        ));
        assert_ok!(DaoModule::sponsor_proposal(
            alice.clone(),
            new_dao_account.clone(),
            0
        ));

        let proposal_index = 0;

        assert_ok!(DaoModule::vote_proposal(
            alice.clone(),
            new_dao_account.clone(),
            proposal_index,
            true
        ));

        assert_noop!(
            DaoModule::process_proposal(alice.clone(), new_dao_account.clone(), proposal_index),
            Error::<Test>::NotReadyToProcessed
        );

        System::set_block_number(System::block_number() + 4);

        assert_noop!(
            DaoModule::process_proposal(alice.clone(), new_dao_account.clone(), 2),
            Error::<Test>::ProposalNotFound
        );
        assert_ok!(DaoModule::process_proposal(
            alice.clone(),
            new_dao_account.clone(),
            proposal_index
        ));
        assert_noop!(
            DaoModule::process_proposal(alice.clone(), new_dao_account.clone(), proposal_index),
            Error::<Test>::ProcessedProposal
        );
    });
}
#[test]
fn run_action() {
    new_test_ext().execute_with(|| {
        let alice_address = 1;
        let alice = Origin::signed(alice_address);

        let bob_address = 2;

        let _ = Balances::deposit_creating(&alice_address, 100);

        let new_dao_account =
            create_a_dao(&alice_address, PROPOSAL_DEPOSIT, PROPOSAL_DEPOSIT);
        let _ = Balances::deposit_creating(&new_dao_account, 100);

        let preimage =
            Call::Balances(<pallet_balances::Call<Test>>::transfer(bob_address, 10)).encode();

        let shares_requested = 1;
        let tribute_offered = 0;
        let tribute_nft = None::<(H256, u128)>;
        let details = Vec::new();
        let action = Some(preimage);

        assert_ok!(DaoModule::submit_proposal(
            alice.clone(),
            new_dao_account.clone(),
            alice_address,
            shares_requested,
            tribute_offered,
            tribute_nft,
            details,
            action
        ));
        assert_ok!(DaoModule::sponsor_proposal(
            alice.clone(),
            new_dao_account.clone(),
            0
        ));

        let proposal_index = 0;

        assert_ok!(DaoModule::vote_proposal(
            alice.clone(),
            new_dao_account.clone(),
            proposal_index,
            true
        ));

        System::set_block_number(System::block_number() + 4);

        assert_ok!(DaoModule::process_proposal(
            alice.clone(),
            new_dao_account.clone(),
            proposal_index
        ));

        let proposal = DaoModule::proposal(&new_dao_account, 0).unwrap();

        assert_eq!(&proposal.did_pass, &true);
        assert_eq!(&proposal.executed, &true);

        let member = DaoModule::member(&new_dao_account, &alice_address);
        assert_eq!(member.shares, 2);

        assert_eq!(Balances::free_balance(&bob_address), 10);
    });
}

#[test]
fn ragequit() {
    new_test_ext().execute_with(|| {
        let alice_address = 1;
        let alice = Origin::signed(alice_address);

        let _ = Balances::deposit_creating(&alice_address, 100);

        let new_dao_account =
            create_a_dao(&alice_address, PROPOSAL_DEPOSIT, PROPOSAL_DEPOSIT);

        let shares_requested = 1;
        let tribute_offered = 1;
        // let tribute_offered = 0;
        let tribute_nft = None::<(H256, u128)>;
        let details = Vec::new();
        let action = Some(Vec::new());

        assert_ok!(DaoModule::submit_proposal(
            alice.clone(),
            new_dao_account.clone(),
            alice_address,
            shares_requested,
            tribute_offered,
            tribute_nft,
            details,
            action
        ));

        let escrow_id = DaoModule::escrow(&new_dao_account);

        assert_eq!(Balances::free_balance(&new_dao_account), 0);
        assert_eq!(Balances::free_balance(&escrow_id), 1);

        assert_ok!(DaoModule::sponsor_proposal(
            alice.clone(),
            new_dao_account.clone(),
            0
        ));

        assert_eq!(Balances::free_balance(&new_dao_account), 0);
        assert_eq!(Balances::free_balance(&escrow_id), 2);
        assert_eq!(Balances::free_balance(&alice_address), 98);

        let proposal_index = 0;

        assert_ok!(DaoModule::vote_proposal(
            alice.clone(),
            new_dao_account.clone(),
            proposal_index,
            true
        ));

        let member = DaoModule::member(&new_dao_account, &alice_address);
        assert_eq!(member.shares, 1);

        System::set_block_number(System::block_number() + 4);

        assert_ok!(DaoModule::process_proposal(
            alice.clone(),
            new_dao_account.clone(),
            proposal_index
        ));

        let member = DaoModule::member(&new_dao_account, &alice_address);
        assert_eq!(member.shares, 2);

        assert_eq!(Balances::free_balance(&new_dao_account), 1);
        assert_eq!(Balances::free_balance(&escrow_id), 0);
        assert_eq!(Balances::free_balance(&alice_address), 99);

        assert_ok!(DaoModule::ragequit(
            alice.clone(),
            new_dao_account.clone(),
            2
        ));

        assert_eq!(Balances::free_balance(&new_dao_account), 0);
        assert_eq!(Balances::free_balance(&escrow_id), 0);
        assert_eq!(Balances::free_balance(&alice_address), 100);

        let member = DaoModule::member(&new_dao_account, &alice_address);
        assert_eq!(member.shares, 0);
    });
}

#[test]
fn ragequit_failed() {
    new_test_ext().execute_with(|| {
        let alice_address = 1;
        let alice = Origin::signed(alice_address);

        let bob_address = 2;
        let bob = Origin::signed(bob_address);

        let _ = Balances::deposit_creating(&alice_address, 100);

        let new_dao_account =
            create_a_dao(&alice_address, PROPOSAL_DEPOSIT, PROPOSAL_DEPOSIT);

        let shares_requested = 1;
        let tribute_offered = 0;
        let tribute_nft = None::<(H256, u128)>;
        let details = Vec::new();
        let action = Some(Vec::new());

        assert_ok!(DaoModule::submit_proposal(
            alice.clone(),
            new_dao_account.clone(),
            alice_address,
            shares_requested,
            tribute_offered,
            tribute_nft,
            details,
            action
        ));
        assert_ok!(DaoModule::sponsor_proposal(
            alice.clone(),
            new_dao_account.clone(),
            0
        ));

        assert_noop!(
            DaoModule::ragequit(alice.clone(), new_dao_account.clone(), 1),
            Error::<Test>::CanNotRagequit
        );

        let proposal_index = 0;

        assert_ok!(DaoModule::vote_proposal(
            alice.clone(),
            new_dao_account.clone(),
            proposal_index,
            true
        ));

        assert_noop!(
            DaoModule::ragequit(alice.clone(), new_dao_account.clone(), 1),
            Error::<Test>::CanNotRagequit
        );

        assert_noop!(
            DaoModule::ragequit(alice.clone(), new_dao_account.clone(), 2),
            Error::<Test>::InsufficientShares
        );

        assert_noop!(
            DaoModule::ragequit(bob.clone(), new_dao_account.clone(), 1),
            Error::<Test>::PermissionDenied
        );
    });
}
