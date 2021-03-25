use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};
use pallet_nft;
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
        let collection_id = CollectionModule::generate_collection_id(nonce).unwrap();
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
        let collection_id = CollectionModule::generate_collection_id(nonce).unwrap();
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
        let collection_id = CollectionModule::generate_collection_id(nonce).unwrap();
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

        assert_ok!(SubModule::create(alice.clone(), collection_id, start_idx, false));

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
        let collection_id = CollectionModule::generate_collection_id(nonce).unwrap();
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

        assert_ok!(SubModule::create(alice.clone(), collection_id, start_idx, false));

        let nonce = CollectionModule::get_nonce();
        let sub_token_collection_id = CollectionModule::generate_collection_id(nonce).unwrap();

        let not_available_collection_id =
            CollectionModule::generate_collection_id(nonce + 1).unwrap();

        assert_noop!(SubModule::recover(alice.clone(), not_available_collection_id), Error::<Test>::CollectionNotFound);
        assert_noop!(SubModule::recover(alice.clone(), collection_id), Error::<Test>::SubTokenNotFound);
        assert_noop!(SubModule::recover(bob, sub_token_collection_id), Error::<Test>::PermissionDenied);
        
    });
}

// #[test]
// fn transfer_to_pallet_sub() {
//     new_test_ext().execute_with(|| {
//         let alice_address = 1;
//         let alice = Origin::signed(alice_address);

//         assert_ok!(NFTModule::create_collection(alice.clone(), vec![2, 3, 3]));

//         let last_collection_id = NFTModule::last_collection_id();

//         assert_ok!(NFTModule::mint(
//             alice.clone(),
//             last_collection_id,
//             vec![2, 3, 3]
//         ));

//         let last_token_id = NFTModule::last_token_id(last_collection_id);

//         let pallet_sub_address = SubModule::account_id();

//         assert_ok!(NFTModule::transfer(
//             alice.clone(),
//             pallet_sub_address,
//             last_collection_id,
//             last_token_id
//         ));
//         let token = NFTModule::tokens((last_collection_id, last_token_id));
//         assert_eq!(token.owner, pallet_sub_address);
//     });
// }

// #[test]
// fn lock() {
//     new_test_ext().execute_with(|| {
//         let alice_address = 1;
//         let alice = Origin::signed(alice_address);
//         let bob_address = 2;
//         let bob = Origin::signed(bob_address);

//         assert_ok!(NFTModule::create_collection(alice.clone(), vec![2, 3, 3]));

//         let last_collection_id = NFTModule::last_collection_id();

//         assert_ok!(NFTModule::mint(
//             alice.clone(),
//             last_collection_id,
//             vec![2, 3, 3]
//         ));

//         let last_token_id = NFTModule::last_token_id(last_collection_id);

//         let not_available_collection_id = last_collection_id + 1;
//         let not_available_token_id = last_token_id + 1;

//         assert_noop!(
//             SubModule::lock(
//                 alice.clone(),
//                 not_available_collection_id,
//                 last_token_id,
//                 true
//             ),
//             Error::<Test>::CollectionNotFound
//         );

//         assert_noop!(
//             SubModule::lock(
//                 alice.clone(),
//                 last_collection_id,
//                 not_available_token_id,
//                 true
//             ),
//             Error::<Test>::TokenNotFound
//         );

//         assert_noop!(
//             SubModule::lock(bob, last_collection_id, last_token_id, true),
//             Error::<Test>::PermissionDenied
//         );

//         assert_ok!(SubModule::lock(
//             alice,
//             last_collection_id,
//             last_token_id,
//             true
//         ));

//         let sub_token = SubModule::locked_tokens((last_collection_id, last_token_id));
//         assert_eq!(sub_token.owner, alice_address);
//     });
// }

// #[test]
// fn mint_non_fungible_token() {
//     new_test_ext().execute_with(|| {
//         let alice_address = 1;
//         let alice = Origin::signed(alice_address);

//         let bob_address = 2;
//         let bob = Origin::signed(bob_address);

//         assert_ok!(NFTModule::create_collection(alice.clone(), vec![2, 3, 3]));

//         let last_collection_id = NFTModule::last_collection_id();

//         assert_ok!(NFTModule::mint(
//             alice.clone(),
//             last_collection_id,
//             vec![2, 3, 3]
//         ));

//         let last_token_id = NFTModule::last_token_id(last_collection_id);

//         assert_ok!(SubModule::lock(
//             alice.clone(),
//             last_collection_id,
//             last_token_id,
//             true
//         ));

//         let not_available_collection_id = last_collection_id + 1;
//         let not_available_token_id = last_token_id + 1;
//         let mint_amount = 0;

//         assert_noop!(
//             SubModule::mint_non_fungible(
//                 alice.clone(),
//                 last_collection_id,
//                 last_token_id,
//                 mint_amount,
//                 vec![2, 3, 3]
//             ),
//             Error::<Test>::AmountLessThanOne
//         );

//         let mint_amount = 1;

//         assert_noop!(
//             SubModule::mint_fungible(
//                 alice.clone(),
//                 last_collection_id,
//                 last_token_id,
//                 mint_amount
//             ),
//             Error::<Test>::WrongTokenType
//         );

//         assert_noop!(
//             SubModule::mint_non_fungible(
//                 alice.clone(),
//                 not_available_collection_id,
//                 last_token_id,
//                 mint_amount,
//                 vec![2, 3, 3]
//             ),
//             Error::<Test>::TokenNotFound
//         );

//         assert_noop!(
//             SubModule::mint_non_fungible(
//                 alice.clone(),
//                 last_collection_id,
//                 not_available_token_id,
//                 mint_amount,
//                 vec![2, 3, 3]
//             ),
//             Error::<Test>::TokenNotFound
//         );

//         assert_noop!(
//             SubModule::mint_non_fungible(
//                 bob,
//                 last_collection_id,
//                 last_token_id,
//                 mint_amount,
//                 vec![2, 3, 3]
//             ),
//             Error::<Test>::PermissionDenied
//         );

//         assert_ok!(SubModule::mint_non_fungible(
//             alice.clone(),
//             last_collection_id,
//             last_token_id,
//             mint_amount,
//             vec![2, 3, 3]
//         ));

//         let balance =
//             SubModule::address_balances((last_collection_id, last_token_id), alice_address);
//         assert_eq!(balance, mint_amount);

//         let sub_token = SubModule::sub_tokens((last_collection_id, last_token_id), last_token_id);

//         assert_eq!(sub_token.owner, alice_address);

//         let next_start_idx = SubModule::last_token_id((last_collection_id, last_token_id)) + 1;
//         let mint_amount = 5;

//         assert_ok!(SubModule::mint_non_fungible(
//             alice.clone(),
//             last_collection_id,
//             last_token_id,
//             mint_amount,
//             vec![2, 3, 3]
//         ));

//         assert_eq!(
//             SubModule::address_balances((last_collection_id, last_token_id), alice_address),
//             balance + mint_amount
//         );
//         let sub_token = SubModule::sub_tokens((last_collection_id, last_token_id), next_start_idx);
//         assert_eq!(sub_token.owner, alice_address);
//     });
// }

// #[test]
// fn transfer_non_fungible() {
//     new_test_ext().execute_with(|| {
//         let alice_address = 1;
//         let alice = Origin::signed(alice_address);

//         let bob_address = 2;
//         let bob = Origin::signed(bob_address);

//         assert_ok!(NFTModule::create_collection(alice.clone(), vec![2, 3, 3]));

//         let last_collection_id = NFTModule::last_collection_id();

//         assert_ok!(NFTModule::mint(
//             alice.clone(),
//             last_collection_id,
//             vec![2, 3, 3]
//         ));

//         let last_token_id = NFTModule::last_token_id(last_collection_id);

//         assert_ok!(SubModule::lock(
//             alice.clone(),
//             last_collection_id,
//             last_token_id,
//             true
//         ));

//         let mint_amount = 5;

//         assert_ok!(SubModule::mint_non_fungible(
//             alice.clone(),
//             last_collection_id,
//             last_token_id,
//             mint_amount,
//             vec![1, 2, 3]
//         ));

//         let last_sub_token_idx = SubModule::last_token_id((last_collection_id, last_token_id));

//         let sub_start_idx = last_sub_token_idx + 1 - mint_amount;

//         let transfer_amount = 1;

//         assert_noop!(SubModule::transfer_non_fungible(bob, alice_address, last_collection_id, last_token_id, sub_start_idx, transfer_amount), Error::<Test>::PermissionDenied);

//         assert_ok!(SubModule::transfer_non_fungible(
//             alice.clone(),
//             bob_address,
//             last_collection_id,
//             last_token_id,
//             sub_start_idx,
//             transfer_amount
//         ));

//         let bob_sub_token = SubModule::sub_tokens((last_collection_id, last_token_id), sub_start_idx);
//         assert_eq!(bob_sub_token.owner, bob_address);

//         let alice_sub_start_idx = sub_start_idx + transfer_amount;
//         let alice_sub_token = SubModule::sub_tokens((last_collection_id, last_token_id), alice_sub_start_idx);
//         assert_eq!(alice_sub_token.owner, alice_address);

//         assert_eq!(SubModule::address_balances((last_collection_id, last_token_id), alice_address), mint_amount - transfer_amount);
//         assert_eq!(SubModule::address_balances((last_collection_id, last_token_id), bob_address),  transfer_amount);
//     });
// }

// #[test]
// fn transfer_fungible() {
//     new_test_ext().execute_with(|| {
//         let alice_address = 1;
//         let alice = Origin::signed(alice_address);

//         let bob_address = 2;
//         let bob = Origin::signed(bob_address);

//         assert_ok!(NFTModule::create_collection(alice.clone(), vec![2, 3, 3]));

//         let last_collection_id = NFTModule::last_collection_id();

//         assert_ok!(NFTModule::mint(
//             alice.clone(),
//             last_collection_id,
//             vec![2, 3, 3]
//         ));

//         let last_token_id = NFTModule::last_token_id(last_collection_id);

//         assert_ok!(SubModule::lock(
//             alice.clone(),
//             last_collection_id,
//             last_token_id,
//             false
//         ));

//         let transfer_amount = 1;

//         assert_noop!(
//             SubModule::transfer_fungible(
//                 alice.clone(),
//                 bob_address,
//                 last_collection_id,
//                 last_token_id,
//                 transfer_amount
//             ),
//             Error::<Test>::AmountTooLarge
//         );

//         let mint_amount = 5;

//         assert_ok!(SubModule::mint_fungible(
//             alice.clone(),
//             last_collection_id,
//             last_token_id,
//             mint_amount,
//         ));

//         assert_noop!(SubModule::transfer_fungible(
//             bob,
//             alice_address,
//             last_collection_id,
//             last_token_id,
//             transfer_amount
//         ), Error::<Test>::AmountTooLarge);

//         assert_ok!(SubModule::transfer_fungible(
//             alice.clone(),
//             bob_address,
//             last_collection_id,
//             last_token_id,
//             transfer_amount
//         ));

//         assert_eq!(SubModule::address_balances((last_collection_id, last_token_id), alice_address), mint_amount - transfer_amount);
//         assert_eq!(SubModule::address_balances((last_collection_id, last_token_id), bob_address),  transfer_amount);

//     });
// }

// #[test]
// fn burn_non_fungible() {
//     new_test_ext().execute_with(|| {
//         let alice_address = 1;
//         let alice = Origin::signed(alice_address);
//         let bob_address = 2;
//         let bob = Origin::signed(bob_address);

//         assert_ok!(NFTModule::create_collection(alice.clone(), vec![2, 3, 3]));

//         let last_collection_id = NFTModule::last_collection_id();

//         assert_ok!(NFTModule::mint(
//             alice.clone(),
//             last_collection_id,
//             vec![2, 3, 3]
//         ));

//         let last_token_id = NFTModule::last_token_id(last_collection_id);

//         assert_ok!(SubModule::lock(
//             alice.clone(),
//             last_collection_id,
//             last_token_id,
//             true
//         ));

//         let mint_amount = 5;

//         assert_ok!(SubModule::mint_non_fungible(
//             alice.clone(),
//             last_collection_id,
//             last_token_id,
//             mint_amount,
//             vec![1, 2, 3]
//         ));

//         let last_sub_token_idx = SubModule::last_token_id((last_collection_id, last_token_id));

//         let sub_start_idx = last_sub_token_idx + 1 - mint_amount;

//         let burn_amount = 1;

//         assert_noop!(SubModule::burn_non_fungible(
//             bob,
//             last_collection_id,
//             last_token_id,
//             sub_start_idx,
//             burn_amount
//         ), Error::<Test>::PermissionDenied);

//         assert_ok!(SubModule::burn_non_fungible(
//             alice.clone(),
//             last_collection_id,
//             last_token_id,
//             sub_start_idx,
//             burn_amount
//         ));

//         assert_eq!(SubModule::address_balances((last_collection_id, last_token_id), alice_address), mint_amount - burn_amount);

//         let next_start_idx = last_token_id + burn_amount;
//         let sub_token = SubModule::sub_tokens((last_collection_id, last_token_id), next_start_idx);

//         assert_eq!(sub_token.owner, alice_address);
//         assert_eq!(SubModule::burned_sub_tokens((last_collection_id, last_token_id)), burn_amount);
//     });
// }

// #[test]
// fn burn_fungible() {
//     new_test_ext().execute_with(|| {
//         let alice_address = 1;
//         let alice = Origin::signed(alice_address);
//         let bob_address = 2;
//         let bob = Origin::signed(bob_address);

//         assert_ok!(NFTModule::create_collection(alice.clone(), vec![2, 3, 3]));

//         let last_collection_id = NFTModule::last_collection_id();

//         assert_ok!(NFTModule::mint(
//             alice.clone(),
//             last_collection_id,
//             vec![2, 3, 3]
//         ));

//         let last_token_id = NFTModule::last_token_id(last_collection_id);

//         assert_ok!(SubModule::lock(
//             alice.clone(),
//             last_collection_id,
//             last_token_id,
//             false
//         ));

//         let mint_amount = 5;

//         assert_ok!(SubModule::mint_fungible(
//             alice.clone(),
//             last_collection_id,
//             last_token_id,
//             mint_amount
//         ));

//         let burn_amount = 1;

//         assert_noop!(SubModule::burn_fungible(
//             bob,
//             last_collection_id,
//             last_token_id,
//             burn_amount
//         ), Error::<Test>::AmountTooLarge);

//         assert_ok!(SubModule::burn_fungible(
//             alice.clone(),
//             last_collection_id,
//             last_token_id,
//             burn_amount
//         ));

//         assert_eq!(SubModule::address_balances((last_collection_id, last_token_id), alice_address), mint_amount - burn_amount);
//         assert_eq!(SubModule::burned_sub_tokens((last_collection_id, last_token_id)), burn_amount);
//     });
// }

// #[test]
// fn unlock() {
//     new_test_ext().execute_with(|| {
//         let alice_address = 1;
//         let alice = Origin::signed(alice_address);

//         assert_ok!(NFTModule::create_collection(alice.clone(), vec![2, 3, 3]));

//         let last_collection_id = NFTModule::last_collection_id();

//         assert_ok!(NFTModule::mint(
//             alice.clone(),
//             last_collection_id,
//             vec![2, 3, 3]
//         ));

//         let last_token_id = NFTModule::last_token_id(last_collection_id);

//         assert_ok!(SubModule::lock(
//             alice.clone(),
//             last_collection_id,
//             last_token_id,
//             false
//         ));

//         let mint_amount = 5;

//         assert_ok!(SubModule::mint_fungible(
//             alice.clone(),
//             last_collection_id,
//             last_token_id,
//             mint_amount
//         ));

//         let token = NFTModule::tokens((last_collection_id, last_token_id));
//         assert_eq!(token.owner, SubModule::account_id());

//         assert_ok!(SubModule::unlock(alice.clone(), last_collection_id, last_token_id));

//         let token = NFTModule::tokens((last_collection_id, last_token_id));
//         assert_eq!(token.owner, alice_address);
//     });
// }
