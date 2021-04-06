#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod tests {
    use frame_support::{impl_outer_origin, parameter_types, weights::Weight};
    use frame_system as system;
    use sp_core::H256;
    use sp_runtime::{
        testing::Header,
        traits::{BlakeTwo256, IdentityLookup},
        Perbill,
    };

    impl_outer_origin! {
        pub enum Origin for TestRuntime {}
    }

    // Configure a mock runtime to test the pallet.

    #[derive(Clone, Eq, PartialEq)]
    pub struct TestRuntime;
    parameter_types! {
        pub const BlockHashCount: u64 = 250;
        pub const MaximumBlockWeight: Weight = 1024;
        pub const MaximumBlockLength: u32 = 2 * 1024;
        pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
        pub const ExistentialDeposit: u64 = 1;
    }

    impl system::Trait for TestRuntime {
        type BaseCallFilter = ();
        type Origin = Origin;
        type Call = ();
        type Index = u64;
        type BlockNumber = u64;
        type Hash = H256;
        type Hashing = BlakeTwo256;
        type AccountId = u64;
        type Lookup = IdentityLookup<Self::AccountId>;
        type Header = Header;
        type Event = ();
        type BlockHashCount = BlockHashCount;
        type MaximumBlockWeight = MaximumBlockWeight;
        type DbWeight = ();
        type BlockExecutionWeight = ();
        type ExtrinsicBaseWeight = ();
        type MaximumExtrinsicWeight = MaximumBlockWeight;
        type MaximumBlockLength = MaximumBlockLength;
        type AvailableBlockRatio = AvailableBlockRatio;
        type Version = ();
        type PalletInfo = ();
        type AccountData = pallet_balances::AccountData<u64>;
        type OnNewAccount = ();
        type OnKilledAccount = ();
        type SystemWeightInfo = ();
    }

    impl pallet_assets::Trait for TestRuntime {
        type Event = ();
        type Balance = u64;
        type AssetId = u64;
    }

    impl pallet_stone_index::Trait for TestRuntime {
        type Event = ();
    }


    impl pallet_balances::Trait for TestRuntime {
        type Balance = u64;
        type Event = ();
        type DustRemoval = ();
        type ExistentialDeposit = ExistentialDeposit;
        type AccountStore = frame_system::Module<TestRuntime>;
        type WeightInfo = ();
        type MaxLocks = ();
    }

    impl pallet_swaps::Trait for TestRuntime {
        type Event = ();
        type SwapId = u64;
        type Currency = pallet_balances::Module<TestRuntime>;
    }

    impl pallet_fungible::Trait for TestRuntime {
        type Event = ();
        type TokenBalance = u64;
        type TokenId = u64;
    }

    pub type Fungible = pallet_fungible::Module<TestRuntime>;
    pub type Swaps = pallet_swaps::Module<TestRuntime>;
    pub type StoneIndex = pallet_stone_index::Module<TestRuntime>;

	fn new_test_ext() -> sp_io::TestExternalities {
		frame_system::GenesisConfig::default().build_storage::<TestRuntime>().unwrap().into()
	}

    fn swap_account_for_asset(index_id: u64) -> u64 {
        match index_id {
            10001 => 78694532,
            10002 => 67534190,
            _ => 0,
        }
    }

    fn token_id_for_index_id(index_id: u64) -> u64 {
        match index_id {
            10001 => 11,
            10002 => 12,
            _ => 0,
        }
    }

    fn deposit_index_to_swap(account: u64, index_id: u64, amount: u64) {
        let to = swap_account_for_asset(index_id);
        let token_id = token_id_for_index_id(index_id);
        StoneIndex::_transfer(index_id, account, to, amount);
        Fungible::mint(token_id, account, amount).unwrap();
    }

    fn withdraw_index_from_swap(account: u64, index_id: u64, amount: u64) {
        let from = swap_account_for_asset(index_id);
        let token_id = token_id_for_index_id(index_id);
        StoneIndex::_transfer(index_id, from, account, amount);
        Fungible::burn(token_id, account, amount).unwrap();
    }

	#[test]
	fn issuing_asset_units_to_issuer_should_work() {
		new_test_ext().execute_with(|| {
            const ACCOUNT_ID: u64 = 6798534;
            const INDEX_ID: u64 = 10001;
            let token_id: u64 = token_id_for_index_id(INDEX_ID);
            StoneIndex::_mint(INDEX_ID, ACCOUNT_ID, 50000);

            deposit_index_to_swap(ACCOUNT_ID, INDEX_ID, 20000);
            assert_eq!(StoneIndex::index_balances((INDEX_ID, ACCOUNT_ID)), 30000);
            assert_eq!(Fungible::balance_of((token_id, ACCOUNT_ID)), 20000);

            withdraw_index_from_swap(ACCOUNT_ID, INDEX_ID, 10000);
            assert_eq!(StoneIndex::index_balances((INDEX_ID, ACCOUNT_ID)), 40000);
            assert_eq!(Fungible::balance_of((token_id, ACCOUNT_ID)), 10000);
		});
	}
}