use crate::{mock::*, Error, Index, IndexComponent};
use frame_support::{assert_noop, assert_ok, debug};

#[test]
fn add_index() {
	new_test_ext().execute_with(|| {
		let test_index = Index {
			id: 100,
			name: "test".as_bytes().to_vec(),
			components: vec![
				IndexComponent {
					asset_id: 10001,
					weight: 1,
				},
				IndexComponent {
					asset_id: 10002,
					weight: 5,
				},
			],
		};
		// Dispatch a signed extrinsic.
		assert_ok!(StoneIndex::add_index(
			Origin::signed(TEST_ACCOUNT_ID),
			test_index.id,
			test_index.name,
			test_index.components
		));
		let out_index = StoneIndex::get_index(&test_index.id);
		assert_eq!(out_index.id, 100);
		assert_eq!(std::str::from_utf8(&out_index.name).unwrap(), "test");
		debug::info!("The index is {:?}", out_index);
	});
}

#[test]
fn buy_or_sell_non_existing_index() {
	new_test_ext().execute_with(|| {
		Assets::mint(10001, TEST_ACCOUNT_ID, 10000);
		Assets::mint(10002, TEST_ACCOUNT_ID, 100);
		assert_noop!(
			StoneIndex::buy_index(Origin::signed(TEST_ACCOUNT_ID), 999999999, 1),
			Error::<TestRuntime>::IndexNotExist
		);
		assert_noop!(
			StoneIndex::sell_index(Origin::signed(TEST_ACCOUNT_ID), 999999999, 1),
			Error::<TestRuntime>::IndexNotExist
		);
	});
}

#[test]
fn buy_too_much_index() {
	new_test_ext().execute_with(|| {
		Assets::mint(10001, TEST_ACCOUNT_ID, 10000);
		Assets::mint(10002, TEST_ACCOUNT_ID, 100);
		assert_ok!(StoneIndex::buy_index(
			Origin::signed(TEST_ACCOUNT_ID),
			TEST_INDEX_ID,
			100
		));
		assert_noop!(
			StoneIndex::buy_index(Origin::signed(TEST_ACCOUNT_ID), TEST_INDEX_ID, 5),
			Error::<TestRuntime>::InsufficientAssetBalance
		);
	});
}

#[test]
fn sell_too_much_index() {
	new_test_ext().execute_with(|| {
		Assets::mint(10001, TEST_ACCOUNT_ID, 10000);
		Assets::mint(10002, TEST_ACCOUNT_ID, 100);
		assert_noop!(
			StoneIndex::sell_index(Origin::signed(TEST_ACCOUNT_ID), TEST_INDEX_ID, 100000000),
			Error::<TestRuntime>::InsufficientIndexBalance
		);
	});
}

#[test]
fn buy_and_sell_index() {
	new_test_ext().execute_with(|| {
		Assets::mint(10001, TEST_ACCOUNT_ID, 10000);
		Assets::mint(10002, TEST_ACCOUNT_ID, 100);
		assert_ok!(StoneIndex::buy_index(
			Origin::signed(TEST_ACCOUNT_ID),
			TEST_INDEX_ID,
			5
		));
		assert_eq!(
			StoneIndex::index_balances((TEST_INDEX_ID, TEST_ACCOUNT_ID)),
			5
		);
		assert_eq!(Assets::balance(10001, TEST_ACCOUNT_ID), 9990);
		assert_eq!(Assets::balance(10002, TEST_ACCOUNT_ID), 95);

		assert_ok!(StoneIndex::sell_index(
			Origin::signed(TEST_ACCOUNT_ID),
			TEST_INDEX_ID,
			1
		));
		assert_eq!(
			StoneIndex::index_balances((TEST_INDEX_ID, TEST_ACCOUNT_ID)),
			4
		);
		assert_eq!(Assets::balance(10001, TEST_ACCOUNT_ID), 9992);
		assert_eq!(Assets::balance(10002, TEST_ACCOUNT_ID), 96);
	});
}
