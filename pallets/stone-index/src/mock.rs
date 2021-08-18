use crate::{GenesisConfig, StoneIndex, StoneIndexComponent, Module, Trait};
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
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
}

impl pallet_assets::Trait for TestRuntime {
	type Event = ();
	type Balance = u64;
	type AssetId = u32;
}

impl Trait for TestRuntime {
	type Event = ();
}

pub type StoneIndexPallet = Module<TestRuntime>;
pub type Assets = pallet_assets::Module<TestRuntime>;

pub const TEST_INDEX_ID: u32 = 1;
pub const TEST_ACCOUNT_ID: u64 = 99999;

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let config: GenesisConfig<TestRuntime> = GenesisConfig {
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
