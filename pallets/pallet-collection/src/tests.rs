use crate::{mock::*, Error};
use frame_support::assert_ok;

#[test]
fn test_create_collection() {
    new_test_ext().execute_with(|| {
		let alice_address = 1;
        let alice = Origin::signed(alice_address);

        assert_ok!(TemplateModule::create_collection(alice, vec![2, 3, 3], false));

        let nonce = TemplateModule::get_nonce();
		assert_eq!(nonce, 1);

        let collection_id = TemplateModule::generate_collection_id(nonce).unwrap();
        let collection = TemplateModule::collections(collection_id);
        assert_eq!(collection.owner, alice_address);
    });
}
