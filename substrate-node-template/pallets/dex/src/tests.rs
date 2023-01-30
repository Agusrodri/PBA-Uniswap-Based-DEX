use crate::{mock::*, Error, Event, Pool, OraclePrice, PoolsMap};
use frame_support::{
	assert_noop, assert_ok,
	traits::{
		fungibles::{self, *},
		Currency,
	},
};

#[test]
fn create_pool_successfully() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		let asset_id = 3u32;
		let liquidity_asset_id = 2u32;
		let account_id = 1u64;

		//create a sender
		let sender = RuntimeOrigin::signed(account_id);

		//transfer currency to the sender
		let _ = <Test as crate::Config>::Currency::deposit_creating(&account_id, 100u128);

		//create an asset
		assert_ok!(Dex::create_asset_helper(asset_id));

		//mint asset to user
		assert!(
			<Test as crate::Config>::Fungibles::mint_into(asset_id, &account_id, 100u128).is_ok()
		);

		//create a pool and add liquidity to it
		assert_ok!(Dex::create_pool(sender, asset_id, liquidity_asset_id, 50u128, 50u128));

		//verify the sender balances changed
		assert_eq!(<Test as crate::Config>::Currency::free_balance(&account_id), 50u128);
		assert_eq!(<Test as crate::Config>::Fungibles::balance(asset_id, &account_id), 50u128);

		//rcheck the new pool values are correct
		let pool =
			Pool { asset_id, liquidity_asset_id, asset_reserve: 50u128, currency_reserve: 50u128 };

		//compare both pools to check values
		assert_eq!(Dex::pools(asset_id).unwrap_or_default(), pool);

		//check last event
		System::assert_last_event(Event::PoolCreated { asset_id, liquidity_asset_id }.into());
	})
}

#[test]
//#[allow(unused_must_use)]
fn create_pool_fails_existing_pool() {
	new_test_ext().execute_with(|| {
		let asset_id = 3u32;
		let liquidity_asset_id = 2u32;
		let second_liquidity_asset_id = 5u32;
		let account_id = 1u64;

		//create a sender
		let sender = RuntimeOrigin::signed(account_id);

		//transfer currency to the sender
		let _ = <Test as crate::Config>::Currency::deposit_creating(&account_id, 100u128);

		//create an asset
		assert_ok!(Dex::create_asset_helper(asset_id));

		//mint asset to user
		assert!(
			<Test as crate::Config>::Fungibles::mint_into(asset_id, &account_id, 100u128).is_ok()
		);

		//create a pool and add liquidity to it
		assert_ok!(Dex::create_pool(sender.clone(), asset_id, liquidity_asset_id, 50u128, 50u128));

		//try to create another pool with the same asset_id
		assert_noop!(
			Dex::create_pool(sender, asset_id, second_liquidity_asset_id, 10u128, 10u128),
			Error::<Test>::PoolAlreadyExists
		);
	})
}

#[test]
fn create_pool_fails_existing_liq_asset() {
	new_test_ext().execute_with(|| {
		let asset_id = 3u32;
		let liquidity_asset_id = 2u32;
		let second_asset_id = 5u32;
		let account_id = 1u64;

		//create a sender
		let sender = RuntimeOrigin::signed(account_id);

		//transfer currency to the sender
		let _ = <Test as crate::Config>::Currency::deposit_creating(&account_id, 100u128);

		//create an asset
		assert_ok!(Dex::create_asset_helper(asset_id));

		//mint asset to user
		assert!(
			<Test as crate::Config>::Fungibles::mint_into(asset_id, &account_id, 100u128).is_ok()
		);

		//create a pool and add liquidity to it
		assert_ok!(Dex::create_pool(sender.clone(), asset_id, liquidity_asset_id, 50u128, 50u128));

		//try to create another pool with the same liquidity_asset_id
		assert_noop!(
			Dex::create_pool(sender, second_asset_id, liquidity_asset_id, 10u128, 10u128),
			Error::<Test>::AssetAlreadyExists
		);
	})
}

#[test]
fn create_pool_fails_asset_not_found() {
	new_test_ext().execute_with(|| {
		let asset_id = 3u32;
		let liquidity_asset_id = 2u32;
		let account_id = 1u64;

		//create a sender
		let sender = RuntimeOrigin::signed(account_id);

		//fails to create a pool with an non-existent asset
		assert_noop!(
			Dex::create_pool(sender.clone(), asset_id, liquidity_asset_id, 50u128, 50u128),
			Error::<Test>::AssetNotFound
		);
	})
}

#[test]
fn create_pool_fails_asset_amount_zero() {
	new_test_ext().execute_with(|| {
		let asset_id = 3u32;
		let liquidity_asset_id = 2u32;
		let account_id = 1u64;

		//create a sender
		let sender = RuntimeOrigin::signed(account_id);

		//transfer currency to the sender
		let _ = <Test as crate::Config>::Currency::deposit_creating(&account_id, 100u128);

		//create an asset
		assert_ok!(Dex::create_asset_helper(asset_id));

		//mint asset to user
		assert!(
			<Test as crate::Config>::Fungibles::mint_into(asset_id, &account_id, 100u128).is_ok()
		);

		//fails to create a pool because of the zero asset amount
		assert_noop!(
			Dex::create_pool(sender, asset_id, liquidity_asset_id, 50u128, 0u128),
			Error::<Test>::AssetAmountZero
		);
	})
}

//add test to create a pool with a currency amount of zero
#[test]
fn create_pool_fails_currency_amount_zero() {
	new_test_ext().execute_with(|| {
		let asset_id = 3u32;
		let liquidity_asset_id = 2u32;
		let account_id = 1u64;

		//create a sender
		let sender = RuntimeOrigin::signed(account_id);

		//transfer currency to the sender
		let _ = <Test as crate::Config>::Currency::deposit_creating(&account_id, 100u128);

		//create an asset
		assert_ok!(Dex::create_asset_helper(asset_id));

		//mint asset to user
		assert!(
			<Test as crate::Config>::Fungibles::mint_into(asset_id, &account_id, 100u128).is_ok()
		);

		//fails to create a pool because of the zero currency amount
		assert_noop!(
			Dex::create_pool(sender, asset_id, liquidity_asset_id, 0u128, 50u128),
			Error::<Test>::CurrencyAmountZero
		);
	})
}

#[test]
fn add_liquidity_successfully() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		let asset_id = 3u32;
		let liquidity_asset_id = 2u32;
		let account_id = 1u64;

		//create a sender
		let sender = RuntimeOrigin::signed(account_id);

		//transfer currency to the sender
		let _ = <Test as crate::Config>::Currency::deposit_creating(&account_id, 100u128);

		//create an asset
		assert_ok!(Dex::create_asset_helper(asset_id));

		//mint asset to user
		assert!(
			<Test as crate::Config>::Fungibles::mint_into(asset_id, &account_id, 100u128).is_ok()
		);

		//create a pool and add liquidity to it
		assert_ok!(Dex::create_pool(sender.clone(), asset_id, liquidity_asset_id, 50u128, 50u128));

		//add liquidity to the pool created previously
		assert_ok!(Dex::add_liquidity(sender, asset_id, 10u128));

		//get the pool
		let pool = Dex::pools(asset_id).unwrap_or_default();

		//check that the new asset amount to add is (currency_amount/ currency_reserve) *
		// asset_reserve
		let asset_amount_to_add = ((10u128 / 50u128) * 50u128) + 1;

		//check that the new liquidity asset amount to mint is (currency_amount/ currency_reserve)
		// *asset_liq_total_issuance
		let liquidity_assets_to_add = (10u128 / 50u128) * 50u128;

		assert_eq!(pool.currency_reserve, 60u128);
		assert_eq!(pool.asset_reserve, 50u128 + asset_amount_to_add);
		assert_eq!(
			<Test as crate::Config>::Fungibles::balance(liquidity_asset_id, &account_id),
			50u128 + liquidity_assets_to_add
		);

		System::assert_last_event(
			Event::LiquidityAdded {
				provider: account_id,
				asset_id,
				currency_amount: 10u128,
				asset_amount: asset_amount_to_add,
				liquidity_minted: liquidity_assets_to_add,
			}
			.into(),
		);
	})
}

#[test]
fn add_liquidity_fails_pool_not_found() {
	new_test_ext().execute_with(|| {
		let asset_id = 3u32;
		let account_id = 1u64;

		//create a sender
		let sender = RuntimeOrigin::signed(account_id);

		//transfer currency to the sender
		let _ = <Test as crate::Config>::Currency::deposit_creating(&account_id, 100u128);

		//create an asset
		assert_ok!(Dex::create_asset_helper(asset_id));

		//fails to add liquidity because the pool doesn't exists
		assert_noop!(Dex::add_liquidity(sender, asset_id, 10u128), Error::<Test>::PoolNotFound);
	})
}

#[test]
fn add_liquidity_fails_currency_amount_zero() {
	new_test_ext().execute_with(|| {
		let asset_id = 3u32;
		let account_id = 1u64;

		//create a sender
		let sender = RuntimeOrigin::signed(account_id);

		//transfer currency to the sender
		let _ = <Test as crate::Config>::Currency::deposit_creating(&account_id, 100u128);

		//create an asset
		assert_ok!(Dex::create_asset_helper(asset_id));

		//fails to add liquidity because the currency amount is zero
		assert_noop!(
			Dex::add_liquidity(sender, asset_id, 0u128),
			Error::<Test>::CurrencyAmountZero
		);
	})
}

#[test]
fn remove_liquidity_successfully() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		let asset_id = 3u32;
		let liquidity_asset_id = 2u32;
		let account_id = 1u64;

		//create a sender
		let sender = RuntimeOrigin::signed(account_id);

		//transfer currency to the sender
		let _ = <Test as crate::Config>::Currency::deposit_creating(&account_id, 100u128);

		//create an asset
		assert_ok!(Dex::create_asset_helper(asset_id));

		//mint asset to user
		assert!(
			<Test as crate::Config>::Fungibles::mint_into(asset_id, &account_id, 100u128).is_ok()
		);

		//create a pool and add liquidity to it
		assert_ok!(Dex::create_pool(sender.clone(), asset_id, liquidity_asset_id, 50u128, 50u128));

		//get the pool and reserves values before removing liquidity
		let pool = Dex::pools(asset_id).unwrap_or_default();
		let currency_reserve_before = pool.currency_reserve.clone();
		let asset_reserve_before = pool.asset_reserve.clone();

		//remove liquidity
		assert_ok!(Dex::remove_liquidity(sender, asset_id, 10u128));

		//check that the new currency amount to remove is (liquidity_amount/
		// asset_liq_total_issuance) * currency_reserve
		let currency_amount_to_remove = (10u128 / 50u128) * pool.currency_reserve;

		//check the new asset amount to remove is (liquidity_amount/ asset_liq_total_issuance) *
		// asset_reserve
		let asset_amount_to_remove = (10u128 / 50u128) * pool.asset_reserve;

		//check the liq_assets were burnt
		assert_eq!(<Test as crate::Config>::Fungibles::total_issuance(liquidity_asset_id), 40);

		//check the sender has new amount of currency and asset
		assert_eq!(
			<Test as crate::Config>::Currency::free_balance(&account_id),
			50u128 + currency_amount_to_remove
		);
		assert_eq!(
			<Test as crate::Config>::Fungibles::balance(asset_id, &account_id),
			50u128 + asset_amount_to_remove
		);

		//check updated values of the pool
		assert_eq!(pool.currency_reserve, currency_reserve_before - currency_amount_to_remove);
		assert_eq!(pool.asset_reserve, asset_reserve_before - asset_amount_to_remove);

		//check the last event
		System::assert_last_event(
			Event::LiquidityRemoved {
				provider: account_id,
				asset_id,
				currency_amount: currency_amount_to_remove,
				asset_amount: asset_amount_to_remove,
				liquidity_amount: 10u128,
			}
			.into(),
		);
	})
}

#[test]
fn remove_liquidity_fails_liquidity_amount_zero() {
	new_test_ext().execute_with(|| {
		let asset_id = 3u32;
		let account_id = 1u64;

		//create a sender
		let sender = RuntimeOrigin::signed(account_id);

		//create an asset
		assert_ok!(Dex::create_asset_helper(asset_id));

		//fails to remove liquidity because the liquidity asset amount is zero
		assert_noop!(Dex::remove_liquidity(sender, asset_id, 0u128), Error::<Test>::LiqAmountZero);
	})
}

#[test]
fn remove_liquidity_fails_pool_not_found() {
	new_test_ext().execute_with(|| {
		let asset_id = 3u32;
		let account_id = 1u64;

		//create a sender
		let sender = RuntimeOrigin::signed(account_id);

		//create an asset
		assert_ok!(Dex::create_asset_helper(asset_id));

		//fails to remove liquidity because the pool with the requested asset_id is not created
		assert_noop!(Dex::remove_liquidity(sender, asset_id, 10u128), Error::<Test>::PoolNotFound);
	})
}

#[test]
fn currency_to_asset_successfully() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		let asset_id = 3u32;
		let account_id = 1u64;
		let liquidity_asset_id = 2u32;

		//create a sender
		let sender = RuntimeOrigin::signed(account_id);

		//transfer currency to the sender
		let _ = <Test as crate::Config>::Currency::deposit_creating(&account_id, 100u128);

		//create an asset
		assert_ok!(Dex::create_asset_helper(asset_id));

		//mint asset to user
		assert!(
			<Test as crate::Config>::Fungibles::mint_into(asset_id, &account_id, 100u128).is_ok()
		);

		//create a pool and add liquidity to it
		assert_ok!(Dex::create_pool(sender.clone(), asset_id, liquidity_asset_id, 50u128, 50u128));

		//get the pool created previously
		let pool = Dex::pools(asset_id).unwrap_or_default();

		//calculate the asset amount to give to the sender based on the currency input amount
		let asset_amount_to_check = Dex::get_input_convert(
			20u128,
			pool.currency_reserve.clone(),
			pool.asset_reserve.clone(),
		)
		.unwrap_or_default();

		//perform the currency to asset swap operation
		assert_ok!(Dex::currency_to_asset(sender, 20u128, asset_id));

		//verify new sender's asset balance
		assert_eq!(
			<Test as crate::Config>::Fungibles::balance(asset_id, &account_id),
			50u128 + asset_amount_to_check
		);

		//verify new sender's currency amount
		assert_eq!(<Test as crate::Config>::Currency::free_balance(&account_id), 30u128);

		//verify new pallet's asset balance
		assert_eq!(
			<Test as crate::Config>::Fungibles::balance(asset_id, &Dex::account_id()),
			50u128 - asset_amount_to_check
		);

		//verify new pallet's currency balance
		assert_eq!(
			<Test as crate::Config>::Currency::free_balance(&Dex::account_id()),
			50u128 + 20u128
		);

		//check the last event
		System::assert_last_event(
			Event::CurrencyToAsset {
				sender: account_id,
				asset_id,
				currency_amount: 20u128,
				asset_amount: asset_amount_to_check,
			}
			.into(),
		);
	})
}

#[test]
fn currency_to_asset_fails_asset_not_found() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		let asset_id = 3u32;
		let account_id = 1u64;

		//create a sender
		let sender = RuntimeOrigin::signed(account_id);

		//fails to swap currency for an non-existent asset
		assert_noop!(
			Dex::currency_to_asset(sender, 20u128, asset_id),
			Error::<Test>::AssetNotFound
		);
	})
}

#[test]
fn currency_to_asset_fails_pool_not_found() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		let asset_id = 3u32;
		let account_id = 1u64;

		//create a sender
		let sender = RuntimeOrigin::signed(account_id);

		//create an asset
		assert_ok!(Dex::create_asset_helper(asset_id));

		//fails to swap currency because the pool does not exist
		assert_noop!(Dex::currency_to_asset(sender, 10u128, asset_id), Error::<Test>::PoolNotFound);
	})
}

#[test]
fn currency_to_asset_fails_currency_amount_zero() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		let asset_id = 3u32;
		let account_id = 1u64;

		//create a sender
		let sender = RuntimeOrigin::signed(account_id);

		//fails to swap currency for a currency amount of zero
		assert_noop!(
			Dex::currency_to_asset(sender, 0u128, asset_id),
			Error::<Test>::CurrencyAmountZero
		);
	})
}

#[test]
fn asset_to_currency_successfully() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		let asset_id = 3u32;
		let account_id = 1u64;
		let liquidity_asset_id = 2u32;

		//create a sender
		let sender = RuntimeOrigin::signed(account_id);

		//transfer currency to the sender
		let _ = <Test as crate::Config>::Currency::deposit_creating(&account_id, 100u128);

		//create an asset
		assert_ok!(Dex::create_asset_helper(asset_id));

		//mint asset to user
		assert!(
			<Test as crate::Config>::Fungibles::mint_into(asset_id, &account_id, 100u128).is_ok()
		);

		//create a pool and add liquidity to it
		assert_ok!(Dex::create_pool(sender.clone(), asset_id, liquidity_asset_id, 50u128, 50u128));

		//get the pool created previously
		let pool = Dex::pools(asset_id).unwrap_or_default();

		//calculate the asset amount to give to the sender based on the asset input amount
		let curency_amount_to_check = Dex::get_input_convert(
			20u128,
			pool.asset_reserve.clone(),
			pool.currency_reserve.clone(),
		)
		.unwrap_or_default();

		//perform the asset to currency swap operation
		assert_ok!(Dex::asset_to_currency(sender.clone(), 20u128, asset_id));

		//verify new sender's asset balance
		assert_eq!(
			<Test as crate::Config>::Fungibles::balance(asset_id, &account_id),
			50u128 - 20u128
		);

		//verify new sender's currency amount
		assert_eq!(
			<Test as crate::Config>::Currency::free_balance(&account_id),
			50u128 + curency_amount_to_check
		);

		//verify new pallet's asset balance
		assert_eq!(
			<Test as crate::Config>::Fungibles::balance(asset_id, &Dex::account_id()),
			50u128 + 20u128
		);

		//verify new pallet's currency balance
		assert_eq!(
			<Test as crate::Config>::Currency::free_balance(&Dex::account_id()),
			50u128 - curency_amount_to_check
		);

		//check the last event
		System::assert_last_event(
			Event::AssetToCurrency {
				sender: account_id,
				asset_id,
				asset_amount: 20u128,
				currency_amount: curency_amount_to_check,
			}
			.into(),
		);
	})
}

#[test]
fn asset_to_currency_fails_asset_not_found() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		let asset_id = 3u32;
		let account_id = 1u64;

		//create a sender
		let sender = RuntimeOrigin::signed(account_id);

		//fails to swap asset because the asset does not exist
		assert_noop!(
			Dex::asset_to_currency(sender, 20u128, asset_id),
			Error::<Test>::AssetNotFound
		);
	})
}

#[test]
fn asset_to_currency_fails_pool_not_found() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		let asset_id = 3u32;
		let account_id = 1u64;

		//create a sender
		let sender = RuntimeOrigin::signed(account_id);

		//create an asset
		assert_ok!(Dex::create_asset_helper(asset_id));

		//fails to swap asset because pool does not exist
		assert_noop!(Dex::asset_to_currency(sender, 10u128, asset_id), Error::<Test>::PoolNotFound);
	})
}

#[test]
fn asset_to_currency_fails_asset_amount_zero() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		let asset_id = 3u32;
		let account_id = 1u64;

		//create a sender
		let sender = RuntimeOrigin::signed(account_id);

		//fails to swap asset for an asset amount of zero
		assert_noop!(
			Dex::asset_to_currency(sender, 0u128, asset_id),
			Error::<Test>::AssetAmountZero
		);
	})
}

#[test]
fn asset_to_asset_successfully() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		let asset_id_from = 3u32;
		let asset_id_to = 5u32;
		let account_id = 1u64;
		let liquidity_asset_id_from = 2u32;
		let liquidity_asset_id_to = 4u32;

		//create a sender
		let sender = RuntimeOrigin::signed(account_id);

		//transfer currency to the sender
		let _ = <Test as crate::Config>::Currency::deposit_creating(&account_id, 300u128);

		//create the asset_from
		assert_ok!(Dex::create_asset_helper(asset_id_from));

		//create the asset_to
		assert_ok!(Dex::create_asset_helper(asset_id_to));

		//mint asset_from to user
		assert!(<Test as crate::Config>::Fungibles::mint_into(asset_id_from, &account_id, 300u128)
			.is_ok());

		//mint asset_to to user
		assert!(<Test as crate::Config>::Fungibles::mint_into(asset_id_to, &account_id, 300u128)
			.is_ok());

		//create pool_from
		assert_ok!(Dex::create_pool(
			sender.clone(),
			asset_id_from,
			liquidity_asset_id_from,
			50u128,
			50u128
		));

		//create pool_to
		assert_ok!(Dex::create_pool(
			sender.clone(),
			asset_id_to,
			liquidity_asset_id_to,
			50u128,
			50u128
		));

		//get the pool_from
		let pool_from = Dex::pools(asset_id_from).unwrap_or_default();

		//get the pool_to
		let pool_to = Dex::pools(asset_id_to).unwrap_or_default();

		//middle calculation
		let curency_amount_middle = Dex::get_input_convert(
			20u128,
			pool_from.asset_reserve.clone(),
			pool_from.currency_reserve.clone(),
		)
		.unwrap_or_default();

		let asset_final_amount = Dex::get_input_convert(
			curency_amount_middle,
			pool_to.currency_reserve.clone(),
			pool_to.asset_reserve.clone(),
		)
		.unwrap_or_default();

		let previous_balance_from =
			<Test as crate::Config>::Fungibles::balance(asset_id_from, &account_id);
		let previous_balance_to =
			<Test as crate::Config>::Fungibles::balance(asset_id_to, &account_id);

		//perform the asset to asset swap operation
		assert_ok!(Dex::asset_to_asset(sender.clone(), asset_id_from, asset_id_to, 20u128));

		//verify new sender's asset_from balance
		assert_eq!(
			<Test as crate::Config>::Fungibles::balance(asset_id_from, &account_id),
			previous_balance_from - 20u128
		);

		//verify new sender's asset_to balance
		assert_eq!(
			<Test as crate::Config>::Fungibles::balance(asset_id_to, &account_id),
			previous_balance_to + asset_final_amount
		);

		//check the last event
		System::assert_last_event(
			Event::AssetToAsset {
				sender: account_id,
				asset_id_from,
				asset_id_to,
				asset_amount: 20u128,
				asset_amount_received: asset_final_amount,
			}
			.into(),
		);
	})
}

#[test]
fn asset_to_asset_fails_asset_from_not_found() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		let asset_id_from = 3u32;
		let asset_id_to = 5u32;
		let account_id = 1u64;

		//create a sender
		let sender = RuntimeOrigin::signed(account_id);

		//fails to swap asset for an asset because asset_id_from does not exist
		assert_noop!(
			Dex::asset_to_asset(sender, asset_id_from, asset_id_to, 19u128),
			Error::<Test>::AssetNotFound
		);
	})
}

#[test]
fn asset_to_asset_fails_asset_to_not_found() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		let asset_id_from = 3u32;
		let asset_id_to = 5u32;
		let account_id = 1u64;

		//create a sender
		let sender = RuntimeOrigin::signed(account_id);

		//create the asset_from
		assert_ok!(Dex::create_asset_helper(asset_id_from));

		//fails to swap asset for an asset because asset_id_to does not exist
		assert_noop!(
			Dex::asset_to_asset(sender, asset_id_from, asset_id_to, 19u128),
			Error::<Test>::AssetNotFound
		);
	})
}

#[test]
fn asset_to_asset_fails_pool_from_not_found() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		let asset_id_from = 3u32;
		let asset_id_to = 5u32;
		let account_id = 1u64;

		//create a sender
		let sender = RuntimeOrigin::signed(account_id);

		//create the asset_from
		assert_ok!(Dex::create_asset_helper(asset_id_from));

		//create the asset_to
		assert_ok!(Dex::create_asset_helper(asset_id_to));

		//fails to swap asset for an asset because pool_from does not exist
		assert_noop!(
			Dex::asset_to_asset(sender, asset_id_from, asset_id_to, 19u128),
			Error::<Test>::PoolNotFound
		);
	})
}

#[test]
fn asset_to_asset_fails_pool_to_not_found() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		let asset_id_from = 3u32;
		let asset_id_to = 5u32;
		let account_id = 1u64;

		//create a sender
		let sender = RuntimeOrigin::signed(account_id);

		//transfer currency to the sender
		let _ = <Test as crate::Config>::Currency::deposit_creating(&account_id, 300u128);

		//create the asset_from
		assert_ok!(Dex::create_asset_helper(asset_id_from));

		//create the asset_to
		assert_ok!(Dex::create_asset_helper(asset_id_to));

		//mint asset_from to sender
		assert!(<Test as crate::Config>::Fungibles::mint_into(asset_id_from, &account_id, 300u128)
			.is_ok());
		
		//create pool_from
		assert_ok!(Dex::create_pool(
			sender.clone(),
			asset_id_from,
			8u32,
			50u128,
			50u128
		));

		//fails to swap asset for an asset because pool_to does not exist
		assert_noop!(
			Dex::asset_to_asset(sender, asset_id_from, asset_id_to, 19u128),
			Error::<Test>::PoolNotFound
		);
	})
}

#[test]
fn asset_to_asset_fails_asset_amount_zero() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		let asset_id_from = 3u32;
		let asset_id_to = 5u32;
		let account_id = 1u64;

		//create a sender
		let sender = RuntimeOrigin::signed(account_id);

		//fails to swap asset for an asset because asset_amount is zero
		assert_noop!(
			Dex::asset_to_asset(sender, asset_id_from, asset_id_to, 0u128),
			Error::<Test>::AssetAmountZero
		);
	})
}

#[test]
fn price_oracle_successfully() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		let asset_id = 3u32;
		let liquidity_asset_id = 2u32;
		let account_id = 1u64;

		//create a sender
		let sender = RuntimeOrigin::signed(account_id);

		//transfer currency to the sender
		let _ = <Test as crate::Config>::Currency::deposit_creating(&account_id, 100u128);

		//create an asset
		assert_ok!(Dex::create_asset_helper(asset_id));

		//mint asset to user
		assert!(
			<Test as crate::Config>::Fungibles::mint_into(asset_id, &account_id, 100u128).is_ok()
		);

		//create a pool and add liquidity to it
		assert_ok!(Dex::create_pool(sender, asset_id, liquidity_asset_id, 50u128, 50u128));

		let expected_oracle = OraclePrice {
			asset_id,
			asset_amount: 1,
			currency_amount: 1
		};

		//verify the oracle's behaviour is correct
		assert_eq!(Dex::price_oracle(asset_id).unwrap_or_default(), expected_oracle);
	})
}
