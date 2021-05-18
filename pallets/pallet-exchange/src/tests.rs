use crate::{mock::*, Error};
use frame_support::traits::Currency;
use frame_support::{assert_noop, assert_ok};
use pallet_collection::CollectionInterface;
use sp_runtime::traits::SaturatedConversion;

#[test]
fn curve() {
    new_test_ext().execute_with(|| {
        let reverse_ratio = 500000;
        let total_supply = 3;
        let pool_balance = 2_u128.saturated_into::<crate::BalanceOf<Test>>();
        let amount = 1;
        let cost =
            TemplateModule::buy_cost(pool_balance, amount, total_supply, reverse_ratio).unwrap();
        //  ceil(1.55)
        assert_eq!(cost, 2);

        let total_supply = 2;
        let pool_balance = 2_u128.saturated_into::<crate::BalanceOf<Test>>();
        let amount = 1;
        let receive =
            TemplateModule::sell_receive(pool_balance, amount, total_supply, reverse_ratio)
                .unwrap();
        // floor(1.5 )
        assert_eq!(receive, 1);

        let amount = 5;
        let m = 1000;
        let first_cost = TemplateModule::first_buy_cost(reverse_ratio, m, amount).unwrap();
        // ceil(to_fixed2(12500.0005371401917915778))
        assert_eq!(first_cost, 12500);
    });
}

#[test]
fn sell_nft() {
    new_test_ext().execute_with(|| {
        let alice_address = 1;
        let alice = Origin::signed(alice_address);
        let mint_amount = 10;

        CollectionModule::create_collection(alice.clone(), vec![2, 3, 3], false).unwrap();

        let nonce = CollectionModule::get_nonce();
        let collection_id =
            <CollectionModule as CollectionInterface<_, _>>::generate_collection_id(nonce).unwrap();

        assert_ok!(NFTModule::mint_non_fungible(
            alice.clone(),
            alice_address,
            collection_id,
            vec![2, 3, 3],
            mint_amount
        ));

        let token_id = 0;
        let price = 1_u128.saturated_into::<crate::BalanceOf<Test>>();

        assert_ok!(TemplateModule::sell_nft(
            alice,
            collection_id,
            token_id,
            mint_amount,
            price
        ));

        let order_id = TemplateModule::next_nft_order_id() - 1;
        let order = TemplateModule::nft_order(order_id);

        assert_eq!(&order.seller, &alice_address);

        let token = NFTModule::tokens(collection_id, token_id);

        assert_eq!(&token.owner, &TemplateModule::account_id());
    });
}

#[test]
fn sell_nft_failed() {
    new_test_ext().execute_with(|| {
        let alice_address = 1;
        let alice = Origin::signed(alice_address);
        let bob_address = 2;
        let bob = Origin::signed(bob_address);
        let mint_amount = 10;

        CollectionModule::create_collection(alice.clone(), vec![2, 3, 3], false).unwrap();

        let nonce = CollectionModule::get_nonce();
        let collection_id =
            <CollectionModule as CollectionInterface<_, _>>::generate_collection_id(nonce).unwrap();
        let token_id = 0;
        let price = 1_u128.saturated_into::<crate::BalanceOf<Test>>();

        assert_noop!(
            TemplateModule::sell_nft(
                alice.clone(),
                collection_id.clone(),
                token_id,
                mint_amount,
                price
            ),
            Error::<Test>::TokenNotFound
        );

        assert_ok!(NFTModule::mint_non_fungible(
            alice.clone(),
            alice_address,
            collection_id,
            vec![2, 3, 3],
            mint_amount
        ));

        assert_noop!(
            TemplateModule::sell_nft(bob, collection_id, token_id, mint_amount, price),
            Error::<Test>::PermissionDenied
        );
    });
}

#[test]
fn buy_nft() {
    new_test_ext().execute_with(|| {
        let alice_address = 1;
        let alice = Origin::signed(alice_address);
        let bob_address = 2;
        let bob = Origin::signed(bob_address);
        let mint_amount = 10;

        CollectionModule::create_collection(alice.clone(), vec![2, 3, 3], false).unwrap();

        let nonce = CollectionModule::get_nonce();
        let collection_id =
            <CollectionModule as CollectionInterface<_, _>>::generate_collection_id(nonce).unwrap();

        assert_ok!(NFTModule::mint_non_fungible(
            alice.clone(),
            alice_address,
            collection_id,
            vec![2, 3, 3],
            mint_amount
        ));

        let token_id = 0;
        let price = 1_u128.saturated_into::<crate::BalanceOf<Test>>();

        assert_ok!(TemplateModule::sell_nft(
            alice,
            collection_id,
            token_id,
            mint_amount,
            price
        ));

        let _ = Balances::deposit_creating(&bob_address, 2);

        let order_id = TemplateModule::next_nft_order_id() - 1;
        // let order = TemplateModule::nft_order(order_id);
        assert_ok!(TemplateModule::buy_nft(bob, order_id, 1));

        let order = TemplateModule::nft_order(order_id);
        assert_eq!(&order.start_idx, &1_u128);

        assert_eq!(NFTModule::tokens(collection_id, 0).owner, bob_address);
        assert_eq!(
            NFTModule::tokens(collection_id, 1).owner,
            TemplateModule::account_id()
        );
    });
}

#[test]
fn cancel_nft_order() {
    new_test_ext().execute_with(|| {
        let alice_address = 1;
        let alice = Origin::signed(alice_address);
        let bob_address = 2;
        let bob = Origin::signed(bob_address);
        let mint_amount = 10;

        CollectionModule::create_collection(alice.clone(), vec![2, 3, 3], false).unwrap();

        let nonce = CollectionModule::get_nonce();
        let collection_id =
            <CollectionModule as CollectionInterface<_, _>>::generate_collection_id(nonce).unwrap();

        assert_ok!(NFTModule::mint_non_fungible(
            alice.clone(),
            alice_address,
            collection_id,
            vec![2, 3, 3],
            mint_amount
        ));

        let token_id = 0;
        let price = 1_u128.saturated_into::<crate::BalanceOf<Test>>();

        assert_ok!(TemplateModule::sell_nft(
            alice.clone(),
            collection_id,
            token_id,
            mint_amount,
            price
        ));

        let _ = Balances::deposit_creating(&bob_address, 2);

        let order_id = TemplateModule::next_nft_order_id() - 1;
        // let order = TemplateModule::nft_order(order_id);
        assert_ok!(TemplateModule::buy_nft(bob.clone(), order_id, 1));

        let order = TemplateModule::nft_order(order_id);
        assert_eq!(&order.start_idx, &1_u128);

        assert_noop!(
            TemplateModule::cancel_nft_order(bob, order_id),
            Error::<Test>::PermissionDenied
        );
        assert_ok!(TemplateModule::cancel_nft_order(alice, order_id));

        assert_eq!(NFTModule::tokens(collection_id, 0).owner, bob_address);
        assert_eq!(NFTModule::tokens(collection_id, 1).owner, alice_address);
    });
}

#[test]
fn buy_nft_failed() {
    new_test_ext().execute_with(|| {
        let alice_address = 1;
        let alice = Origin::signed(alice_address);
        let bob_address = 2;
        let bob = Origin::signed(bob_address);
        let mint_amount = 10;

        CollectionModule::create_collection(alice.clone(), vec![2, 3, 3], false).unwrap();

        let nonce = CollectionModule::get_nonce();
        let collection_id =
            <CollectionModule as CollectionInterface<_, _>>::generate_collection_id(nonce).unwrap();

        assert_ok!(NFTModule::mint_non_fungible(
            alice.clone(),
            alice_address,
            collection_id,
            vec![2, 3, 3],
            mint_amount
        ));

        let token_id = 0;
        let price = 1_u128.saturated_into::<crate::BalanceOf<Test>>();

        assert_ok!(TemplateModule::sell_nft(
            alice,
            collection_id,
            token_id,
            mint_amount,
            price
        ));

        let _ = Balances::deposit_creating(&bob_address, 2);

        let order_id = TemplateModule::next_nft_order_id() - 1;

        assert_noop!(
            TemplateModule::buy_nft(bob.clone(), order_id, 0),
            Error::<Test>::AmountLessThanOne
        );
        assert_noop!(
            TemplateModule::buy_nft(bob.clone(), 1, 1),
            Error::<Test>::OrderNotFound
        );
        assert_noop!(
            TemplateModule::buy_nft(bob, order_id, 11),
            Error::<Test>::AmountTooLarge
        );
    });
}

#[test]
fn create_smei_token_pool() {
    new_test_ext().execute_with(|| {
        let alice_address = 1;
        let alice = Origin::signed(alice_address);
        let mint_amount = 10;

        CollectionModule::create_collection(alice.clone(), vec![2, 3, 3], true).unwrap();

        let nonce = CollectionModule::get_nonce();
        let collection_id =
            <CollectionModule as CollectionInterface<_, _>>::generate_collection_id(nonce).unwrap();

        assert_ok!(NFTModule::mint_fungible(
            alice.clone(),
            alice_address,
            collection_id,
            mint_amount
        ));
        // crate::
        let duration = 1_u128.saturated_into::<BlockNumber>();

        let reverse_ratio = 500000;
        let m = 20;

        assert_ok!(TemplateModule::create_semi_token_pool(
            alice,
            collection_id,
            10,
            reverse_ratio,
            m,
            duration
        ));

        let pool = TemplateModule::semi_fungible_pool((&collection_id, &alice_address));

        assert_eq!(&pool.end_time, &1_u128.saturated_into::<BlockNumber>());
        assert_eq!(
            NFTModule::address_balances((collection_id, TemplateModule::account_id())),
            10
        );
    });
}

#[test]
fn create_smei_token_pool_falied() {
    new_test_ext().execute_with(|| {
        let alice_address = 1;
        let alice = Origin::signed(alice_address);
        let bob_address = 2;
        let bob = Origin::signed(bob_address);
        let mint_amount = 10;

        CollectionModule::create_collection(alice.clone(), vec![2, 3, 3], false).unwrap();

        let nonce = CollectionModule::get_nonce();
        let collection_id =
            <CollectionModule as CollectionInterface<_, _>>::generate_collection_id(nonce).unwrap();

        assert_ok!(NFTModule::mint_non_fungible(
            alice.clone(),
            alice_address,
            collection_id,
            vec![2, 3, 3],
            mint_amount
        ));
        // crate::
        let duration = 1_u128.saturated_into::<BlockNumber>();

        let reverse_ratio = 500000;
        let m = 20;

        assert_noop!(
            TemplateModule::create_semi_token_pool(
                alice.clone(),
                collection_id,
                10,
                m,
                reverse_ratio,
                duration
            ),
            Error::<Test>::WrongTokenType
        );

        CollectionModule::create_collection(alice.clone(), vec![2, 3, 3], true).unwrap();

        let nonce = CollectionModule::get_nonce();
        let collection_id =
            <CollectionModule as CollectionInterface<_, _>>::generate_collection_id(nonce).unwrap();

        assert_ok!(NFTModule::mint_fungible(
            alice.clone(),
            alice_address,
            collection_id,
            mint_amount
        ));

        assert_noop!(
            TemplateModule::create_semi_token_pool(
                alice.clone(),
                collection_id,
                10,
                0,
                m,
                duration
            ),
            Error::<Test>::ReverseRatioLessThanOne
        );
        assert_noop!(
            TemplateModule::create_semi_token_pool(
                alice.clone(),
                collection_id,
                10,
                reverse_ratio,
                0,
                duration
            ),
            Error::<Test>::MLessThanOne
        );
        assert_noop!(
            TemplateModule::create_semi_token_pool(
                bob,
                collection_id,
                10,
                reverse_ratio,
                m,
                duration
            ),
            Error::<Test>::AmountTooLarge
        );

        assert_ok!(TemplateModule::create_semi_token_pool(
            alice.clone(),
            collection_id,
            10,
            reverse_ratio,
            m,
            duration
        ));
        assert_noop!(
            TemplateModule::create_semi_token_pool(
                alice,
                collection_id,
                10,
                reverse_ratio,
                m,
                duration
            ),
            Error::<Test>::PoolExisted
        );
    });
}

#[test]
fn buy_and_sell_smei_token() {
    new_test_ext().execute_with(|| {
        let alice_address = 1;
        let alice = Origin::signed(alice_address);
        let bob_address = 2;
        let bob = Origin::signed(bob_address);
        let mint_amount = 10;

        CollectionModule::create_collection(alice.clone(), vec![2, 3, 3], true).unwrap();

        let nonce = CollectionModule::get_nonce();
        let collection_id =
            <CollectionModule as CollectionInterface<_, _>>::generate_collection_id(nonce).unwrap();

        assert_ok!(NFTModule::mint_fungible(
            alice.clone(),
            alice_address,
            collection_id,
            mint_amount
        ));
        // crate::
        let duration = 1_u128.saturated_into::<BlockNumber>();

        let reverse_ratio = 500000;
        let m = 20;
        let _ = Balances::deposit_creating(&bob_address, 100);

        assert_ok!(TemplateModule::create_semi_token_pool(
            alice.clone(),
            collection_id,
            10,
            reverse_ratio,
            m,
            duration
        ));
        assert_ok!(TemplateModule::buy_semi_token(
            bob.clone(),
            collection_id.clone(),
            alice_address.clone(),
            1
        ));
        // cost 10s
        assert_eq!(Balances::free_balance(&bob_address), 90);

        assert_ok!(TemplateModule::sell_semi_token(
            bob.clone(),
            collection_id.clone(),
            alice_address.clone(),
            1
        ));
        assert_eq!(Balances::free_balance(&bob_address), 100);
    });
}

#[test]
fn buy_and_sell_smei_token_failed() {
    new_test_ext().execute_with(|| {
        let alice_address = 1;
        let alice = Origin::signed(alice_address);
        let bob_address = 2;
        let bob = Origin::signed(bob_address);
        let mint_amount = 10;

        CollectionModule::create_collection(alice.clone(), vec![2, 3, 3], true).unwrap();

        let nonce = CollectionModule::get_nonce();
        let collection_id =
            <CollectionModule as CollectionInterface<_, _>>::generate_collection_id(nonce).unwrap();

        assert_ok!(NFTModule::mint_fungible(
            alice.clone(),
            alice_address,
            collection_id,
            mint_amount
        ));
        // crate::
        let duration = 1_u128.saturated_into::<BlockNumber>();

        let reverse_ratio = 500000;
        let m = 20;
        let _ = Balances::deposit_creating(&bob_address, 100);

        assert_ok!(TemplateModule::create_semi_token_pool(
            alice.clone(),
            collection_id,
            10,
            reverse_ratio,
            m,
            duration
        ));
        assert_noop!(
            TemplateModule::buy_semi_token(
                bob.clone(),
                collection_id.clone(),
                alice_address.clone(),
                100
            ),
            Error::<Test>::AmountTooLarge
        );
        assert_noop!(
            TemplateModule::buy_semi_token(
                bob.clone(),
                collection_id.clone(),
                bob_address.clone(),
                100
            ),
            Error::<Test>::PoolNotFound
        );

        assert_ok!(TemplateModule::buy_semi_token(
            bob.clone(),
            collection_id.clone(),
            alice_address.clone(),
            1
        ));

        assert_noop!(
            TemplateModule::sell_semi_token(
                bob.clone(),
                collection_id.clone(),
                alice_address.clone(),
                100
            ),
            Error::<Test>::AmountTooLarge
        );
        assert_noop!(
            TemplateModule::sell_semi_token(
                bob.clone(),
                collection_id.clone(),
                bob_address.clone(),
                100
            ),
            Error::<Test>::PoolNotFound
        );

        System::set_block_number(System::block_number() + 2);

        assert_noop!(
            TemplateModule::buy_semi_token(
                bob.clone(),
                collection_id.clone(),
                alice_address.clone(),
                1
            ),
            Error::<Test>::ExpiredSoldTime
        );
        assert_noop!(
            TemplateModule::sell_semi_token(
                bob.clone(),
                collection_id.clone(),
                alice_address.clone(),
                1
            ),
            Error::<Test>::ExpiredSoldTime
        );
    });
}

#[test]
fn withdraw_pool() {
    new_test_ext().execute_with(|| {
        let alice_address = 1;
        let alice = Origin::signed(alice_address);
        let bob_address = 2;
        let bob = Origin::signed(bob_address);
        let mint_amount = 10;

        CollectionModule::create_collection(alice.clone(), vec![2, 3, 3], true).unwrap();

        let nonce = CollectionModule::get_nonce();
        let collection_id =
            <CollectionModule as CollectionInterface<_, _>>::generate_collection_id(nonce).unwrap();

        assert_ok!(NFTModule::mint_fungible(
            alice.clone(),
            alice_address,
            collection_id,
            mint_amount
        ));
        // crate::
        let duration = 1_u128.saturated_into::<BlockNumber>();

        let reverse_ratio = 500000;
        let m = 20;
        let _ = Balances::deposit_creating(&bob_address, 100);

        assert_ok!(TemplateModule::create_semi_token_pool(
            alice.clone(),
            collection_id,
            10,
            reverse_ratio,
            m,
            duration
        ));
        assert_ok!(TemplateModule::buy_semi_token(
            bob.clone(),
            collection_id.clone(),
            alice_address.clone(),
            1
        ));

        assert_noop!(
            TemplateModule::withdraw_pool(alice.clone(), collection_id.clone()),
            Error::<Test>::CanNotWithdraw
        );
        assert_noop!(
            TemplateModule::withdraw_pool(bob.clone(), collection_id.clone()),
            Error::<Test>::PoolNotFound
        );

        System::set_block_number(System::block_number() + 3);

        assert_ok!(TemplateModule::withdraw_pool(
            alice.clone(),
            collection_id.clone()
        ));

        assert_eq!(Balances::free_balance(&alice_address), 10);
        assert_eq!(
            NFTModule::address_balances((collection_id, alice_address)),
            9
        );
    });
}
