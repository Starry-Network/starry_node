use crate::{Error, mock::*};
use frame_support::{assert_ok, assert_noop};

#[test]
fn link() {
	new_test_ext().execute_with(|| {
		// Dispatch a signed extrinsic.
		let alice_address = 1;
		let alice = Origin::signed(alice_address);
        let mint_amount = 10;
		let parent_token_id = 0;

        CollectionModule::create_collection(alice.clone(), vec![2, 3, 3], false).unwrap();

        let nonce = CollectionModule::get_nonce();
        let child_collection_id = CollectionModule::generate_collection_id(nonce).unwrap();

		CollectionModule::create_collection(alice.clone(), vec![2, 3, 3], false).unwrap();

		let nonce = CollectionModule::get_nonce();
        let parent_collection_id = CollectionModule::generate_collection_id(nonce).unwrap();


        assert_ok!(NFTModule::mint_non_fungible(
            alice.clone(),
            alice_address,
            child_collection_id,
            vec![2, 3, 3],
            mint_amount
        ));

		assert_ok!(NFTModule::mint_non_fungible(
            alice.clone(),
            alice_address,
            parent_collection_id,
            vec![2, 3, 3],
            mint_amount
        ));

		assert_ok!(GraphModule::link(alice.clone(), child_collection_id, 0, parent_collection_id, parent_token_id));

		assert_ok!(GraphModule::link(alice.clone(), child_collection_id, 1, child_collection_id, 0));
		assert_ok!(GraphModule::link(alice.clone(), child_collection_id, 2, child_collection_id, 1));
		assert_ok!(GraphModule::link(alice.clone(), child_collection_id, 3, child_collection_id, 2));
		assert_ok!(GraphModule::link(alice.clone(), child_collection_id, 4, child_collection_id, 3));
		assert_ok!(GraphModule::link(alice.clone(), child_collection_id, 5, child_collection_id, 4));



		let token = NFTModule::tokens(child_collection_id, 2);
		assert_eq!(token.owner, GraphModule::account_id());
		let root_owner = GraphModule::find_root_owner(child_collection_id, 1).unwrap();
		assert_eq!(root_owner, alice_address);

		let is_ancestor = GraphModule::is_ancestor((parent_collection_id, parent_token_id), (child_collection_id, 0)).unwrap();
		assert_eq!(is_ancestor, true);

		let is_ancestor = GraphModule::is_ancestor((child_collection_id, 0), (parent_collection_id, parent_token_id)).unwrap();
		assert_eq!(is_ancestor, false);

		let is_ancestor = GraphModule::is_ancestor((parent_collection_id, parent_token_id), (child_collection_id, 4)).unwrap();
		assert_eq!(is_ancestor, true);

		let is_ancestor = GraphModule::is_ancestor((child_collection_id, 2), (child_collection_id, 4)).unwrap();
		assert_eq!(is_ancestor, true);

		let is_ancestor = GraphModule::is_ancestor((child_collection_id, 5), (parent_collection_id, parent_token_id)).unwrap();
		assert_eq!(is_ancestor, false);

		assert_ok!(GraphModule::link(alice.clone(), child_collection_id, 3, parent_collection_id, parent_token_id));
	});
}

#[test]
fn recover() {
	new_test_ext().execute_with(|| {
		// Dispatch a signed extrinsic.
		let alice_address = 1;
		let alice = Origin::signed(alice_address);
        let mint_amount = 10;
		let parent_token_id = 0;

        CollectionModule::create_collection(alice.clone(), vec![2, 3, 3], false).unwrap();

        let nonce = CollectionModule::get_nonce();
        let child_collection_id = CollectionModule::generate_collection_id(nonce).unwrap();

		CollectionModule::create_collection(alice.clone(), vec![2, 3, 3], false).unwrap();

		let nonce = CollectionModule::get_nonce();
        let parent_collection_id = CollectionModule::generate_collection_id(nonce).unwrap();


        assert_ok!(NFTModule::mint_non_fungible(
            alice.clone(),
            alice_address,
            child_collection_id,
            vec![2, 3, 3],
            mint_amount
        ));

		assert_ok!(NFTModule::mint_non_fungible(
            alice.clone(),
            alice_address,
            parent_collection_id,
            vec![2, 3, 3],
            mint_amount
        ));

		assert_ok!(GraphModule::link(alice.clone(), child_collection_id, 0, parent_collection_id, parent_token_id));
		assert_ok!(GraphModule::link(alice.clone(), child_collection_id, 1, child_collection_id, 0));
		
		assert_noop!(GraphModule::recover(alice.clone(), child_collection_id, 0), Error::<Test>::CanNotRecoverParentToken);

		assert_ok!(GraphModule::recover(alice.clone(), child_collection_id, 1));

	});
}
