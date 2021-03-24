use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

#[test]
fn mint_success() {
    new_test_ext().execute_with(|| {
        let alice_address = 1;
        let alice = Origin::signed(alice_address);
        CollectionModule::create_collection(alice.clone(), vec![2, 3, 3]).unwrap();

        let nonce = CollectionModule::get_nonce();
        let collection_id = CollectionModule::generate_collection_id(nonce).unwrap();
        NFTModule::mint(alice.clone(), collection_id, vec![2, 3, 3]).unwrap();

        let last_token_id = NFTModule::last_token_id(collection_id);
        let token = NFTModule::tokens((collection_id, last_token_id));

        assert_eq!(token.end_idx, 0);
        assert_eq!(token.owner, alice_address);
        assert_eq!(token.uri, vec![2, 3, 3]);
        assert_eq!(
            NFTModule::address_balances((collection_id, alice_address)),
            1
        );

        let collection = CollectionModule::collections(collection_id);

        assert_eq!(collection.total_supply, 1)
    });
}

#[test]
fn mint_failed() {
    new_test_ext().execute_with(|| {
        let alice_address = 1;
        let alice = Origin::signed(alice_address);

        CollectionModule::create_collection(alice.clone(), vec![2, 3, 3]).unwrap();

        let nonce = CollectionModule::get_nonce();
        // let collection_id = CollectionModule::generate_collection_id(nonce).unwrap();
        let not_available_collection_id =
            CollectionModule::generate_collection_id(nonce + 1).unwrap();

        assert_noop!(
            NFTModule::mint(alice.clone(), not_available_collection_id, vec![2, 3, 3]),
            Error::<Test>::CollectionNotFound
        );
    });
}

#[test]
fn batch_mint_success() {
    new_test_ext().execute_with(|| {
        let alice_address = 1;
        let alice = Origin::signed(alice_address);
        CollectionModule::create_collection(alice.clone(), vec![2, 3, 3]).unwrap();

        let nonce = CollectionModule::get_nonce();
        let collection_id = CollectionModule::generate_collection_id(nonce).unwrap();
        let mint_amount = 5;

        assert_ok!(NFTModule::batch_mint(
            alice.clone(),
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
        let token = NFTModule::tokens((collection_id, start_idx));

        assert_eq!(token.end_idx, mint_amount - 1);
        assert_eq!(
            NFTModule::address_balances((collection_id, alice_address)),
            mint_amount
        );

        let collection = CollectionModule::collections(collection_id);

        assert_eq!(collection.total_supply, mint_amount)
    });
}

#[test]
fn batch_mint_failed() {
    new_test_ext().execute_with(|| {
        let alice_address = 1;
        let alice = Origin::signed(alice_address);
        CollectionModule::create_collection(alice.clone(), vec![2, 3, 3]).unwrap();

        let nonce = CollectionModule::get_nonce();
        let collection_id = CollectionModule::generate_collection_id(nonce).unwrap();
        let mint_amount = 0;

        assert_noop!(
            NFTModule::batch_mint(alice.clone(), collection_id, vec![2, 3, 3], mint_amount,),
            Error::<Test>::AmountLessThanOne
        );

        let mint_amount = 5;
        let not_available_collection_id =
            CollectionModule::generate_collection_id(nonce + 1).unwrap();

        assert_noop!(
            NFTModule::batch_mint(
                alice.clone(),
                not_available_collection_id,
                vec![2, 3, 3],
                mint_amount,
            ),
            Error::<Test>::CollectionNotFound
        );
    });
}

#[test]
fn transfer_success() {
    new_test_ext().execute_with(|| {
        let alice_address = 1;
        let bob_address = 2;
        let alice = Origin::signed(alice_address);
        CollectionModule::create_collection(alice.clone(), vec![2, 3, 3]).unwrap();

        let nonce = CollectionModule::get_nonce();
        let collection_id = CollectionModule::generate_collection_id(nonce).unwrap();
        let last_token_id = NFTModule::last_token_id(collection_id);
        NFTModule::mint(alice.clone(), collection_id, vec![2, 3, 3]).unwrap();

        assert_ok!(NFTModule::transfer(
            alice.clone(),
            bob_address,
            collection_id,
            last_token_id
        ));

        let token = NFTModule::tokens((collection_id, last_token_id));
        assert_eq!(token.owner, bob_address);

        assert_eq!(
            NFTModule::address_balances((collection_id, alice_address)),
            0
        );
        assert_eq!(NFTModule::address_balances((collection_id, bob_address)), 1);
    });
}

#[test]
fn transfer_failed() {
    new_test_ext().execute_with(|| {
        let alice_address = 1;
        let bob_address = 2;
        let alice = Origin::signed(alice_address);
        CollectionModule::create_collection(alice.clone(), vec![2, 3, 3]).unwrap();

        let nonce = CollectionModule::get_nonce();
        let collection_id = CollectionModule::generate_collection_id(nonce).unwrap();
        let last_token_id = NFTModule::last_token_id(collection_id);
        NFTModule::mint(alice.clone(), collection_id, vec![2, 3, 3]).unwrap();
        let not_available_token_id = last_token_id + 1;
        let not_available_collection_id =
            CollectionModule::generate_collection_id(nonce + 1).unwrap();

        assert_noop!(
            NFTModule::transfer(
                alice.clone(),
                bob_address,
                collection_id,
                not_available_token_id
            ),
            Error::<Test>::TokenNotFound
        );
        assert_noop!(
            NFTModule::transfer(
                alice.clone(),
                bob_address,
                not_available_collection_id,
                last_token_id
            ),
            Error::<Test>::CollectionNotFound
        );
        assert_noop!(
            NFTModule::transfer(alice.clone(), alice_address, collection_id, last_token_id),
            Error::<Test>::ReceiverIsSender
        );
    });
}

#[test]
fn batch_transfer_success() {
    new_test_ext().execute_with(|| {
        let alice_address = 1;
        let bob_address = 2;
        let alice = Origin::signed(alice_address);
        CollectionModule::create_collection(alice.clone(), vec![2, 3, 3]).unwrap();

        let nonce = CollectionModule::get_nonce();
        let collection_id = CollectionModule::generate_collection_id(nonce).unwrap();
        let mint_amount = 5;
        NFTModule::batch_mint(alice.clone(), collection_id, vec![2, 3, 3], mint_amount).unwrap();
        let last_token_id = NFTModule::last_token_id(collection_id);
        let start_idx = mint_amount - last_token_id - 1;
        let token = NFTModule::tokens((collection_id, start_idx));

        assert_eq!(token.owner, alice_address);
        assert_eq!(
            NFTModule::address_balances((collection_id, alice_address)),
            mint_amount
        );

        let transfer_amount = 2;
        assert_ok!(NFTModule::batch_transfer(
            alice.clone(),
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
        let bob_nfts = NFTModule::tokens((collection_id, start_idx));
        let alice_nfts = NFTModule::tokens((collection_id, alice_start_idx));

        assert_eq!(bob_nfts.owner, bob_address);
        assert_eq!(alice_nfts.owner, alice_address);

        assert_eq!(alice_nfts.uri, bob_nfts.uri);
    });
}

#[test]
fn batch_transfer_failed() {
    new_test_ext().execute_with(|| {
        let alice_address = 1;
        let bob_address = 2;
        let alice = Origin::signed(alice_address);
        CollectionModule::create_collection(alice.clone(), vec![2, 3, 3]).unwrap();

        let nonce = CollectionModule::get_nonce();
        let collection_id = CollectionModule::generate_collection_id(nonce).unwrap();
        let mint_amount = 5;
        NFTModule::batch_mint(alice.clone(), collection_id, vec![2, 3, 3], mint_amount).unwrap();
        let last_token_id = NFTModule::last_token_id(collection_id);
        let start_idx = mint_amount - last_token_id - 1;
        let transfer_amount = 0;

        assert_noop!(
            NFTModule::batch_transfer(
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
            NFTModule::batch_transfer(
                alice.clone(),
                bob_address,
                collection_id,
                not_available_token_id,
                transfer_amount
            ),
            Error::<Test>::TokenNotFound
        );
        assert_noop!(
            NFTModule::batch_transfer(
                alice.clone(),
                bob_address,
                not_available_collection_id,
                start_idx,
                transfer_amount
            ),
            Error::<Test>::CollectionNotFound
        );
        assert_noop!(
            NFTModule::batch_transfer(
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
            NFTModule::batch_transfer(
                alice.clone(),
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
fn burn_success() {
    new_test_ext().execute_with(|| {
        let alice_address = 1;
        let alice = Origin::signed(alice_address);
        CollectionModule::create_collection(alice.clone(), vec![2, 3, 3]).unwrap();

        let nonce = CollectionModule::get_nonce();
        let collection_id = CollectionModule::generate_collection_id(nonce).unwrap();
        NFTModule::mint(alice.clone(), collection_id, vec![2, 3, 3]).unwrap();
        let last_token_id = NFTModule::last_token_id(collection_id);

        assert_eq!(
            NFTModule::address_balances((collection_id, alice_address)),
            1
        );
        assert_ok!(NFTModule::burn(alice.clone(), collection_id, last_token_id));
        assert_eq!(
            NFTModule::address_balances((collection_id, alice_address)),
            0
        );

        let collection = CollectionModule::collections(collection_id);

        assert_eq!(collection.total_supply, 0)
    });
}

#[test]
fn burn_failed() {
    new_test_ext().execute_with(|| {
        let alice_address = 1;
        let alice = Origin::signed(alice_address);
        CollectionModule::create_collection(alice.clone(), vec![2, 3, 3]).unwrap();

        let nonce = CollectionModule::get_nonce();
        let collection_id = CollectionModule::generate_collection_id(nonce).unwrap();
        NFTModule::mint(alice.clone(), collection_id, vec![2, 3, 3]).unwrap();
        let last_token_id = NFTModule::last_token_id(collection_id);
        let not_available_token_id = last_token_id + 1;
        let not_available_collection_id =
            CollectionModule::generate_collection_id(nonce + 1).unwrap();

        assert_noop!(
            NFTModule::burn(alice.clone(), collection_id, not_available_token_id),
            Error::<Test>::TokenNotFound
        );
        assert_noop!(
            NFTModule::burn(alice.clone(), not_available_collection_id, last_token_id),
            Error::<Test>::CollectionNotFound
        );
    });
}

#[test]
fn batch_burn_success() {
    new_test_ext().execute_with(|| {
        let alice_address = 1;
        let alice = Origin::signed(alice_address);
        CollectionModule::create_collection(alice.clone(), vec![2, 3, 3]).unwrap();

        let nonce = CollectionModule::get_nonce();
        let collection_id = CollectionModule::generate_collection_id(nonce).unwrap();
        let mint_amount = 5;
        NFTModule::batch_mint(alice.clone(), collection_id, vec![2, 3, 3], mint_amount).unwrap();
        let last_token_id = NFTModule::last_token_id(collection_id);

        assert_eq!(
            NFTModule::address_balances((collection_id, alice_address)),
            mint_amount
        );

        let burn_amount = 2;
        let start_idx = mint_amount - last_token_id - 1;

        assert_ok!(NFTModule::batch_burn(
            alice.clone(),
            collection_id,
            start_idx,
            burn_amount
        ));
        assert_eq!(
            NFTModule::address_balances((collection_id, alice_address)),
            mint_amount - burn_amount
        );

        let collection = CollectionModule::collections(collection_id);
        assert_eq!(collection.total_supply, mint_amount - burn_amount)
    });
}

#[test]
fn batch_burn_failed() {
    new_test_ext().execute_with(|| {
        let alice_address = 1;
        let alice = Origin::signed(alice_address);
        CollectionModule::create_collection(alice.clone(), vec![2, 3, 3]).unwrap();

        let nonce = CollectionModule::get_nonce();
        let collection_id = CollectionModule::generate_collection_id(nonce).unwrap();
        let mint_amount = 5;
        NFTModule::batch_mint(alice.clone(), collection_id, vec![2, 3, 3], mint_amount).unwrap();
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
            NFTModule::batch_burn(
                alice.clone(),
                collection_id,
                not_available_token_id,
                burn_amount
            ),
            Error::<Test>::TokenNotFound
        );
        assert_noop!(
            NFTModule::batch_burn(
                alice.clone(),
                not_available_collection_id,
                start_idx,
                burn_amount
            ),
            Error::<Test>::CollectionNotFound
        );

        let burn_amount = 10;

        assert_noop!(
            NFTModule::batch_burn(alice.clone(), collection_id, start_idx, burn_amount),
            Error::<Test>::AmountTooLarge
        );
    });
}
