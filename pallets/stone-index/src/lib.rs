#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, ensure, traits::Get,
};
use frame_system::ensure_signed;
use sp_runtime::traits::{Zero, StaticLookup};
use sp_std::prelude::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[derive(Encode, Decode, Default, Clone, PartialEq, Debug)]
pub struct IndexComponent<AssetId> {
    asset_id: AssetId,
    weight: u32,
}

#[derive(Encode, Decode, Default, Clone, PartialEq, Debug)]
pub struct Index<AssetId> {
    id: AssetId,
    name: Vec<u8>,
    components: Vec<IndexComponent<AssetId>>,
}

pub trait Trait: pallet_assets::Trait {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
}

// The pallet's runtime storage items.
// https://substrate.dev/docs/en/knowledgebase/runtime/storage
decl_storage! {
    // A unique name is used to ensure that the pallet's storage items are isolated.
    // This name may be updated, but each pallet in the runtime must use a unique name.
    // ---------------------------------vvvvvvvvvvvvvv
    trait Store for Module<T: Trait> as StoneIndex {
        Indexes get(fn indexes) config(): map hasher(blake2_128_concat) T::AssetId => Index<T::AssetId>;
        IndexBalances get(fn index_balances): map hasher(blake2_128_concat) (T::AssetId, T::AccountId) => T::Balance;
    }
}

// Pallets use events to inform users when important changes are made.
// https://substrate.dev/docs/en/knowledgebase/runtime/events
decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Trait>::AccountId,
        AssetId = <T as pallet_assets::Trait>::AssetId,
        Balance = <T as pallet_assets::Trait>::Balance,
    {
        // [index_id, amount, who]
        BuyIndex(AssetId, Balance, AccountId),
        SellIndex(AssetId, Balance, AccountId),
		TransferIndex(AssetId, AccountId, AccountId, Balance),
    }
);

// Errors inform users that something went wrong.
decl_error! {
    pub enum Error for Module<T: Trait> {
        /// Error names should be descriptive.
        IndexNotExist,
        InsufficientAssetBalance,
        InsufficientIndexBalance,
        TransferAmountZero
    }
}

// Dispatchable functions allows users to interact with the pallet and invoke state changes.
// These functions materialize as "extrinsics", which are often compared to transactions.
// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        // Errors must be initialized if they are used by the pallet.
        type Error = Error<T>;

        // Events must be initialized if they are used by the pallet.
        fn deposit_event() = default;

        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn add_index(origin, id: T::AssetId, name: Vec<u8>, components: Vec<IndexComponent<T::AssetId>>) {
            let _who = ensure_signed(origin)?;

            <Indexes<T>>::insert(&id, Index {
                id,
                name,
                components
            });
        }

        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn buy_index(origin, index_id: T::AssetId, amount: T::Balance) {
            let origin = ensure_signed(origin)?;
            ensure!(<Indexes<T>>::contains_key(&index_id), Error::<T>::IndexNotExist);
            let index = Self::indexes(&index_id);

            for comp in index.components.iter() {
                let comp_value = amount * T::Balance::from(comp.weight);
                let asset_balance = pallet_assets::Module::<T>::balance(comp.asset_id, origin.clone());
                ensure!(asset_balance >= comp_value, Error::<T>::InsufficientAssetBalance);
            }

            for comp in index.components.iter() {
                let comp_value = amount * T::Balance::from(comp.weight);
                pallet_assets::Module::<T>::burn(comp.asset_id, origin.clone(), comp_value);
            }
            <IndexBalances<T>>::mutate((&index_id, &origin), |balance| *balance += amount);

            Self::deposit_event(RawEvent::BuyIndex(index_id, amount, origin));
        }

        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn sell_index(origin, index_id: T::AssetId, amount: T::Balance) {
            let origin = ensure_signed(origin)?;
            ensure!(<Indexes<T>>::contains_key(&index_id), Error::<T>::IndexNotExist);
            let index = Self::indexes(&index_id);
            let index_balance = Self::index_balances((&index_id, &origin));
            ensure!(index_balance >= amount, Error::<T>::InsufficientIndexBalance);

            for comp in index.components.iter() {
                let comp_value = amount * T::Balance::from(comp.weight);
                pallet_assets::Module::<T>::mint(comp.asset_id, origin.clone(), comp_value);
            }
            <IndexBalances<T>>::mutate((&index_id, &origin), |balance| *balance -= amount);

            Self::deposit_event(RawEvent::SellIndex(index_id, amount, origin));
        }

        #[weight = 0]
		fn transfer(origin,
			#[compact] id: T::AssetId,
			target: <T::Lookup as StaticLookup>::Source,
			#[compact] amount: T::Balance
		) {
			let origin = ensure_signed(origin)?;
			let origin_account = (id, origin.clone());
			let origin_balance = <IndexBalances<T>>::get(&origin_account);
			let target = T::Lookup::lookup(target)?;
			ensure!(!amount.is_zero(), Error::<T>::TransferAmountZero);
			ensure!(origin_balance >= amount, Error::<T>::InsufficientIndexBalance);

			Self::deposit_event(RawEvent::TransferIndex(id, origin, target.clone(), amount));
			<IndexBalances<T>>::insert(origin_account, origin_balance - amount);
			<IndexBalances<T>>::mutate((id, target), |balance| *balance += amount);
		}
    }
}

impl<T: Trait> Module<T> {
    // Public immutables

    pub fn get_index(id: &T::AssetId) -> Index<T::AssetId> {
        Self::indexes(id)
    }
}
