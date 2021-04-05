use crate::{mock::*, Error};
use codec::Decode;
use codec::Encode;
use frame_support::{assert_noop, assert_ok};
use sp_runtime::traits::{BlakeTwo256, Hash};
// use pallet_collection::CollectionInterface;

#[test]
fn it_works_for_default_value() {
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
