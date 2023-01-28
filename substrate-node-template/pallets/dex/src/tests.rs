use crate::{mock::*, Error, Event, Pool, PoolsMap};
use frame_support::{
	assert_noop, assert_ok,
	traits::{
		fungibles::{self, *},
		Currency,
	},
};

/* #[test]
fn it_works_for_default_value() {
	new_test_ext().execute_with(|| {
		// Go past genesis block so events get deposited
		System::set_block_number(1);
		// Dispatch a signed extrinsic.
		assert_ok!(Dex::do_something(RuntimeOrigin::signed(1), 42));
		// Read pallet storage and assert an expected result.
		assert_eq!(Dex::something(), Some(42));
		// Assert that the correct event was deposited
		System::assert_last_event(Event::SomethingStored { something: 42, who: 1 }.into());
	});
} */

/* #[test]
fn correct_error_for_none_value() {
	new_test_ext().execute_with(|| {
		// Ensure the expected error is thrown when no value is present.
		assert_noop!(Dex::cause_error(RuntimeOrigin::signed(1)), Error::<Test>::NoneValue);
	});
} */

/* #[test]
fn test_create_asset() {
	new_test_ext().execute_with(|| {
		assert_ok!(Dex::create_asset(RuntimeOrigin::signed(1), 1));
		// assert asset exists
		assert!(<<Test as crate::Config>::Fungibles as fungibles::Inspect<_>>::asset_exists(1));
	});
} */

#[test]
fn create_pool_successfull() {
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

		//create a pool and add liquidity to it
		assert_ok!(Dex::create_pool(sender, asset_id, liquidity_asset_id, 50u128, 50u128));

		//verify the sender balances changed
		assert_eq!(<Test as crate::Config>::Currency::free_balance(&account_id), 50u128);
		assert_eq!(<Test as crate::Config>::Fungibles::balance(asset_id, &account_id), 50u128);

		//read the pool and check values
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

		//fails to create a pool with an unexisting asset
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
