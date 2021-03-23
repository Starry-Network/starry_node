use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

// #[test]
// fn test_create_collection() {
// 	new_test_ext().execute_with(|| {
// 		let alice = Origin::signed(1);

// 		assert_ok!(NFTModule::create_collection(alice, vec![2, 3, 3]));

// 		let last_collection_id = NFTModule::last_collection_id();

// 		assert_eq!(last_collection_id, 0);

// 		let collection = NFTModule::collections(last_collection_id);

// 		assert_eq!(collection.owner, 1);
// 		assert_eq!(collection.uri, vec![2, 3, 3]);
// 	});
// }

// #[test]
// fn test_mint() {
// 	new_test_ext().execute_with(|| {
// 		let alice_address = 1;
// 		let alice = Origin::signed(alice_address);
// 		assert_ok!(NFTModule::create_collection(alice.clone(), vec![2, 3, 3]));

// 		let last_collection_id = NFTModule::last_collection_id();
// 		let not_available_collection_id = last_collection_id + 1;

// 		assert_noop!(
// 			NFTModule::mint(alice.clone(), not_available_collection_id, vec![2, 3, 3]),
// 			Error::<Test>::CollectionNotFound
// 		);
// 		assert_ok!(NFTModule::mint(
// 			alice.clone(),
// 			last_collection_id,
// 			vec![2, 3, 3]
// 		));

// 		assert_eq!(
// 			NFTModule::address_balances((last_collection_id, alice_address)),
// 			1
// 		);

// 		let last_token_id = NFTModule::last_token_id(last_collection_id);
// 		let token = NFTModule::tokens((last_collection_id, last_token_id));

// 		assert_eq!(token.end_idx, 0);
// 		assert_eq!(token.owner, alice_address);
// 		assert_eq!(token.uri, vec![2, 3, 3]);

// 		let collection = NFTModule::collections(last_collection_id);

// 		assert_eq!(collection.total_supply, 1)
// 	});
// }

// #[test]
// fn test_batch_mint() {
// 	new_test_ext().execute_with(|| {
// 		let alice_address = 1;
// 		let alice = Origin::signed(alice_address);
// 		assert_ok!(NFTModule::create_collection(alice.clone(), vec![2, 3, 3]));

// 		let last_collection_id = NFTModule::last_collection_id();
// 		let mint_amount = 0;

// 		assert_noop!(
// 			NFTModule::batch_mint(
// 				alice.clone(),
// 				last_collection_id,
// 				mint_amount,
// 				vec![2, 3, 3]
// 			),
// 			Error::<Test>::AmountLessThanOne
// 		);

// 		let mint_amount = 5;
// 		let not_available_collection_id = last_collection_id + 1;

// 		assert_noop!(
// 			NFTModule::batch_mint(
// 				alice.clone(),
// 				not_available_collection_id,
// 				mint_amount,
// 				vec![2, 3, 3]
// 			),
// 			Error::<Test>::CollectionNotFound
// 		);

// 		assert_ok!(NFTModule::batch_mint(
// 			alice.clone(),
// 			last_collection_id,
// 			5,
// 			vec![2, 3, 3]
// 		));
// 		assert_eq!(
// 			NFTModule::address_balances((last_collection_id, alice_address)),
// 			mint_amount
// 		);
// 		let last_token_id = NFTModule::last_token_id(last_collection_id);
// 		assert_eq!(last_token_id, mint_amount - 1);

// 		let start_idx = last_token_id + 1 - mint_amount;

// 		let token = NFTModule::tokens((last_collection_id, start_idx));

// 		assert_eq!(token.end_idx, mint_amount - 1);

// 		let collection = NFTModule::collections(last_collection_id);

// 		assert_eq!(collection.total_supply, mint_amount)
// 	});
// }

// #[test]
// fn test_transfer() {
// 	new_test_ext().execute_with(|| {
// 		let alice_address = 1;
// 		let bob_address = 2;
// 		let alice = Origin::signed(alice_address);

// 		assert_ok!(NFTModule::create_collection(alice.clone(), vec![2, 3, 3]));

// 		let last_collection_id = NFTModule::last_collection_id();

// 		assert_ok!(NFTModule::mint(
// 			alice.clone(),
// 			last_collection_id,
// 			vec![2, 3, 3]
// 		));

// 		let last_token_id = NFTModule::last_token_id(last_collection_id);
// 		let token = NFTModule::tokens((last_collection_id, last_token_id));
// 		assert_eq!(token.owner, alice_address);
// 		assert_eq!(
// 			NFTModule::address_balances((last_collection_id, alice_address)),
// 			1
// 		);

// 		let not_available_token_id = last_token_id + 1;
// 		let not_available_collection_id = last_collection_id + 1;

// 		assert_noop!(
// 			NFTModule::transfer(
// 				alice.clone(),
// 				bob_address,
// 				last_collection_id,
// 				not_available_token_id
// 			),
// 			Error::<Test>::TokenNotFound
// 		);
// 		assert_noop!(
// 			NFTModule::transfer(
// 				alice.clone(),
// 				bob_address,
// 				not_available_collection_id,
// 				last_token_id
// 			),
// 			Error::<Test>::CollectionNotFound
// 		);
// 		assert_noop!(
// 			NFTModule::transfer(
// 				alice.clone(),
// 				alice_address,
// 				last_collection_id,
// 				not_available_token_id
// 			),
// 			Error::<Test>::ReceiverIsSender
// 		);
// 		assert_ok!(NFTModule::transfer(
// 			alice.clone(),
// 			bob_address,
// 			last_collection_id,
// 			last_token_id
// 		));
// 		let token = NFTModule::tokens((last_collection_id, last_token_id));
// 		assert_eq!(token.owner, bob_address);

// 		assert_eq!(
// 			NFTModule::address_balances((last_collection_id, alice_address)),
// 			0
// 		);
// 		assert_eq!(
// 			NFTModule::address_balances((last_collection_id, bob_address)),
// 			1
// 		);
// 	});
// }

// #[test]
// fn test_batch_transfer() {
// 	new_test_ext().execute_with(|| {
// 		let alice_address = 1;
// 		let bob_address = 2;
// 		let alice = Origin::signed(alice_address);

// 		assert_ok!(NFTModule::create_collection(alice.clone(), vec![2, 3, 3]));

// 		let last_collection_id = NFTModule::last_collection_id();
// 		let mint_amount = 5;
// 		assert_ok!(NFTModule::batch_mint(
// 			alice.clone(),
// 			last_collection_id,
// 			mint_amount,
// 			vec![2, 3, 3]
// 		));

// 		let last_token_id = NFTModule::last_token_id(last_collection_id);
// 		let start_idx = mint_amount - last_token_id - 1;
// 		let token = NFTModule::tokens((last_collection_id, start_idx));
// 		assert_eq!(token.owner, alice_address);
// 		assert_eq!(
// 			NFTModule::address_balances((last_collection_id, alice_address)),
// 			mint_amount
// 		);

// 		let transfer_amount = 0;
// 		assert_noop!(
// 			NFTModule::batch_transfer(
// 				alice.clone(),
// 				bob_address,
// 				last_collection_id,
// 				start_idx,
// 				transfer_amount
// 			),
// 			Error::<Test>::AmountLessThanOne
// 		);

// 		let transfer_amount = 2;
// 		let not_available_token_id = last_token_id + 1;
// 		let not_available_collection_id = last_collection_id + 1;

// 		assert_noop!(
// 			NFTModule::batch_transfer(
// 				alice.clone(),
// 				bob_address,
// 				last_collection_id,
// 				not_available_token_id,
// 				transfer_amount
// 			),
// 			Error::<Test>::TokenNotFound
// 		);
// 		assert_noop!(
// 			NFTModule::batch_transfer(
// 				alice.clone(),
// 				bob_address,
// 				not_available_collection_id,
// 				start_idx,
// 				transfer_amount
// 			),
// 			Error::<Test>::CollectionNotFound
// 		);
// 		assert_noop!(
// 			NFTModule::batch_transfer(
// 				alice.clone(),
// 				alice_address,
// 				last_collection_id,
// 				start_idx,
// 				transfer_amount
// 			),
// 			Error::<Test>::ReceiverIsSender
// 		);

// 		assert_ok!(NFTModule::batch_transfer(
// 			alice.clone(),
// 			bob_address,
// 			last_collection_id,
// 			start_idx,
// 			transfer_amount
// 		));

// 		assert_eq!(
// 			NFTModule::address_balances((last_collection_id, alice_address)),
// 			mint_amount - transfer_amount
// 		);
// 		assert_eq!(
// 			NFTModule::address_balances((last_collection_id, bob_address)),
// 			transfer_amount
// 		);

// 		let alice_start_idx = last_token_id - transfer_amount;
// 		let bob_nft = NFTModule::tokens((last_collection_id, start_idx));
// 		let alice_nft = NFTModule::tokens((last_collection_id, alice_start_idx));

// 		assert_eq!(bob_nft.owner, bob_address);
// 		assert_eq!(alice_nft.owner, alice_address);

// 		assert_eq!(alice_nft.uri, bob_nft.uri);
// 	});
// }

// #[test]
// fn test_burn() {
// 	new_test_ext().execute_with(|| {
// 		let alice_address = 1;
// 		let alice = Origin::signed(alice_address);

// 		assert_ok!(NFTModule::create_collection(alice.clone(), vec![2, 3, 3]));

// 		let last_collection_id = NFTModule::last_collection_id();

// 		assert_ok!(NFTModule::mint(
// 			alice.clone(),
// 			last_collection_id,
// 			vec![2, 3, 3]
// 		));
// 		assert_eq!(
// 			NFTModule::address_balances((last_collection_id, alice_address)),
// 			1
// 		);

// 		let last_token_id = NFTModule::last_token_id(last_collection_id);
// 		let not_available_token_id = last_token_id + 1;
// 		let not_available_collection_id = last_collection_id + 1;

// 		let collection = NFTModule::collections(last_collection_id);
// 		assert_eq!(collection.total_supply, 1);

// 		assert_noop!(
// 			NFTModule::burn(alice.clone(), last_collection_id, not_available_token_id),
// 			Error::<Test>::TokenNotFound
// 		);
// 		assert_noop!(
// 			NFTModule::burn(alice.clone(), not_available_collection_id, last_token_id),
// 			Error::<Test>::CollectionNotFound
// 		);
// 		assert_ok!(NFTModule::burn(
// 			alice.clone(),
// 			last_collection_id,
// 			last_token_id
// 		));
// 		assert_eq!(
// 			NFTModule::address_balances((last_collection_id, alice_address)),
// 			0
// 		);

// 		let collection = NFTModule::collections(last_collection_id);
// 		assert_eq!(collection.total_supply, 0)
// 	});
// }

// #[test]
// fn test_batch_burn() {
// 	new_test_ext().execute_with(|| {
// 		let alice_address = 1;
// 		let alice = Origin::signed(alice_address);

// 		assert_ok!(NFTModule::create_collection(alice.clone(), vec![2, 3, 3]));

// 		let last_collection_id = NFTModule::last_collection_id();
// 		let mint_amount = 5;

// 		assert_ok!(NFTModule::batch_mint(
// 			alice.clone(),
// 			last_collection_id,
// 			mint_amount,
// 			vec![2, 3, 3]
// 		));
// 		assert_eq!(
// 			NFTModule::address_balances((last_collection_id, alice_address)),
// 			mint_amount
// 		);

// 		let last_token_id = NFTModule::last_token_id(last_collection_id);
// 		let not_available_token_id = last_token_id + 1;
// 		let not_available_collection_id = last_collection_id + 1;

// 		let collection = NFTModule::collections(last_collection_id);
// 		assert_eq!(collection.total_supply, mint_amount);

// 		let burn_amount = 2;
// 		let start_idx = mint_amount - last_token_id - 1;

// 		assert_noop!(
// 			NFTModule::batch_burn(
// 				alice.clone(),
// 				last_collection_id,
// 				not_available_token_id,
// 				burn_amount
// 			),
// 			Error::<Test>::TokenNotFound
// 		);
// 		assert_noop!(
// 			NFTModule::batch_burn(
// 				alice.clone(),
// 				not_available_collection_id,
// 				start_idx,
// 				burn_amount
// 			),
// 			Error::<Test>::CollectionNotFound
// 		);
// 		assert_ok!(NFTModule::batch_burn(
// 			alice.clone(),
// 			last_collection_id,
// 			start_idx,
// 			burn_amount
// 		));
// 		assert_eq!(
// 			NFTModule::address_balances((last_collection_id, alice_address)),
// 			mint_amount - burn_amount
// 		);

// 		let collection = NFTModule::collections(last_collection_id);
// 		assert_eq!(collection.total_supply, mint_amount - burn_amount)
// 	});
// }