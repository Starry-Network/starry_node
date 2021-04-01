use crate::{mock::*, Error};
use codec::Decode;
use codec::Encode;
use frame_support::{assert_noop, assert_ok};

#[test]
fn it_works_for_default_value() {
    new_test_ext().execute_with(|| {
        assert_ok!(DaoModule::do_something(Origin::signed(1), 42));
        assert_eq!(DaoModule::something(), Some(42));

        let value = 2;
        let preimage = Call::Template(<pallet_template::Call<Test>>::do_something(value)).encode();
        let is_ok =  DaoModule::run( preimage).unwrap();
        assert_eq!(is_ok, true);
        assert_eq!(Template::something(), Some(value));


    });
}

// #[test]
// fn correct_error_for_none_value() {
//     new_test_ext().execute_with(|| {
//         // Ensure the expected error is thrown when no value is present.
//         assert_noop!(
//             DaoModule::cause_error(Origin::signed(1)),
//             Error::<Test>::NoneValue
//         );
//     });
// }
