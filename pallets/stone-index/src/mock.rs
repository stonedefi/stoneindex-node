use crate as pallet_stone_index;
use pallet_stone_index::{StoneIndex, StoneIndexComponent, Config};
use frame_support::{parameter_types, construct_runtime};
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<TestRuntime>;
type Block = frame_system::mocking::MockBlock<TestRuntime>;

// Configure a mock runtime to test the pallet.
construct_runtime!(
	pub enum TestRuntime where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Module, Call, Config, Storage, Event<T>},
		Assets: pallet_assets::{Module, Call, Event<T>},
		StoneIndexPallet: pallet_stone_index::{Module, Call, Storage, Event<T>},
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
}

impl frame_system::Config for TestRuntime {
	type BaseCallFilter = ();
	type BlockWeights = ();
	type BlockLength = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = ();
	type BlockHashCount = BlockHashCount;
	type DbWeight = ();
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = SS58Prefix;
}

impl pallet_assets::Config for TestRuntime {
	type Event = ();
	type Balance = u64;
	type AssetId = u32;
}

parameter_types! {
	pub const CustodialAccount: u64 = 250;
}

impl Config for TestRuntime {
	type Event = ();
}

pub const TEST_INDEX_ID: u32 = 1;
pub const TEST_ACCOUNT_ID: u64 = 99999;

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let config: pallet_stone_index::GenesisConfig<TestRuntime> = pallet_stone_index::GenesisConfig {
		indexes: vec![(
			TEST_INDEX_ID,
			StoneIndex {
				id: TEST_INDEX_ID,
				name: "FirstIndex".as_bytes().to_vec(),
				components: vec![
					StoneIndexComponent {
						asset_id: 10001,
						weight: 2,
					},
					StoneIndexComponent {
						asset_id: 10002,
						weight: 1,
					},
				],
				owner: TEST_ACCOUNT_ID,
			},
		)]
	};
	config.build_storage().unwrap().into()
}
