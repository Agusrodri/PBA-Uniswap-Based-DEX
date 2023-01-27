#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		PalletId,
		pallet_prelude::{*, DispatchResult},
		sp_runtime::{
			traits::{One, Zero, Convert},
			FixedPointOperand

		}, 
		traits::{fungibles::{self, *},
				tokens::{Balance, WithdrawConsequence},
				Currency, ReservableCurrency, LockableCurrency, ExistenceRequirement}, storage::unhashed::get 
	};
	use frame_system::pallet_prelude::{*, OriginFor};
	use codec::EncodeLike;
	use frame_support::sp_runtime::traits::AccountIdConversion;
	use sp_std::fmt::Debug;
	use frame_support::sp_runtime::traits::CheckedAdd;

	pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
	pub type BalanceOf<T> = <T as Config>::AssetBalance;
	pub type AssetIdOf<T> = <<T as Config>::Fungibles as fungibles::Inspect<<T as frame_system::Config>::AccountId>>::AssetId;
	
	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Type to access the Balances Pallet.
		type Currency: Currency<Self::AccountId, Balance = Self::AssetBalance>
			+ ReservableCurrency<Self::AccountId>
			+ LockableCurrency<Self::AccountId>;
		
		/// Type for tokens
		type Fungibles: fungibles::Inspect<Self::AccountId, AssetId = Self::AssetId, Balance = Self::AssetBalance>
			+ fungibles::Mutate<Self::AccountId>
			+ fungibles::InspectMetadata<Self::AccountId>
			+ fungibles::Transfer<Self::AccountId>
			+ fungibles::Create<Self::AccountId>
			+ fungibles::Destroy<Self::AccountId>;
			
		/// The balance type for tokens
        type AssetBalance: Balance
            + FixedPointOperand
            + MaxEncodedLen
            + MaybeSerializeDeserialize
            + TypeInfo;

		/// The asset ID type.
        type AssetId: MaybeSerializeDeserialize
            + MaxEncodedLen
            + TypeInfo
            + Clone
            + Debug
            + PartialEq
            + EncodeLike
            + Decode;

		#[pallet::constant]
		type PalletId: Get<PalletId>;
	}

	#[derive(
        Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default, MaxEncodedLen, TypeInfo,
    )]
    pub struct Pool<AssetId, AssetBalance> {
        pub asset_id: AssetId,
        pub currency_reserve: AssetBalance,
        pub asset_reserve: AssetBalance,
        pub liquidity_asset_id: AssetId,
    }

	type PoolOf<T> = Pool<AssetIdOf<T>, BalanceOf<T>>;

	type PoolData<T> =
        (AccountIdOf<T>, AssetIdOf<T>, AssetIdOf<T>, BalanceOf<T>, BalanceOf<T>);

	#[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub exchanges: Vec<PoolData<T>>,
    }

	#[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> GenesisConfig<T> {
            GenesisConfig { exchanges: vec![] }
        }
    }

	#[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            
        }
    } 

	// The pallet's runtime storage items.
	// https://docs.substrate.io/main-docs/build/runtime-storage/
	#[pallet::storage]
	#[pallet::getter(fn something)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/main-docs/build/runtime-storage/#declaring-storage-items
	pub type Something<T> = StorageValue<_, u32>;

	#[pallet::storage]
    #[pallet::getter(fn pools)]
    pub(super) type PoolsMap<T: Config> =
        StorageMap<_, Twox64Concat, AssetIdOf<T>, PoolOf<T>, OptionQuery>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored { something: u32, who: T::AccountId },

		PoolCreated(AssetIdOf<T>, AssetIdOf<T>),
		AddedLiquidity(T::AccountId,
            AssetIdOf<T>,
            BalanceOf<T>,
            BalanceOf<T>,
            BalanceOf<T>,
		)
	}

	#[pallet::error]
	pub enum Error<T> {
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,

		AssetNotFound,

		AssetLiquidityIDTaken,

		PoolAlreadyExists,

		AssetAmountZero,

		BalanceTooLow,

		AssetAlreadyExists
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		//create pool
		#[pallet::call_index(2)]
		#[pallet::weight(0)]
		pub fn create_pool(
			origin: OriginFor<T>,
            asset_id: AssetIdOf<T>,
            liquidity_asset_id: AssetIdOf<T>,
            currency_amount: BalanceOf<T>,
            asset_amount: BalanceOf<T>,
		) -> DispatchResult {

			//verify origin signature
			let sender = ensure_signed(origin)?;

			//verify that liquidity asset_id does not exists
			ensure!(T::Fungibles::total_issuance(liquidity_asset_id.clone()).is_zero(), Error::<T>::AssetAlreadyExists);

			//verify that the asset_id is created 
			ensure!(!(T::Fungibles::total_issuance(asset_id.clone()).is_zero()), Error::<T>::AssetNotFound);

			//verify that the asset_id does not have an asociate pool
			ensure!(PoolsMap::<T>::get::<AssetIdOf<T>>(asset_id.clone()) == None, Error::<T>::PoolAlreadyExists);
			
			//verify that asset_amount is not zero
			ensure!(!asset_amount.is_zero(), Error::<T>::AssetAmountZero);

			//create liquidity token
			Self::create_asset_helper(liquidity_asset_id.clone())?;

			//create pool
			let pool = Pool {
				asset_id: asset_id.clone(),
				currency_reserve: <BalanceOf<T>>::zero(),
        		asset_reserve: <BalanceOf<T>>::zero(),
        		liquidity_asset_id: liquidity_asset_id.clone()
			};

			let liquidity_minted = currency_amount.clone();

			Self::add_liquidity_helper(
                pool,
                currency_amount,
                asset_amount,
                liquidity_minted,
                sender,
            )?;

			Self::deposit_event(Event::PoolCreated(asset_id, liquidity_asset_id));

			Ok(())
		}

		#[pallet::call_index(3)]
		#[pallet::weight(0)]
		pub fn create_asset(origin: OriginFor<T>, asset_id: AssetIdOf<T>) -> DispatchResult{
			let who = ensure_signed(origin)?;
			T::Fungibles::create(asset_id, who, false, <BalanceOf<T>>::one())?;
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn account_id() -> T::AccountId {
			T::PalletId::get().into_account_truncating()
		}

		pub fn create_asset_helper(asset_id: AssetIdOf<T>) -> DispatchResult{
			ensure!(T::Fungibles::total_issuance(asset_id.clone()).is_zero(), Error::<T>::AssetAlreadyExists);
			T::Fungibles::create(asset_id, Self::account_id(), false, <BalanceOf<T>>::one())?;
			Ok(())
		}

		pub fn add_liquidity_helper(
			pool: PoolOf<T>,
            currency_amount: BalanceOf<T>,
            asset_amount: BalanceOf<T>,
            liquidity_minted: BalanceOf<T>,
            provider: AccountIdOf<T>,
		) -> DispatchResult{

			let asset_id = pool.asset_id.clone();
            let pallet_account = Self::account_id();

			T::Currency::transfer(
                &provider,
                &pallet_account,
                currency_amount,
                ExistenceRequirement::KeepAlive,
            )?;
			T::Fungibles::transfer(asset_id.clone(), &provider, &pallet_account, asset_amount, true)?;
            T::Fungibles::mint_into(
                pool.liquidity_asset_id.clone(),
                &provider,
                liquidity_minted,
            )?;

			pool.currency_reserve.checked_add(&currency_amount);
            pool.asset_reserve.checked_add(&asset_amount);
            <PoolsMap<T>>::insert(asset_id.clone(), pool);

			Self::deposit_event(Event::AddedLiquidity(
                provider,
                asset_id,
                currency_amount,
                asset_amount,
                liquidity_minted,
            ));

			Ok(())

		}
	}
}
