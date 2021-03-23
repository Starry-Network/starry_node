use crate::{Error, mock::*};
use frame_support::{assert_ok};

#[test]
fn test_create_collection() {
	new_test_ext().execute_with(|| {
		let alice = Origin::signed(1);

		assert_ok!(TemplateModule::create_collection(alice, vec![2, 3, 3]));

		// let last_collection_id = TemplateModule::last_collection_id();

		// assert_eq!(last_collection_id, 0);

		// let collection = TemplateModule::collections(last_collection_id);

		// assert_eq!(collection.owner, 1);
		// assert_eq!(collection.uri, vec![2, 3, 3]);
	});
}
