use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};
use sp_runtime::traits::{SaturatedConversion, Saturating};
#[test]
fn it_works_for_default_value() {
    new_test_ext().execute_with(|| {
        let reverse_ratio = 500000;
        let total_supply = 3;
        let pool_balance = 2_u128.saturated_into::<crate::BalanceOf<Test>>();
        let amount = 1;
        let cost = TemplateModule::buy_cost(pool_balance, amount, total_supply, reverse_ratio).unwrap();
        //  ceil(1.55)
        assert_eq!(cost, 2);

        let total_supply = 2;
        let pool_balance = 2_u128.saturated_into::<crate::BalanceOf<Test>>();
        let amount = 1;
        let receive = TemplateModule::sell_receive(pool_balance, amount, total_supply, reverse_ratio).unwrap();
        // floor(1.5 )
        assert_eq!(receive, 1);
    });
}
