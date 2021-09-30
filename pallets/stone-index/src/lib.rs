#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::codec::{Decode, Encode};
use frame_support::{
    Parameter, decl_error, decl_event, decl_module, decl_storage, ensure, traits::Get,
};
use frame_system::ensure_signed;
use sp_runtime::traits::{Zero, StaticLookup, AtLeast32BitUnsigned, MaybeSerializeDeserialize};

use sp_std::prelude::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[derive(Encode, Decode, Default, Clone, PartialEq, Debug)]
pub struct StoneIndexComponent<AssetId> {
    pub asset_id: AssetId,
    pub weight: u32,
}

#[derive(Encode, Decode, Default, Clone, PartialEq, Debug)]
pub struct StoneIndex<IndexId, AssetId, AccountId> {
    pub id: IndexId,
    pub name: Vec<u8>,
    pub components: Vec<StoneIndexComponent<AssetId>>,
    pub owner: AccountId,
}

pub trait Config: pallet_assets::Config {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
    type IndexId: Parameter + AtLeast32BitUnsigned + Default + Copy + MaybeSerializeDeserialize;
}

// The pallet's runtime storage items.
// https://substrate.dev/docs/en/knowledgebase/runtime/storage
decl_storage! {
    // A unique name is used to ensure that the pallet's storage items are isolated.
    // This name may be updated, but each pallet in the runtime must use a unique name.
    // ---------------------------------vvvvvvvvvvvvvv
    trait Store for Module<T: Config> as StoneIndexPallet {
        Indexes get(fn indexes) config(): map hasher(blake2_128_concat) T::IndexId => StoneIndex<T::IndexId, T::AssetId, T::AccountId>;
        IndexBalances get(fn index_balances): map hasher(blake2_128_concat) (T::IndexId, T::AccountId) => T::Balance;
    }
}

// Pallets use events to inform users when important changes are made.
// https://substrate.dev/docs/en/knowledgebase/runtime/events
decl_event!(
    pub enum Event<T>
    where
        IndexId = <T as Config>::IndexId,
        Balance = <T as pallet_assets::Config>::Balance,
        AccountId = <T as frame_system::Config>::AccountId,
    {
        // [index_id, amount, who]
        BuyIndex(IndexId, Balance, AccountId),
        SellIndex(IndexId, Balance, AccountId),
        TransferIndex(IndexId, AccountId, AccountId, Balance),
    }
);

// Errors inform users that something went wrong.
decl_error! {
    pub enum Error for Module<T: Config> {
        /// The id of the index isn't existing.
        IndexNotExist,
        /// The balances of the underlying assets are insufficient.
        InsufficientAssetBalance,
        /// The balance of the index is insufficient.
        InsufficientIndexBalance,
        /// Transfer amount should be non-zero.
        TransferAmountZero,
        /// The index can only be updated by its owner
        NotTheOwner
    }
}

// Dispatchable functions allows users to interact with the pallet and invoke state changes.
// These functions materialize as "extrinsics", which are often compared to transactions.
// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        // Errors must be initialized if they are used by the pallet.
        type Error = Error<T>;

        // Events must be initialized if they are used by the pallet.
        fn deposit_event() = default;

        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn add_index(origin, #[compact] id: T::IndexId, name: Vec<u8>, components: Vec<StoneIndexComponent<T::AssetId>>) {
            let _who = ensure_signed(origin)?;

            <Indexes<T>>::insert(&id, StoneIndex {
                id,
                name,
                components,
                owner: _who
            });
        }

        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn update_index(origin, #[compact] id: T::IndexId, name: Vec<u8>, components: Vec<StoneIndexComponent<T::AssetId>>) {
            let _who = ensure_signed(origin)?;
            ensure!(<Indexes<T>>::contains_key(&id), Error::<T>::IndexNotExist);
            let index = Self::indexes(&id);
            ensure!(_who == index.owner, Error::<T>::NotTheOwner);

            <Indexes<T>>::insert(&id, StoneIndex {
                id,
                name,
                components,
                owner: _who
            });
        }

        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn buy_index(origin, #[compact] index_id: T::IndexId, #[compact] amount: T::Balance) {
            let from = ensure_signed(origin.clone())?;
            ensure!(<Indexes<T>>::contains_key(&index_id), Error::<T>::IndexNotExist);
            let index = Self::indexes(&index_id);

            for comp in index.components.iter() {
                let comp_value = amount * T::Balance::from(comp.weight);
                let asset_balance = pallet_assets::Module::<T>::balance(comp.asset_id, from.clone());
                ensure!(asset_balance >= comp_value, Error::<T>::InsufficientAssetBalance);
            }

            for comp in index.components.iter() {
                let comp_value = amount * T::Balance::from(comp.weight);
                pallet_assets::Module::<T>::burn(comp.asset_id, from.clone(), comp_value);
            }
            <IndexBalances<T>>::mutate((&index_id, &from), |balance| *balance += amount);

            Self::deposit_event(RawEvent::BuyIndex(index_id, amount, from));
        }

        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn sell_index(origin, #[compact] index_id: T::IndexId, #[compact] amount: T::Balance) {
            let from = ensure_signed(origin)?;
            ensure!(<Indexes<T>>::contains_key(&index_id), Error::<T>::IndexNotExist);
            let index = Self::indexes(&index_id);
            let index_balance = Self::index_balances((&index_id, &from));
            ensure!(index_balance >= amount, Error::<T>::InsufficientIndexBalance);

            for comp in index.components.iter() {
                let comp_value = amount * T::Balance::from(comp.weight);
                pallet_assets::Module::<T>::mint(comp.asset_id, from.clone(), comp_value);
            }
            <IndexBalances<T>>::mutate((&index_id, &from), |balance| *balance -= amount);

            Self::deposit_event(RawEvent::SellIndex(index_id, amount, from));
        }

        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn transfer(origin,
            #[compact] id: T::IndexId,
            target: <T::Lookup as StaticLookup>::Source,
            #[compact] amount: T::Balance
        ) {
            let origin = ensure_signed(origin)?;
            let origin_account = (id, origin.clone());
            let origin_balance = <IndexBalances<T>>::get(&origin_account);
            let target = T::Lookup::lookup(target)?;
            ensure!(!amount.is_zero(), Error::<T>::TransferAmountZero);
            ensure!(origin_balance >= amount, Error::<T>::InsufficientIndexBalance);

            Self::deposit_event(RawEvent::TransferIndex(id, origin.clone(), target.clone(), amount));
            Self::_transfer(id, origin, target, amount);
        }
    }
}

impl<T: Config> Module<T> {
    // Public immutables

    pub fn get_index(id: &T::IndexId) -> StoneIndex<T::IndexId, T::AssetId, T::AccountId> {
        Self::indexes(id)
    }

    pub fn _mint(index_id: T::IndexId, account: T::AccountId, amount: T::Balance) {
        <IndexBalances<T>>::mutate((index_id, account), |balance| *balance += amount);
    }

    pub fn _transfer(index_id: T::IndexId, from: T::AccountId, to: T::AccountId, amount: T::Balance) {
        <IndexBalances<T>>::mutate((index_id, from), |balance| *balance -= amount);
        <IndexBalances<T>>::mutate((index_id, to), |balance| *balance += amount);
    }
}
