use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};
use pallet_collection::CollectionInterface;

#[test]
fn mint_non_fungible_success() {
    new_test_ext().execute_with(|| {
        let alice_address = 1;
        let alice = Origin::signed(alice_address);
        CollectionModule::create_collection(alice.clone(), vec![2, 3, 3], false).unwrap();

        let nonce = CollectionModule::get_nonce();
        let collection_id = <CollectionModule as CollectionInterface<_, _>>::generate_collection_id(nonce).unwrap();

        let mint_amount = 5;

        assert_ok!(NFTModule::mint_non_fungible(
            alice,
            alice_address,
            collection_id,
            vec![2, 3, 3],
            mint_amount
        ));

        assert_eq!(
            NFTModule::address_balances((collection_id, alice_address)),
            mint_amount
        );

        let last_token_id = NFTModule::last_token_id(collection_id);
        assert_eq!(last_token_id, mint_amount - 1);

        let start_idx = last_token_id + 1 - mint_amount;
        let token = NFTModule::tokens(collection_id, start_idx);

        assert_eq!(token.end_idx, mint_amount - 1);
        assert_eq!(
            NFTModule::address_balances((collection_id, alice_address)),
            mint_amount
        );

        let collection = CollectionModule::collections(collection_id);

        assert_eq!(collection.total_supply, mint_amount);
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
        let not_available_collection_id =
            CollectionModule::generate_collection_id(nonce + 1).unwrap();
        let mint_amount = 5;

        assert_noop!(
            NFTModule::mint_non_fungible(
                alice.clone(),
                alice_address,
                not_available_collection_id,
                vec![2, 3, 3],
                mint_amount
            ),
            Error::<Test>::CollectionNotFound
        );

        let mint_amount = 0;
        assert_noop!(
            NFTModule::mint_non_fungible(
                alice,
                alice_address,
                collection_id,
                vec![2, 3, 3],
                mint_amount
            ),
            Error::<Test>::AmountLessThanOne
        );
    });
}
#[test]
fn mint_fungible_success() {
    new_test_ext().execute_with(|| {
        let alice_address = 1;
        let alice = Origin::signed(alice_address);
        CollectionModule::create_collection(alice.clone(), vec![2, 3, 3], true).unwrap();

        let nonce = CollectionModule::get_nonce();
        let collection_id = <CollectionModule as CollectionInterface<_, _>>::generate_collection_id(nonce).unwrap();
        let mint_amount = 5;

        assert_ok!(NFTModule::mint_fungible(
            alice,
            alice_address,
            collection_id,
            mint_amount
        ));

        assert_eq!(
            NFTModule::address_balances((collection_id, alice_address)),
            mint_amount
        );

        let collection = CollectionModule::collections(collection_id);

        assert_eq!(collection.total_supply, mint_amount);
    });
}

#[test]
fn mint_fungible_failed() {
    new_test_ext().execute_with(|| {
        let alice_address = 1;
        let alice = Origin::signed(alice_address);
        CollectionModule::create_collection(alice.clone(), vec![2, 3, 3], true).unwrap();

        let nonce = CollectionModule::get_nonce();
        let collection_id = <CollectionModule as CollectionInterface<_, _>>::generate_collection_id(nonce).unwrap();
        let not_available_collection_id =
            CollectionModule::generate_collection_id(nonce + 1).unwrap();
        let mint_amount = 5;

        assert_noop!(
            NFTModule::mint_fungible(
                alice.clone(),
                alice_address,
                not_available_collection_id,
                mint_amount
            ),
            Error::<Test>::CollectionNotFound
        );

        let mint_amount = 0;
        assert_noop!(
            NFTModule::mint_fungible(alice, alice_address, collection_id, mint_amount),
            Error::<Test>::AmountLessThanOne
        );
    });
}

#[test]
fn transfer_non_fungible_success() {
    new_test_ext().execute_with(|| {
        let alice_address = 1;
        let bob_address = 2;
        let alice = Origin::signed(alice_address);
        CollectionModule::create_collection(alice.clone(), vec![2, 3, 3], false).unwrap();

        let nonce = CollectionModule::get_nonce();
        let collection_id = <CollectionModule as CollectionInterface<_, _>>::generate_collection_id(nonce).unwrap();
        let mint_amount = 5;
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
        let token = NFTModule::tokens(collection_id, start_idx);

        assert_eq!(token.owner, alice_address);
        assert_eq!(
            NFTModule::address_balances((collection_id, alice_address)),
            mint_amount
        );

        let transfer_amount = 2;
        assert_ok!(NFTModule::transfer_non_fungible(
            alice,
            bob_address,
            collection_id,
            start_idx,
            transfer_amount
        ));

        assert_eq!(
            NFTModule::address_balances((collection_id, alice_address)),
            mint_amount - transfer_amount
        );
        assert_eq!(
            NFTModule::address_balances((collection_id, bob_address)),
            transfer_amount
        );

        let alice_start_idx = last_token_id - transfer_amount;
        let bob_nfts = NFTModule::tokens(collection_id, start_idx);
        let alice_nfts = NFTModule::tokens(collection_id, alice_start_idx);

        assert_eq!(bob_nfts.owner, bob_address);
        assert_eq!(alice_nfts.owner, alice_address);

        assert_eq!(alice_nfts.uri, bob_nfts.uri);
    });
}

#[test]
fn transfer_non_fungible_failed() {
    new_test_ext().execute_with(|| {
        let alice_address = 1;
        let bob_address = 2;
        let alice = Origin::signed(alice_address);
        let bob = Origin::signed(bob_address);

        CollectionModule::create_collection(alice.clone(), vec![2, 3, 3], false).unwrap();

        let nonce = CollectionModule::get_nonce();
        let collection_id = <CollectionModule as CollectionInterface<_, _>>::generate_collection_id(nonce).unwrap();
        let mint_amount = 5;
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
        let transfer_amount = 0;

        assert_noop!(
            NFTModule::transfer_non_fungible(
                alice.clone(),
                bob_address,
                collection_id,
                start_idx,
                transfer_amount
            ),
            Error::<Test>::AmountLessThanOne
        );

        let transfer_amount = 2;
        let not_available_token_id = last_token_id + 1;
        let not_available_collection_id =
            CollectionModule::generate_collection_id(nonce + 1).unwrap();

        assert_noop!(
            NFTModule::transfer_non_fungible(
                alice.clone(),
                bob_address,
                collection_id,
                not_available_token_id,
                transfer_amount
            ),
            Error::<Test>::TokenNotFound
        );
        assert_noop!(
            NFTModule::transfer_non_fungible(
                alice.clone(),
                bob_address,
                not_available_collection_id,
                start_idx,
                transfer_amount
            ),
            Error::<Test>::CollectionNotFound
        );
        assert_noop!(
            NFTModule::transfer_non_fungible(
                bob,
                alice_address,
                collection_id,
                start_idx,
                transfer_amount
            ),
            Error::<Test>::PermissionDenied
        );
        assert_noop!(
            NFTModule::transfer_non_fungible(
                alice.clone(),
                alice_address,
                collection_id,
                start_idx,
                transfer_amount
            ),
            Error::<Test>::ReceiverIsSender
        );

        let transfer_amount = 10;

        assert_noop!(
            NFTModule::transfer_non_fungible(
                alice,
                bob_address,
                collection_id,
                start_idx,
                transfer_amount
            ),
            Error::<Test>::AmountTooLarge
        );
    });
}

#[test]
fn transfer_fungible_success() {
    new_test_ext().execute_with(|| {
        let alice_address = 1;
        let bob_address = 2;
        let alice = Origin::signed(alice_address);
        CollectionModule::create_collection(alice.clone(), vec![2, 3, 3], true).unwrap();

        let nonce = CollectionModule::get_nonce();
        let collection_id = <CollectionModule as CollectionInterface<_, _>>::generate_collection_id(nonce).unwrap();
        let mint_amount = 5;
        NFTModule::mint_fungible(alice.clone(), alice_address, collection_id, mint_amount).unwrap();

        assert_eq!(
            NFTModule::address_balances((collection_id, alice_address)),
            mint_amount
        );

        let transfer_amount = 2;
        assert_ok!(NFTModule::transfer_fungible(
            alice,
            bob_address,
            collection_id,
            transfer_amount
        ));

        assert_eq!(
            NFTModule::address_balances((collection_id, alice_address)),
            mint_amount - transfer_amount
        );
        assert_eq!(
            NFTModule::address_balances((collection_id, bob_address)),
            transfer_amount
        );
    });
}

#[test]
fn transfer_fungible_failed() {
    new_test_ext().execute_with(|| {
        let alice_address = 1;
        let bob_address = 2;
        let alice = Origin::signed(alice_address);
        CollectionModule::create_collection(alice.clone(), vec![2, 3, 3], true).unwrap();

        let nonce = CollectionModule::get_nonce();
        let collection_id = <CollectionModule as CollectionInterface<_, _>>::generate_collection_id(nonce).unwrap();
        let mint_amount = 5;

        assert_ok!(NFTModule::mint_fungible(
            alice.clone(),
            alice_address,
            collection_id,
            mint_amount
        ));

        let transfer_amount = 0;

        assert_noop!(
            NFTModule::transfer_fungible(
                alice.clone(),
                bob_address,
                collection_id,
                transfer_amount
            ),
            Error::<Test>::AmountLessThanOne
        );
        let not_available_collection_id =
            CollectionModule::generate_collection_id(nonce + 1).unwrap();
        let transfer_amount = 5;

        assert_noop!(
            NFTModule::transfer_fungible(
                alice.clone(),
                bob_address,
                not_available_collection_id,
                transfer_amount
            ),
            Error::<Test>::CollectionNotFound
        );
        assert_noop!(
            NFTModule::transfer_fungible(
                alice.clone(),
                alice_address,
                not_available_collection_id,
                transfer_amount
            ),
            Error::<Test>::ReceiverIsSender
        );
        let transfer_amount = 20;
        assert_noop!(
            NFTModule::transfer_fungible(
                alice,
                bob_address,
                collection_id,
                transfer_amount
            ),
            Error::<Test>::AmountTooLarge
        );
    });
}

#[test]
fn burn_non_fungible_success() {
    new_test_ext().execute_with(|| {
        let alice_address = 1;
        let alice = Origin::signed(alice_address);
        CollectionModule::create_collection(alice.clone(), vec![2, 3, 3], false).unwrap();

        let nonce = CollectionModule::get_nonce();
        let collection_id = <CollectionModule as CollectionInterface<_, _>>::generate_collection_id(nonce).unwrap();
        let mint_amount = 5;
        NFTModule::mint_non_fungible(
            alice.clone(),
            alice_address,
            collection_id,
            vec![2, 3, 3],
            mint_amount,
        )
        .unwrap();
        let last_token_id = NFTModule::last_token_id(collection_id);

        assert_eq!(
            NFTModule::address_balances((collection_id, alice_address)),
            mint_amount
        );

        let burn_amount = 2;
        let start_idx = mint_amount - last_token_id - 1;

        assert_ok!(NFTModule::burn_non_fungible(
            alice,
            collection_id,
            start_idx,
            burn_amount
        ));
        assert_eq!(
            NFTModule::address_balances((collection_id, alice_address)),
            mint_amount - burn_amount
        );

        let collection = CollectionModule::collections(collection_id);
        assert_eq!(collection.total_supply, mint_amount - burn_amount);
        assert_eq!(NFTModule::burned_tokens(collection_id), burn_amount);

        assert_eq!(NFTModule::tokens(collection_id, 2).owner, alice_address);
    });
}

#[test]
fn burn_non_fungible_failed() {
    new_test_ext().execute_with(|| {
        let alice_address = 1;
        let alice = Origin::signed(alice_address);
        CollectionModule::create_collection(alice.clone(), vec![2, 3, 3], false).unwrap();

        let nonce = CollectionModule::get_nonce();
        let collection_id = <CollectionModule as CollectionInterface<_, _>>::generate_collection_id(nonce).unwrap();
        let mint_amount = 5;
        NFTModule::mint_non_fungible(
            alice.clone(),
            alice_address,
            collection_id,
            vec![2, 3, 3],
            mint_amount,
        )
        .unwrap();
        let last_token_id = NFTModule::last_token_id(collection_id);
        let not_available_token_id = last_token_id + 1;
        let not_available_collection_id =
            CollectionModule::generate_collection_id(nonce + 1).unwrap();

        assert_eq!(
            NFTModule::address_balances((collection_id, alice_address)),
            mint_amount
        );

        let burn_amount = 2;
        let start_idx = mint_amount - last_token_id - 1;

        assert_noop!(
            NFTModule::burn_non_fungible(
                alice.clone(),
                collection_id,
                not_available_token_id,
                burn_amount
            ),
            Error::<Test>::TokenNotFound
        );
        assert_noop!(
            NFTModule::burn_non_fungible(
                alice.clone(),
                not_available_collection_id,
                start_idx,
                burn_amount
            ),
            Error::<Test>::CollectionNotFound
        );

        let burn_amount = 10;

        assert_noop!(
            NFTModule::burn_non_fungible(alice, collection_id, start_idx, burn_amount),
            Error::<Test>::AmountTooLarge
        );
    });
}

#[test]
fn burn_fungible_success() {
    new_test_ext().execute_with(|| {
        let alice_address = 1;
        let alice = Origin::signed(alice_address);
        CollectionModule::create_collection(alice.clone(), vec![2, 3, 3], true).unwrap();

        let nonce = CollectionModule::get_nonce();
        let collection_id = <CollectionModule as CollectionInterface<_, _>>::generate_collection_id(nonce).unwrap();
        let mint_amount = 5;
        NFTModule::mint_fungible(alice.clone(), alice_address, collection_id, mint_amount).unwrap();

        assert_eq!(
            NFTModule::address_balances((collection_id, alice_address)),
            mint_amount
        );

        let burn_amount = 2;
        assert_ok!(NFTModule::burn_fungible(
            alice,
            collection_id,
            burn_amount
        ));

        assert_eq!(
            NFTModule::address_balances((collection_id, alice_address)),
            mint_amount - burn_amount
        );

        let collection = CollectionModule::collections(collection_id);
        assert_eq!(collection.total_supply, mint_amount - burn_amount);
        assert_eq!(NFTModule::burned_tokens(collection_id), burn_amount);
    });
}

#[test]
fn burn_fungible_failed() {
    new_test_ext().execute_with(|| {
        let alice_address = 1;
        let alice = Origin::signed(alice_address);
        CollectionModule::create_collection(alice.clone(), vec![2, 3, 3], true).unwrap();

        let nonce = CollectionModule::get_nonce();
        let collection_id = <CollectionModule as CollectionInterface<_, _>>::generate_collection_id(nonce).unwrap();
        let mint_amount = 5;
        NFTModule::mint_fungible(alice.clone(), alice_address, collection_id, mint_amount).unwrap();
        let not_available_collection_id =
            CollectionModule::generate_collection_id(nonce + 1).unwrap();

        assert_eq!(
            NFTModule::address_balances((collection_id, alice_address)),
            mint_amount
        );

        let burn_amount = 2;

        assert_noop!(
            NFTModule::burn_fungible(alice.clone(), not_available_collection_id, burn_amount),
            Error::<Test>::CollectionNotFound
        );

        let burn_amount = 10;
        assert_noop!(
            NFTModule::burn_fungible(alice, collection_id, burn_amount),
            Error::<Test>::AmountTooLarge
        );
    });
}