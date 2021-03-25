use crate::{Error, mock::*};
use frame_support::{assert_ok, assert_noop};

#[test]
fn create_graph() {
	new_test_ext().execute_with(|| {
		// Dispatch a signed extrinsic.
		let alice_address = 1;
		let alice = Origin::signed(alice_address);

		assert_ok!(GraphModule::create_graph(alice, vec![2,3,4]>));
	});
}
