use crate::{mock::*, Error};
use codec::Decode;
use codec::Encode;
use frame_support::{assert_noop, assert_ok};
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
            DAO_NAME,
            METADATA,
            PERIOD_DURATION,
            VOTING_PERIOD,
            GRACE_PERIOD,
            SHARES_REQUESTED,
            PROPOSAL_DEPOSIT,
            PROCESSING_REWARD,
            DILUTION_BOUND
        ));

        let dao_account = get_last_dao_account(&alice_address, &DAO_NAME);
        let dao = DaoModule::dao(&dao_account);
        assert_eq!(&dao.name, &DAO_NAME);
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
                DAO_NAME,
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
                DAO_NAME,
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
                DAO_NAME,
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
                DAO_NAME,
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
                DAO_NAME,
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
