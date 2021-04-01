use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};
use sp_core::H256;
use pallet_collection;

#[test]
fn link_non_fungible() {
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

        assert_ok!(GraphModule::link_non_fungible(
            alice.clone(),
            child_collection_id,
            0,
            parent_collection_id,
            parent_token_id
        ));

        assert_ok!(GraphModule::link_non_fungible(
            alice.clone(),
            child_collection_id,
            1,
            child_collection_id,
            0
        ));
        assert_ok!(GraphModule::link_non_fungible(
            alice.clone(),
            child_collection_id,
            2,
            child_collection_id,
            1
        ));
        assert_ok!(GraphModule::link_non_fungible(
            alice.clone(),
            child_collection_id,
            3,
            child_collection_id,
            2
        ));
        assert_ok!(GraphModule::link_non_fungible(
            alice.clone(),
            child_collection_id,
            4,
            child_collection_id,
            3
        ));
        assert_ok!(GraphModule::link_non_fungible(
            alice.clone(),
            child_collection_id,
            5,
            child_collection_id,
            4
        ));

        let token = NFTModule::tokens(child_collection_id, 2);
        assert_eq!(token.owner, GraphModule::account_id());
        let root_owner = GraphModule::find_root_owner(child_collection_id, 1).unwrap();
        assert_eq!(root_owner, alice_address);

        let is_ancestor = GraphModule::is_ancestor(
            (parent_collection_id, parent_token_id),
            (child_collection_id, 0),
        )
        .unwrap();
        assert_eq!(is_ancestor, true);

        let is_ancestor = GraphModule::is_ancestor(
            (child_collection_id, 0),
            (parent_collection_id, parent_token_id),
        )
        .unwrap();
        assert_eq!(is_ancestor, false);

        let is_ancestor = GraphModule::is_ancestor(
            (parent_collection_id, parent_token_id),
            (child_collection_id, 4),
        )
        .unwrap();
        assert_eq!(is_ancestor, true);

        let is_ancestor =
            GraphModule::is_ancestor((child_collection_id, 2), (child_collection_id, 4)).unwrap();
        assert_eq!(is_ancestor, true);

        let is_ancestor = GraphModule::is_ancestor(
            (child_collection_id, 5),
            (parent_collection_id, parent_token_id),
        )
        .unwrap();
        assert_eq!(is_ancestor, false);

        assert_ok!(GraphModule::link_non_fungible(
            alice.clone(),
            child_collection_id,
            3,
            parent_collection_id,
            parent_token_id
        ));
    });
}

#[test]
fn link_fungible() {
    new_test_ext().execute_with(|| {
        // Dispatch a signed extrinsic.
        let alice_address = 1;
        let alice = Origin::signed(alice_address);
        let bob_address = 2;
        let bob = Origin::signed(bob_address);

        CollectionModule::create_collection(alice.clone(), vec![2, 3, 3], true).unwrap();

        let nonce = CollectionModule::get_nonce();
        let fungible_collection_id = CollectionModule::generate_collection_id(nonce).unwrap();

        CollectionModule::create_collection(alice.clone(), vec![2, 3, 3], false).unwrap();

        let nonce = CollectionModule::get_nonce();
        let child_collection_id = CollectionModule::generate_collection_id(nonce).unwrap();

        CollectionModule::create_collection(alice.clone(), vec![2, 3, 3], false).unwrap();

        let nonce = CollectionModule::get_nonce();
        let parent_collection_id = CollectionModule::generate_collection_id(nonce).unwrap();

        assert_ok!(NFTModule::mint_fungible(
            alice.clone(),
            alice_address,
            fungible_collection_id,
            10
        ));

        assert_ok!(NFTModule::mint_non_fungible(
            alice.clone(),
            alice_address,
            parent_collection_id,
            vec![2, 3, 3],
            1
        ));

        assert_ok!(NFTModule::mint_non_fungible(
            alice.clone(),
            alice_address,
            child_collection_id,
            vec![2, 3, 3],
            1
        ));

        let child_collection_id_none = None::<H256>;
        let child_token_id_none = None::<u128>;
        assert_ok!(GraphModule::link_fungible(
            alice.clone(),
            child_collection_id_none,
            child_token_id_none,
            fungible_collection_id,
            child_collection_id,
            0,
            1
        ));

        // let child_token_none: Option<(H256, u128)>= None::<(H256, u128)>;
        // assert_ok!(GraphModule::link_fungible(
        //     bob.clone(),
        //     child_token_none,
        //     fungible_collection_id,
        //     child_collection_id,
        //     0,
        //     0
        // ));

        let child_collection_id: Option<H256> = Some(child_collection_id);
        let child_token_id: Option<u128> = Some(0);

        assert_noop!(
            GraphModule::link_fungible(
                bob.clone(),
                child_collection_id,
                child_token_id,
                fungible_collection_id,
                parent_collection_id,
                0,
                1
            ),
            Error::<Test>::PermissionDenied
        );
        assert_ok!(GraphModule::link_fungible(
            alice.clone(),
            child_collection_id,
            child_token_id,
            fungible_collection_id,
            parent_collection_id,
            0,
            1
        ));
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

        assert_ok!(GraphModule::link_non_fungible(
            alice.clone(),
            child_collection_id,
            0,
            parent_collection_id,
            parent_token_id
        ));
        assert_ok!(GraphModule::link_non_fungible(
            alice.clone(),
            child_collection_id,
            1,
            child_collection_id,
            0
        ));

        assert_noop!(
            GraphModule::recover_non_fungible(alice.clone(), child_collection_id, 0),
            Error::<Test>::CanNotRecoverParentToken
        );

        assert_ok!(GraphModule::recover_non_fungible(
            alice.clone(),
            child_collection_id,
            1
        ));
    });
}

#[test]
fn recover_fungible() {
    new_test_ext().execute_with(|| {
        // Dispatch a signed extrinsic.
        let alice_address = 1;
        let alice = Origin::signed(alice_address);
        let bob_address = 2;
        let bob = Origin::signed(bob_address);

        CollectionModule::create_collection(alice.clone(), vec![2, 3, 3], true).unwrap();

        let nonce = CollectionModule::get_nonce();
        let fungible_collection_id = CollectionModule::generate_collection_id(nonce).unwrap();

        CollectionModule::create_collection(alice.clone(), vec![2, 3, 3], false).unwrap();

        let nonce = CollectionModule::get_nonce();
        let child_collection_id = CollectionModule::generate_collection_id(nonce).unwrap();

        CollectionModule::create_collection(alice.clone(), vec![2, 3, 3], false).unwrap();

        let nonce = CollectionModule::get_nonce();
        let parent_collection_id = CollectionModule::generate_collection_id(nonce).unwrap();

        assert_ok!(NFTModule::mint_fungible(
            alice.clone(),
            alice_address,
            fungible_collection_id,
            10
        ));

        assert_ok!(NFTModule::mint_non_fungible(
            alice.clone(),
            alice_address,
            parent_collection_id,
            vec![2, 3, 3],
            1
        ));

        assert_ok!(NFTModule::mint_non_fungible(
            alice.clone(),
            alice_address,
            child_collection_id,
            vec![2, 3, 3],
            1
        ));

        let child_collection_id_none = None::<H256>;
        let child_token_id_none = None::<u128>;
        assert_ok!(GraphModule::link_fungible(
            alice.clone(),
            child_collection_id_none,
            child_token_id_none,
            fungible_collection_id,
            child_collection_id,
            0,
            1
        ));

        let child_collection_id: Option<H256> = Some(child_collection_id);
        let child_token_id: Option<u128> = Some(0);

        assert_ok!(GraphModule::link_fungible(
            alice.clone(),
            child_collection_id,
            child_token_id,
            fungible_collection_id,
            parent_collection_id,
            0,
            1
        ));

        assert_noop!(
            GraphModule::recover_fungible(
                bob.clone(),
                parent_collection_id,
                0,
                fungible_collection_id,
                1
            ),
            Error::<Test>::PermissionDenied
        );

        assert_ok!(GraphModule::recover_fungible(
            alice.clone(),
            parent_collection_id,
            0,
            fungible_collection_id,
            1
        ));
    });
}
