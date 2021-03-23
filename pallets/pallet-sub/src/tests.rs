use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

#[test]
fn it_works_for_pallet_nft() {
    new_test_ext().execute_with(|| {
        let alice = Origin::signed(1);
        assert_ok!(NFTModule::create_collection(alice, vec![2, 3, 3]));
    });
}

#[test]
fn transfer_to_pallet_sub() {
    new_test_ext().execute_with(|| {
        let alice_address = 1;
        let alice = Origin::signed(alice_address);

        assert_ok!(NFTModule::create_collection(alice.clone(), vec![2, 3, 3]));

        let last_collection_id = NFTModule::last_collection_id();

        assert_ok!(NFTModule::mint(
            alice.clone(),
            last_collection_id,
            vec![2, 3, 3]
        ));

        let last_token_id = NFTModule::last_token_id(last_collection_id);

        let pallet_sub_address = SubModule::account_id();

        assert_ok!(NFTModule::transfer(
            alice.clone(),
            pallet_sub_address,
            last_collection_id,
            last_token_id
        ));
        let token = NFTModule::tokens((last_collection_id, last_token_id));
        assert_eq!(token.owner, pallet_sub_address);
    });
}

#[test]
fn lock() {
    new_test_ext().execute_with(|| {
        let alice_address = 1;
        let alice = Origin::signed(alice_address);
        let bob_address = 2;
        let bob = Origin::signed(bob_address);

        assert_ok!(NFTModule::create_collection(alice.clone(), vec![2, 3, 3]));

        let last_collection_id = NFTModule::last_collection_id();

        assert_ok!(NFTModule::mint(
            alice.clone(),
            last_collection_id,
            vec![2, 3, 3]
        ));

        let last_token_id = NFTModule::last_token_id(last_collection_id);

        let not_available_collection_id = last_collection_id + 1;
        let not_available_token_id = last_token_id + 1;

        assert_noop!(
            SubModule::lock(
                alice.clone(),
                not_available_collection_id,
                last_token_id,
                true
            ),
            Error::<Test>::CollectionNotFound
        );

        assert_noop!(
            SubModule::lock(
                alice.clone(),
                last_collection_id,
                not_available_token_id,
                true
            ),
            Error::<Test>::TokenNotFound
        );

        assert_noop!(
            SubModule::lock(bob, last_collection_id, last_token_id, true),
            Error::<Test>::PermissionDenied
        );

        assert_ok!(SubModule::lock(
            alice,
            last_collection_id,
            last_token_id,
            true
        ));

        let sub_token = SubModule::locked_tokens((last_collection_id, last_token_id));
        assert_eq!(sub_token.owner, alice_address);
    });
}

#[test]
fn mint_non_fungible_token() {
    new_test_ext().execute_with(|| {
        let alice_address = 1;
        let alice = Origin::signed(alice_address);

        let bob_address = 2;
        let bob = Origin::signed(bob_address);

        assert_ok!(NFTModule::create_collection(alice.clone(), vec![2, 3, 3]));

        let last_collection_id = NFTModule::last_collection_id();

        assert_ok!(NFTModule::mint(
            alice.clone(),
            last_collection_id,
            vec![2, 3, 3]
        ));

        let last_token_id = NFTModule::last_token_id(last_collection_id);

        assert_ok!(SubModule::lock(
            alice.clone(),
            last_collection_id,
            last_token_id,
            true
        ));

        let not_available_collection_id = last_collection_id + 1;
        let not_available_token_id = last_token_id + 1;
        let mint_amount = 0;

        assert_noop!(
            SubModule::mint_non_fungible(
                alice.clone(),
                last_collection_id,
                last_token_id,
                mint_amount,
                vec![2, 3, 3]
            ),
            Error::<Test>::AmountLessThanOne
        );

        let mint_amount = 1;

        assert_noop!(
            SubModule::mint_fungible(
                alice.clone(),
                last_collection_id,
                last_token_id,
                mint_amount
            ),
            Error::<Test>::WrongTokenType
        );

        assert_noop!(
            SubModule::mint_non_fungible(
                alice.clone(),
                not_available_collection_id,
                last_token_id,
                mint_amount,
                vec![2, 3, 3]
            ),
            Error::<Test>::TokenNotFound
        );

        assert_noop!(
            SubModule::mint_non_fungible(
                alice.clone(),
                last_collection_id,
                not_available_token_id,
                mint_amount,
                vec![2, 3, 3]
            ),
            Error::<Test>::TokenNotFound
        );

        assert_noop!(
            SubModule::mint_non_fungible(
                bob,
                last_collection_id,
                last_token_id,
                mint_amount,
                vec![2, 3, 3]
            ),
            Error::<Test>::PermissionDenied
        );

        assert_ok!(SubModule::mint_non_fungible(
            alice.clone(),
            last_collection_id,
            last_token_id,
            mint_amount,
            vec![2, 3, 3]
        ));

        let balance =
            SubModule::address_balances((last_collection_id, last_token_id), alice_address);
        assert_eq!(balance, mint_amount);

        let sub_token = SubModule::sub_tokens((last_collection_id, last_token_id), last_token_id);

        assert_eq!(sub_token.owner, alice_address);

        let next_start_idx = SubModule::last_token_id((last_collection_id, last_token_id)) + 1;
        let mint_amount = 5;

        assert_ok!(SubModule::mint_non_fungible(
            alice.clone(),
            last_collection_id,
            last_token_id,
            mint_amount,
            vec![2, 3, 3]
        ));

        assert_eq!(
            SubModule::address_balances((last_collection_id, last_token_id), alice_address),
            balance + mint_amount
        );
        let sub_token = SubModule::sub_tokens((last_collection_id, last_token_id), next_start_idx);
        assert_eq!(sub_token.owner, alice_address);
    });
}

#[test]
fn transfer_non_fungible() {
    new_test_ext().execute_with(|| {
        let alice_address = 1;
        let alice = Origin::signed(alice_address);

        let bob_address = 2;
        let bob = Origin::signed(bob_address);

        assert_ok!(NFTModule::create_collection(alice.clone(), vec![2, 3, 3]));

        let last_collection_id = NFTModule::last_collection_id();

        assert_ok!(NFTModule::mint(
            alice.clone(),
            last_collection_id,
            vec![2, 3, 3]
        ));

        let last_token_id = NFTModule::last_token_id(last_collection_id);

        assert_ok!(SubModule::lock(
            alice.clone(),
            last_collection_id,
            last_token_id,
            true
        ));

        let mint_amount = 5;

        assert_ok!(SubModule::mint_non_fungible(
            alice.clone(),
            last_collection_id,
            last_token_id,
            mint_amount,
            vec![1, 2, 3]
        ));

        let last_sub_token_idx = SubModule::last_token_id((last_collection_id, last_token_id));

        let sub_start_idx = last_sub_token_idx + 1 - mint_amount;

        let transfer_amount = 1;

        assert_noop!(SubModule::transfer_non_fungible(bob, alice_address, last_collection_id, last_token_id, sub_start_idx, transfer_amount), Error::<Test>::PermissionDenied);

        assert_ok!(SubModule::transfer_non_fungible(
            alice.clone(),
            bob_address,
            last_collection_id,
            last_token_id,
            sub_start_idx,
            transfer_amount
        ));

        let bob_sub_token = SubModule::sub_tokens((last_collection_id, last_token_id), sub_start_idx);
        assert_eq!(bob_sub_token.owner, bob_address);

        let alice_sub_start_idx = sub_start_idx + transfer_amount;
        let alice_sub_token = SubModule::sub_tokens((last_collection_id, last_token_id), alice_sub_start_idx);
        assert_eq!(alice_sub_token.owner, alice_address);

        assert_eq!(SubModule::address_balances((last_collection_id, last_token_id), alice_address), mint_amount - transfer_amount);
        assert_eq!(SubModule::address_balances((last_collection_id, last_token_id), bob_address),  transfer_amount);
    });
}

#[test]
fn transfer_fungible() {
    new_test_ext().execute_with(|| {
        let alice_address = 1;
        let alice = Origin::signed(alice_address);

        let bob_address = 2;
        let bob = Origin::signed(bob_address);

        assert_ok!(NFTModule::create_collection(alice.clone(), vec![2, 3, 3]));

        let last_collection_id = NFTModule::last_collection_id();

        assert_ok!(NFTModule::mint(
            alice.clone(),
            last_collection_id,
            vec![2, 3, 3]
        ));

        let last_token_id = NFTModule::last_token_id(last_collection_id);

        assert_ok!(SubModule::lock(
            alice.clone(),
            last_collection_id,
            last_token_id,
            false
        ));

        let transfer_amount = 1;

        assert_noop!(
            SubModule::transfer_fungible(
                alice.clone(),
                bob_address,
                last_collection_id,
                last_token_id,
                transfer_amount
            ),
            Error::<Test>::AmountTooLarge
        );

        let mint_amount = 5;

        assert_ok!(SubModule::mint_fungible(
            alice.clone(),
            last_collection_id,
            last_token_id,
            mint_amount,
        ));

        assert_noop!(SubModule::transfer_fungible(
            bob,
            alice_address,
            last_collection_id,
            last_token_id,
            transfer_amount
        ), Error::<Test>::AmountTooLarge);

        assert_ok!(SubModule::transfer_fungible(
            alice.clone(),
            bob_address,
            last_collection_id,
            last_token_id,
            transfer_amount
        ));

        assert_eq!(SubModule::address_balances((last_collection_id, last_token_id), alice_address), mint_amount - transfer_amount);
        assert_eq!(SubModule::address_balances((last_collection_id, last_token_id), bob_address),  transfer_amount);

    });
}

#[test]
fn burn_non_fungible() {
    new_test_ext().execute_with(|| {
        let alice_address = 1;
        let alice = Origin::signed(alice_address);
        let bob_address = 2;
        let bob = Origin::signed(bob_address);

        assert_ok!(NFTModule::create_collection(alice.clone(), vec![2, 3, 3]));

        let last_collection_id = NFTModule::last_collection_id();

        assert_ok!(NFTModule::mint(
            alice.clone(),
            last_collection_id,
            vec![2, 3, 3]
        ));

        let last_token_id = NFTModule::last_token_id(last_collection_id);

        assert_ok!(SubModule::lock(
            alice.clone(),
            last_collection_id,
            last_token_id,
            true
        ));

        let mint_amount = 5;

        assert_ok!(SubModule::mint_non_fungible(
            alice.clone(),
            last_collection_id,
            last_token_id,
            mint_amount,
            vec![1, 2, 3]
        ));

        let last_sub_token_idx = SubModule::last_token_id((last_collection_id, last_token_id));

        let sub_start_idx = last_sub_token_idx + 1 - mint_amount;

        let burn_amount = 1;

        assert_noop!(SubModule::burn_non_fungible(
            bob,
            last_collection_id,
            last_token_id,
            sub_start_idx,
            burn_amount
        ), Error::<Test>::PermissionDenied);

        assert_ok!(SubModule::burn_non_fungible(
            alice.clone(),
            last_collection_id,
            last_token_id,
            sub_start_idx,
            burn_amount
        ));

        assert_eq!(SubModule::address_balances((last_collection_id, last_token_id), alice_address), mint_amount - burn_amount);

        let next_start_idx = last_token_id + burn_amount;
        let sub_token = SubModule::sub_tokens((last_collection_id, last_token_id), next_start_idx);

        assert_eq!(sub_token.owner, alice_address);
        assert_eq!(SubModule::burned_sub_tokens((last_collection_id, last_token_id)), burn_amount);
    });
}

#[test]
fn burn_fungible() {
    new_test_ext().execute_with(|| {
        let alice_address = 1;
        let alice = Origin::signed(alice_address);
        let bob_address = 2;
        let bob = Origin::signed(bob_address);

        assert_ok!(NFTModule::create_collection(alice.clone(), vec![2, 3, 3]));

        let last_collection_id = NFTModule::last_collection_id();

        assert_ok!(NFTModule::mint(
            alice.clone(),
            last_collection_id,
            vec![2, 3, 3]
        ));

        let last_token_id = NFTModule::last_token_id(last_collection_id);

        assert_ok!(SubModule::lock(
            alice.clone(),
            last_collection_id,
            last_token_id,
            false
        ));

        let mint_amount = 5;

        assert_ok!(SubModule::mint_fungible(
            alice.clone(),
            last_collection_id,
            last_token_id,
            mint_amount
        ));

        let burn_amount = 1;

        assert_noop!(SubModule::burn_fungible(
            bob,
            last_collection_id,
            last_token_id,
            burn_amount
        ), Error::<Test>::AmountTooLarge);

        assert_ok!(SubModule::burn_fungible(
            alice.clone(),
            last_collection_id,
            last_token_id,
            burn_amount
        ));

        assert_eq!(SubModule::address_balances((last_collection_id, last_token_id), alice_address), mint_amount - burn_amount);
        assert_eq!(SubModule::burned_sub_tokens((last_collection_id, last_token_id)), burn_amount);
    });
}

#[test]
fn unlock() {
    new_test_ext().execute_with(|| {
        let alice_address = 1;
        let alice = Origin::signed(alice_address);
        

        assert_ok!(NFTModule::create_collection(alice.clone(), vec![2, 3, 3]));

        let last_collection_id = NFTModule::last_collection_id();

        assert_ok!(NFTModule::mint(
            alice.clone(),
            last_collection_id,
            vec![2, 3, 3]
        ));

        let last_token_id = NFTModule::last_token_id(last_collection_id);

        assert_ok!(SubModule::lock(
            alice.clone(),
            last_collection_id,
            last_token_id,
            false
        ));

        let mint_amount = 5;

        assert_ok!(SubModule::mint_fungible(
            alice.clone(),
            last_collection_id,
            last_token_id,
            mint_amount
        ));

        let token = NFTModule::tokens((last_collection_id, last_token_id));
        assert_eq!(token.owner, SubModule::account_id());

        assert_ok!(SubModule::unlock(alice.clone(), last_collection_id, last_token_id));

        let token = NFTModule::tokens((last_collection_id, last_token_id));
        assert_eq!(token.owner, alice_address);
    });
}