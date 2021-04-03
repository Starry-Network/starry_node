use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};
use pallet_nft;
use pallet_collection::CollectionInterface;

use sp_core::H256;

#[test]
fn it_works_for_pallet_collection() {
    new_test_ext().execute_with(|| {
        let alice = Origin::signed(1);
        assert_ok!(CollectionModule::create_collection(
            alice,
            vec![2, 3, 3],
            false
        ));
    });
}

#[test]
fn create_success() {
    new_test_ext().execute_with(|| {
        let alice_address = 1;
        let alice = Origin::signed(alice_address);
        CollectionModule::create_collection(alice.clone(), vec![2, 3, 3], false).unwrap();

        let nonce = CollectionModule::get_nonce();
        let collection_id = <CollectionModule as CollectionInterface<_, _>>::generate_collection_id(nonce).unwrap();
        let mint_amount = 10;
        NFTModule::mint_non_fungible(
            alice.clone(),
            alice_address,
            collection_id,
            vec![2, 3, 3],
            mint_amount,
        )
        .unwrap();
        let last_token_id = NFTModule::last_token_id(collection_id);
        let start_idx = mint_amount - last_token_id - 1;

        assert_ok!(SubModule::create(alice, collection_id, start_idx, false));

        let token = NFTModule::tokens(collection_id, start_idx);

        assert_eq!(token.owner, SubModule::account_id());

        let nonce = CollectionModule::get_nonce();
        let collection_id = <CollectionModule as CollectionInterface<_, _>>::generate_collection_id(nonce).unwrap();
        let collection = CollectionModule::collections(collection_id);

        assert_eq!(collection.owner, SubModule::account_id());
    });
}

#[test]
fn create_failed() {
    new_test_ext().execute_with(|| {
        let alice_address = 1;
        let alice = Origin::signed(alice_address);
        let bob_address = 2;
        let bob = Origin::signed(bob_address);

        CollectionModule::create_collection(alice.clone(), vec![2, 3, 3], false).unwrap();

        let nonce = CollectionModule::get_nonce();
        let collection_id = CollectionModule::generate_collection_id(nonce).unwrap();
        let not_available_collection_id =
            CollectionModule::generate_collection_id(nonce + 1).unwrap();
        NFTModule::mint_non_fungible(
            alice.clone(),
            alice_address,
            collection_id,
            vec![2, 3, 3],
            10,
        )
        .unwrap();

        let start_idx = 1;
        assert_noop!(
            SubModule::create(alice.clone(), collection_id, start_idx, false),
            <pallet_nft::Error<Test>>::TokenNotFound
        );

        let start_idx = 0;
        assert_noop!(
            SubModule::create(alice, not_available_collection_id, start_idx, false),
            <pallet_nft::Error<Test>>::CollectionNotFound
        );
        assert_noop!(
            SubModule::create(bob, collection_id, start_idx, false),
            <pallet_nft::Error<Test>>::PermissionDenied
        );
    });
}

#[test]
fn recover_success() {
    new_test_ext().execute_with(|| {
        let alice_address = 1;
        let alice = Origin::signed(alice_address);
        CollectionModule::create_collection(alice.clone(), vec![2, 3, 3], false).unwrap();

        let nonce = CollectionModule::get_nonce();
        let collection_id = <CollectionModule as CollectionInterface<_, _>>::generate_collection_id(nonce).unwrap();
        let mint_amount = 10;
        NFTModule::mint_non_fungible(
            alice.clone(),
            alice_address,
            collection_id,
            vec![2, 3, 3],
            mint_amount,
        )
        .unwrap();
        let last_token_id = NFTModule::last_token_id(collection_id);
        let start_idx = mint_amount - last_token_id - 1;

        assert_ok!(SubModule::create(
            alice.clone(),
            collection_id,
            start_idx,
            false
        ));

        let nonce = CollectionModule::get_nonce();
        let sub_token_collection_id = CollectionModule::generate_collection_id(nonce).unwrap();

        assert_ok!(SubModule::recover(alice, sub_token_collection_id));

        let token = NFTModule::tokens(collection_id, start_idx);

        assert_eq!(token.owner, alice_address);

        let (collection_id, _) = SubModule::sub_tokens(sub_token_collection_id);

        assert_eq!(H256::is_zero(&collection_id), true);
    });
}

#[test]
fn recover_failed() {
    new_test_ext().execute_with(|| {
        let alice_address = 1;
        let alice = Origin::signed(alice_address);
        let bob_address = 2;
        let bob = Origin::signed(bob_address);

        CollectionModule::create_collection(alice.clone(), vec![2, 3, 3], false).unwrap();

        let nonce = CollectionModule::get_nonce();
        let collection_id = <CollectionModule as CollectionInterface<_, _>>::generate_collection_id(nonce).unwrap();
        let mint_amount = 10;
        NFTModule::mint_non_fungible(
            alice.clone(),
            alice_address,
            collection_id,
            vec![2, 3, 3],
            mint_amount,
        )
        .unwrap();
        let last_token_id = NFTModule::last_token_id(collection_id);
        let start_idx = mint_amount - last_token_id - 1;

        assert_ok!(SubModule::create(
            alice.clone(),
            collection_id,
            start_idx,
            false
        ));

        let nonce = CollectionModule::get_nonce();
        let sub_token_collection_id = CollectionModule::generate_collection_id(nonce).unwrap();

        let not_available_collection_id =
            CollectionModule::generate_collection_id(nonce + 1).unwrap();

        assert_noop!(
            SubModule::recover(alice.clone(), not_available_collection_id),
            Error::<Test>::CollectionNotFound
        );
        assert_noop!(
            SubModule::recover(alice.clone(), collection_id),
            Error::<Test>::SubTokenNotFound
        );
        assert_noop!(
            SubModule::recover(bob, sub_token_collection_id),
            Error::<Test>::PermissionDenied
        );

        let burn_amount = 2;
        SubModule::mint_non_fungible(
            alice.clone(),
            alice_address,
            sub_token_collection_id,
            vec![2, 3, 3],
            mint_amount,
        )
        .unwrap();
        let last_sub_token_id = NFTModule::last_token_id(sub_token_collection_id);
        let sub_token_start_idx = mint_amount - last_sub_token_id - 1;

        NFTModule::burn_non_fungible(
            alice.clone(),
            sub_token_collection_id,
            sub_token_start_idx,
            burn_amount,
        )
        .unwrap();
        assert_noop!(
            SubModule::recover(alice, sub_token_collection_id),
            Error::<Test>::BurnedtokensExistent
        );
    });
}

#[test]
fn mint_non_fungible_success() {
    new_test_ext().execute_with(|| {
        let alice_address = 1;
        let alice = Origin::signed(alice_address);
        CollectionModule::create_collection(alice.clone(), vec![2, 3, 3], false).unwrap();

        let nonce = CollectionModule::get_nonce();
        let collection_id = <CollectionModule as CollectionInterface<_, _>>::generate_collection_id(nonce).unwrap();
        let mint_amount = 10;
        NFTModule::mint_non_fungible(
            alice.clone(),
            alice_address,
            collection_id,
            vec![2, 3, 3],
            mint_amount,
        )
        .unwrap();
        let last_token_id = NFTModule::last_token_id(collection_id);
        let start_idx = mint_amount - last_token_id - 1;

        assert_ok!(SubModule::create(
            alice.clone(),
            collection_id,
            start_idx,
            false
        ));

        let nonce = CollectionModule::get_nonce();
        let sub_token_collection_id = CollectionModule::generate_collection_id(nonce).unwrap();

        assert_ok!(SubModule::mint_non_fungible(
            alice.clone(),
            alice_address,
            sub_token_collection_id,
            vec![2, 3, 3],
            mint_amount
        ));

        assert_eq!(
            NFTModule::address_balances((sub_token_collection_id, alice_address)),
            mint_amount
        );

        let last_sub_token_id = NFTModule::last_token_id(sub_token_collection_id);
        assert_eq!(last_sub_token_id, mint_amount - 1);

        let sub_token_start_idx = last_token_id + 1 - mint_amount;
        let sub_token = NFTModule::tokens(sub_token_collection_id, sub_token_start_idx);

        assert_eq!(sub_token.end_idx, mint_amount - 1);
        assert_eq!(
            NFTModule::address_balances((sub_token_collection_id, alice_address)),
            mint_amount
        );

        let collection = CollectionModule::collections(sub_token_collection_id);

        assert_eq!(collection.total_supply, mint_amount)
    });
}

#[test]
fn mint_non_fungible_failed() {
    new_test_ext().execute_with(|| {
        let alice_address = 1;
        let alice = Origin::signed(alice_address);
        CollectionModule::create_collection(alice.clone(), vec![2, 3, 3], false).unwrap();

        let nonce = CollectionModule::get_nonce();
        let collection_id = <CollectionModule as CollectionInterface<_, _>>::generate_collection_id(nonce).unwrap();
        let mint_amount = 10;
        NFTModule::mint_non_fungible(
            alice.clone(),
            alice_address,
            collection_id,
            vec![2, 3, 3],
            mint_amount,
        )
        .unwrap();
        let last_token_id = NFTModule::last_token_id(collection_id);
        let start_idx = mint_amount - last_token_id - 1;

        assert_ok!(SubModule::create(
            alice.clone(),
            collection_id,
            start_idx,
            false
        ));

        let nonce = CollectionModule::get_nonce();
        let sub_token_collection_id = CollectionModule::generate_collection_id(nonce).unwrap();
        let not_available_sub_token_collection_id =
            CollectionModule::generate_collection_id(nonce + 1).unwrap();

        assert_noop!(
            SubModule::mint_non_fungible(
                alice.clone(),
                alice_address,
                not_available_sub_token_collection_id,
                vec![2, 3, 3],
                mint_amount
            ),
            Error::<Test>::CollectionNotFound
        );

        assert_noop!(
            SubModule::mint_non_fungible(
                alice.clone(),
                alice_address,
                collection_id,
                vec![2, 3, 3],
                mint_amount
            ),
            Error::<Test>::SubTokenNotFound
        );

        let mint_amount = 0;

        assert_noop!(
            SubModule::mint_non_fungible(
                alice.clone(),
                alice_address,
                sub_token_collection_id,
                vec![2, 3, 3],
                mint_amount
            ),
            <pallet_nft::Error<Test>>::AmountLessThanOne
        );
    });
}

#[test]
fn mint_fungible_success() {
    new_test_ext().execute_with(|| {
        let alice_address = 1;
        let alice = Origin::signed(alice_address);
        CollectionModule::create_collection(alice.clone(), vec![2, 3, 3], false).unwrap();

        let nonce = CollectionModule::get_nonce();
        let collection_id = <CollectionModule as CollectionInterface<_, _>>::generate_collection_id(nonce).unwrap();
        let mint_amount = 10;
        NFTModule::mint_non_fungible(
            alice.clone(),
            alice_address,
            collection_id,
            vec![2, 3, 3],
            mint_amount,
        )
        .unwrap();
        let last_token_id = NFTModule::last_token_id(collection_id);
        let start_idx = mint_amount - last_token_id - 1;

        assert_ok!(SubModule::create(
            alice.clone(),
            collection_id,
            start_idx,
            true
        ));

        let nonce = CollectionModule::get_nonce();
        let sub_token_collection_id = CollectionModule::generate_collection_id(nonce).unwrap();

        assert_ok!(SubModule::mint_fungible(
            alice.clone(),
            alice_address,
            sub_token_collection_id,
            mint_amount
        ));

        assert_eq!(
            NFTModule::address_balances((sub_token_collection_id, alice_address)),
            mint_amount
        );

        let collection = CollectionModule::collections(sub_token_collection_id);

        assert_eq!(collection.total_supply, mint_amount);
    });
}

#[test]
fn mint_fungible_failed() {
    new_test_ext().execute_with(|| {
        let alice_address = 1;
        let alice = Origin::signed(alice_address);
        CollectionModule::create_collection(alice.clone(), vec![2, 3, 3], false).unwrap();

        let nonce = CollectionModule::get_nonce();
        let collection_id = <CollectionModule as CollectionInterface<_, _>>::generate_collection_id(nonce).unwrap();
        let mint_amount = 10;
        NFTModule::mint_non_fungible(
            alice.clone(),
            alice_address,
            collection_id,
            vec![2, 3, 3],
            mint_amount,
        )
        .unwrap();
        let last_token_id = NFTModule::last_token_id(collection_id);
        let start_idx = mint_amount - last_token_id - 1;

        assert_ok!(SubModule::create(
            alice.clone(),
            collection_id,
            start_idx,
            true
        ));

        let nonce = CollectionModule::get_nonce();
        let sub_token_collection_id = CollectionModule::generate_collection_id(nonce).unwrap();
        let not_available_sub_token_collection_id =
            CollectionModule::generate_collection_id(nonce + 1).unwrap();

        let mint_amount = 5;

        assert_noop!(
            SubModule::mint_fungible(
                alice.clone(),
                alice_address,
                not_available_sub_token_collection_id,
                mint_amount
            ),
            Error::<Test>::CollectionNotFound
        );

        assert_noop!(
            SubModule::mint_fungible(
                alice.clone(),
                alice_address,
                collection_id,
                mint_amount
            ),
            Error::<Test>::SubTokenNotFound
        );

        let mint_amount = 0;
        assert_noop!(
            SubModule::mint_fungible(alice.clone(), alice_address, sub_token_collection_id, mint_amount),
            <pallet_nft::Error<Test>>::AmountLessThanOne
        );
    });
}
