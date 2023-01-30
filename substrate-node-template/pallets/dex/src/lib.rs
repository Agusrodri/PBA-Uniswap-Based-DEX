#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {

	//imports
	use codec::EncodeLike;
	use frame_support::{
		pallet_prelude::{DispatchResult, *},
		sp_runtime::{
			traits::{
				AccountIdConversion, CheckedAdd, CheckedDiv, CheckedMul, CheckedSub, One, Zero,
			},
			FixedPointOperand,
		},
		traits::{
			fungibles::{self, *},
			tokens::Balance,
			Currency, ExistenceRequirement, LockableCurrency, ReservableCurrency,
		},
		PalletId,
	};
	use frame_system::pallet_prelude::{OriginFor, *};
	use sp_std::fmt::Debug;

	//types
	pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
	pub type BalanceOf<T> = <T as Config>::AssetBalance;
	pub type AssetIdOf<T> = <<T as Config>::Fungibles as fungibles::Inspect<
		<T as frame_system::Config>::AccountId,
	>>::AssetId;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	///pallet configuration
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Type to access the Balances Pallet.
		type Currency: Currency<Self::AccountId, Balance = Self::AssetBalance>
			+ ReservableCurrency<Self::AccountId>
			+ LockableCurrency<Self::AccountId>;

		/// Type for assets
		type Fungibles: fungibles::Inspect<
				Self::AccountId,
				AssetId = Self::AssetId,
				Balance = Self::AssetBalance,
			> + fungibles::Mutate<Self::AccountId>
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

		//types to manage the percentage fee
		#[pallet::constant]
		type Thousand: Get<BalanceOf<Self>>;

		#[pallet::constant]
		type Fee: Get<BalanceOf<Self>>;
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

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub assets: Vec<AssetIdOf<T>>,
		pub initial_amount: BalanceOf<T>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> GenesisConfig<T> {
			GenesisConfig { assets: vec![], initial_amount: <BalanceOf<T>>::one() }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			for i in &self.assets {
				//create genesis assets
				assert!(
					T::Fungibles::create(
						i.clone(),
						T::PalletId::get().into_account_truncating(),
						false,
						self.initial_amount
					)
					.is_ok(),
					"Failed creating initial assets"
				);
			}

			//deposit currency into pallet's account
			T::Currency::deposit_creating(
				&T::PalletId::get().into_account_truncating(),
				self.initial_amount,
			);
		}
	}

	//pallet storage
	#[pallet::storage]
	#[pallet::getter(fn pools)]
	pub(super) type PoolsMap<T: Config> =
		StorageMap<_, Twox64Concat, AssetIdOf<T>, PoolOf<T>, OptionQuery>;

	//pallet events
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		PoolCreated {
			asset_id: AssetIdOf<T>,
			liquidity_asset_id: AssetIdOf<T>,
		},

		LiquidityAdded {
			provider: T::AccountId,
			asset_id: AssetIdOf<T>,
			currency_amount: BalanceOf<T>,
			asset_amount: BalanceOf<T>,
			liquidity_minted: BalanceOf<T>,
		},

		LiquidityRemoved {
			provider: T::AccountId,
			asset_id: AssetIdOf<T>,
			currency_amount: BalanceOf<T>,
			asset_amount: BalanceOf<T>,
			liquidity_amount: BalanceOf<T>,
		},

		CurrencyToAsset {
			sender: T::AccountId,
			asset_id: AssetIdOf<T>,
			currency_amount: BalanceOf<T>,
			asset_amount: BalanceOf<T>,
		},

		AssetToCurrency {
			sender: T::AccountId,
			asset_id: AssetIdOf<T>,
			asset_amount: BalanceOf<T>,
			currency_amount: BalanceOf<T>,
		},

		AssetToAsset {
			sender: T::AccountId,
			asset_id_from: AssetIdOf<T>,
			asset_id_to: AssetIdOf<T>,
			asset_amount: BalanceOf<T>,
			asset_amount_received: BalanceOf<T>,
		},
	}

	//pallet errors
	#[pallet::error]
	pub enum Error<T> {
		NoneValue,

		StorageOverflow,

		//asset not found for the requested asset_id
		AssetNotFound,

		//liquidity_asset_id already exists
		AssetLiquidityIDTaken,

		//a pool with the requested asset_id is already created
		PoolAlreadyExists,

		//asset input amount cannot be zero
		AssetAmountZero,

		//asset_id requested to be created already exists
		AssetAlreadyExists,

		//pool with the requested asset_id not found
		PoolNotFound,

		//not enough fungible asset balance
		InsufficientAssetBalance,

		//not enough currency balance
		InsufficientCurrencyBalance,

		//currency amount input equal to zero
		CurrencyAmountZero,

		//the requested liquidity_asset amount to withdraw is zero
		LiqAmountZero,

		//this error occurs when a check_add, checked_sub, checked_mul or
		//checked_div operation results in an overflow
		OperationOverflow,
	}

	//pallet calls
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		//create pool
		#[pallet::call_index(0)]
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

			//verify that the currency amount is not zero
			ensure!(!currency_amount.is_zero(), Error::<T>::CurrencyAmountZero);

			//verify that liquidity asset_id does not exists
			ensure!(
				!(T::Fungibles::asset_exists(liquidity_asset_id.clone())),
				Error::<T>::AssetAlreadyExists
			);

			//verify that the asset_id is created
			ensure!(T::Fungibles::asset_exists(asset_id.clone()), Error::<T>::AssetNotFound);

			//verify that the asset_id does not have an asociate pool
			ensure!(
				PoolsMap::<T>::get::<AssetIdOf<T>>(asset_id.clone()) == None,
				Error::<T>::PoolAlreadyExists
			);

			//verify that asset_amount is not zero
			ensure!(!asset_amount.is_zero(), Error::<T>::AssetAmountZero);

			//create liquidity token
			Self::create_asset_helper(liquidity_asset_id.clone())?;

			//create pool
			let pool = Pool {
				asset_id: asset_id.clone(),
				currency_reserve: <BalanceOf<T>>::zero(),
				asset_reserve: <BalanceOf<T>>::zero(),
				liquidity_asset_id: liquidity_asset_id.clone(),
			};

			//set the liquidity asset amount to mint to the liquidity provider
			//when the pool is created, this amount is the same as the currency amount provided
			let liquidity_to_mint = currency_amount.clone();

			//add liquidity to the new pool
			Self::add_liquidity_helper(
				pool,
				currency_amount,
				asset_amount,
				liquidity_to_mint,
				sender,
			)?;

			//deposit event
			Self::deposit_event(Event::PoolCreated { asset_id, liquidity_asset_id });

			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(0)]
		pub fn add_liquidity(
			origin: OriginFor<T>,
			asset_id: AssetIdOf<T>,
			currency_amount: BalanceOf<T>,
		) -> DispatchResult {
			//verify origin signature
			let sender = ensure_signed(origin)?;

			//verify that the currency amount is not zero
			ensure!(!currency_amount.is_zero(), Error::<T>::CurrencyAmountZero);

			//verify the pool exists
			let pool = <PoolsMap<T>>::get(asset_id.clone()).ok_or(Error::<T>::PoolNotFound)?;

			//get the total issuance of liquidity token from the pool
			let asset_total_issuance =
				T::Fungibles::total_issuance(pool.liquidity_asset_id.clone());

			//calculate the asset amount starting from de currency amount
			let currency_amount = currency_amount.clone();
			let pool_currency_reserve = pool.currency_reserve.clone();

			let currency_div_currency_reserve = currency_amount
				.checked_div(&pool_currency_reserve)
				.ok_or(Error::<T>::OperationOverflow)?;

			let asset_amount = currency_div_currency_reserve
				.checked_mul(&pool.asset_reserve.clone())
				.ok_or(Error::<T>::OperationOverflow)?
				.checked_add(&One::one())
				.ok_or(Error::<T>::OperationOverflow)?;

			let liquidity_to_mint = currency_div_currency_reserve
				.checked_mul(&asset_total_issuance)
				.ok_or(Error::<T>::OperationOverflow)?;

			//add liquidity to the new pool
			Self::add_liquidity_helper(
				pool,
				currency_amount,
				asset_amount,
				liquidity_to_mint,
				sender,
			)?;

			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(0)]
		pub fn remove_liquidity(
			origin: OriginFor<T>,
			asset_id: AssetIdOf<T>,
			liquidity_amount: BalanceOf<T>,
		) -> DispatchResult {
			//verify origin signature
			let sender = ensure_signed(origin)?;

			//verify the liq amount is not zero
			ensure!(!liquidity_amount.is_zero(), Error::<T>::LiqAmountZero);

			//search the pool
			let pool = <PoolsMap<T>>::get(asset_id.clone()).ok_or(Error::<T>::PoolNotFound)?;

			//get the total issuance of liquidity asset from the pool
			let asset_total_issuance =
				T::Fungibles::total_issuance(pool.liquidity_asset_id.clone());

			//perform the calculation of the asset amount and currency amount to withdraw
			let liquidity_amount_div = liquidity_amount
				.checked_div(&asset_total_issuance)
				.ok_or(Error::<T>::OperationOverflow)?;

			let currency_amount = liquidity_amount_div
				.checked_mul(&pool.currency_reserve.clone())
				.ok_or(Error::<T>::OperationOverflow)?;

			//final asset amount to withdraw from the pool
			let asset_amount = liquidity_amount_div
				.checked_mul(&pool.asset_reserve.clone())
				.ok_or(Error::<T>::OperationOverflow)?;

			//call remove liquidity helper
			Self::remove_liquidity_helper(
				pool,
				currency_amount,
				asset_amount,
				liquidity_amount,
				sender,
			)?;

			Ok(())
		}

		#[pallet::call_index(3)]
		#[pallet::weight(0)]
		pub fn currency_to_asset(
			origin: OriginFor<T>,
			currency_amount: BalanceOf<T>,
			asset_id: AssetIdOf<T>,
		) -> DispatchResult {
			//verify origin signature
			let sender = ensure_signed(origin)?;
			let pallet_account = Self::account_id();

			//verify the currency amount is not zero
			ensure!(!currency_amount.is_zero(), Error::<T>::CurrencyAmountZero);

			//verify the asset exists
			ensure!((T::Fungibles::asset_exists(asset_id.clone())), Error::<T>::AssetNotFound);

			//verify the pool exists
			let mut pool = <PoolsMap<T>>::get(asset_id.clone()).ok_or(Error::<T>::PoolNotFound)?;

			//call convert helper function
			let asset_amount = Self::get_input_convert(
				currency_amount,
				pool.currency_reserve.clone(),
				pool.asset_reserve.clone(),
			)?;

			//transfer currency from sender to pallet
			T::Currency::transfer(
				&sender,
				&pallet_account,
				currency_amount,
				ExistenceRequirement::KeepAlive,
			)?;

			//transfer assets from pallet to sender
			T::Fungibles::transfer(
				asset_id.clone(),
				&pallet_account,
				&sender,
				asset_amount.clone(),
				true,
			)?;

			//update pool's reserves
			pool.currency_reserve = pool
				.currency_reserve
				.checked_add(&currency_amount)
				.ok_or(Error::<T>::OperationOverflow)?;
			pool.asset_reserve = pool
				.asset_reserve
				.checked_sub(&asset_amount)
				.ok_or(Error::<T>::OperationOverflow)?;

			//update pool in storage
			<PoolsMap<T>>::insert(asset_id.clone(), pool);

			//deposit event
			Self::deposit_event(Event::CurrencyToAsset {
				sender,
				asset_id,
				currency_amount,
				asset_amount,
			});

			Ok(())
		}

		#[pallet::call_index(4)]
		#[pallet::weight(0)]
		pub fn asset_to_currency(
			origin: OriginFor<T>,
			asset_amount: BalanceOf<T>,
			asset_id: AssetIdOf<T>,
		) -> DispatchResult {
			//verify origin signature
			let sender = ensure_signed(origin)?;
			let pallet_account = Self::account_id();

			//verify the asset amount is not zero
			ensure!(!asset_amount.is_zero(), Error::<T>::AssetAmountZero);

			//verify the asset exists
			ensure!((T::Fungibles::asset_exists(asset_id.clone())), Error::<T>::AssetNotFound);

			//verify the pool exists
			let mut pool = <PoolsMap<T>>::get(asset_id.clone()).ok_or(Error::<T>::PoolNotFound)?;

			//call convert helper function
			let currency_amount = Self::get_input_convert(
				asset_amount,
				pool.asset_reserve.clone(),
				pool.currency_reserve.clone(),
			)?;

			//transfer assets from sender to pallet
			T::Fungibles::transfer(
				asset_id.clone(),
				&sender,
				&pallet_account,
				asset_amount.clone(),
				true,
			)?;

			//transfer currency from pallet to sender
			T::Currency::transfer(
				&pallet_account,
				&sender,
				currency_amount,
				ExistenceRequirement::KeepAlive,
			)?;

			//update pool's reserves
			pool.currency_reserve = pool
				.currency_reserve
				.checked_sub(&currency_amount)
				.ok_or(Error::<T>::OperationOverflow)?;
			pool.asset_reserve = pool
				.asset_reserve
				.checked_add(&asset_amount)
				.ok_or(Error::<T>::OperationOverflow)?;

			//update pool in storage
			<PoolsMap<T>>::insert(asset_id.clone(), pool);

			//deposit event
			Self::deposit_event(Event::AssetToCurrency {
				sender,
				asset_id,
				asset_amount,
				currency_amount,
			});

			Ok(())
		}

		#[pallet::call_index(5)]
		#[pallet::weight(0)]
		pub fn asset_to_asset(
			origin: OriginFor<T>,
			asset_id_from: AssetIdOf<T>,
			asset_id_to: AssetIdOf<T>,
			asset_amount: BalanceOf<T>,
		) -> DispatchResult {
			//verify origin signature
			let sender = ensure_signed(origin)?;
			let pallet_account = Self::account_id();

			//verify the asset amount is not zero
			ensure!(!asset_amount.is_zero(), Error::<T>::AssetAmountZero);

			//verify the asset from exists
			ensure!((T::Fungibles::asset_exists(asset_id_from.clone())), Error::<T>::AssetNotFound);

			//verify the asset to exists
			ensure!((T::Fungibles::asset_exists(asset_id_to.clone())), Error::<T>::AssetNotFound);

			//verify the pool of asset from exists
			let mut pool_from =
				<PoolsMap<T>>::get(asset_id_from.clone()).ok_or(Error::<T>::PoolNotFound)?;

			//verify the pool of asset to exists
			let mut pool_to =
				<PoolsMap<T>>::get(asset_id_to.clone()).ok_or(Error::<T>::PoolNotFound)?;

			//first convert to currency
			let currency_amount = Self::get_input_convert(
				asset_amount.clone(),
				pool_from.asset_reserve.clone(),
				pool_from.currency_reserve.clone(),
			)?;

			//then convert from currency to asset
			let asset_final_amount = Self::get_input_convert(
				currency_amount,
				pool_to.currency_reserve.clone(),
				pool_to.asset_reserve.clone(),
			)?;

			//transfer asset_from from sender to pallet
			T::Fungibles::transfer(
				asset_id_from.clone(),
				&sender,
				&pallet_account,
				asset_amount.clone(),
				true,
			)?;

			//transfer asset_from from sender to pallet
			T::Fungibles::transfer(
				asset_id_to.clone(),
				&pallet_account,
				&sender,
				asset_final_amount.clone(),
				true,
			)?;

			//update pool_from reserves
			pool_from.currency_reserve = pool_from
				.currency_reserve
				.checked_sub(&currency_amount)
				.ok_or(Error::<T>::OperationOverflow)?;
			pool_from.asset_reserve = pool_from
				.asset_reserve
				.checked_add(&asset_amount)
				.ok_or(Error::<T>::OperationOverflow)?;

			//update pool_from in storage
			<PoolsMap<T>>::insert(asset_id_from.clone(), pool_from);

			//update pool_to reserves
			pool_to.currency_reserve = pool_to
				.currency_reserve
				.checked_add(&currency_amount)
				.ok_or(Error::<T>::OperationOverflow)?;
			pool_to.asset_reserve = pool_to
				.asset_reserve
				.checked_sub(&asset_amount)
				.ok_or(Error::<T>::OperationOverflow)?;

			//update pool_to in storage
			<PoolsMap<T>>::insert(asset_id_to.clone(), pool_to);

			//deposit event
			Self::deposit_event(Event::AssetToAsset {
				sender,
				asset_id_from,
				asset_id_to,
				asset_amount,
				asset_amount_received: asset_final_amount,
			});

			Ok(())
		}

		#[pallet::call_index(6)]
		#[pallet::weight(0)]
		pub fn mint_asset(
			origin: OriginFor<T>,
			asset_id: AssetIdOf<T>,
			asset_amount: BalanceOf<T>,
		) -> DispatchResult {
			//verify origin signature
			let sender = ensure_signed(origin)?;

			//mint funds into sender's account
			T::Fungibles::mint_into(asset_id.clone(), &sender, asset_amount)?;

			Ok(())
		}
	}

	//oracle struct
	#[derive(
		Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default, MaxEncodedLen, TypeInfo,
	)]
	pub struct OraclePrice<AssetId, AssetBalance> {
		pub asset_id: AssetId,
		pub asset_amount: AssetBalance,
		pub currency_amount: AssetBalance,
	}

	//helper pallet functions
	impl<T: Config> Pallet<T> {
		pub fn account_id() -> T::AccountId {
			T::PalletId::get().into_account_truncating()
		}

		pub fn create_asset_helper(asset_id: AssetIdOf<T>) -> DispatchResult {
			//verify the asset exists
			ensure!(
				!(T::Fungibles::asset_exists(asset_id.clone())),
				Error::<T>::AssetAlreadyExists
			);
			T::Fungibles::create(asset_id, Self::account_id(), false, <BalanceOf<T>>::one())?;
			Ok(())
		}

		pub fn add_liquidity_helper(
			mut pool: PoolOf<T>,
			currency_amount: BalanceOf<T>,
			asset_amount: BalanceOf<T>,
			liquidity_minted: BalanceOf<T>,
			provider: AccountIdOf<T>,
		) -> DispatchResult {
			let asset_id = pool.asset_id.clone();
			let pallet_account = Self::account_id();

			//transfer the respective currency amount from liquidity provider account to pallet
			// account
			T::Currency::transfer(
				&provider,
				&pallet_account,
				currency_amount,
				ExistenceRequirement::KeepAlive,
			)?;

			//transfer the respective asset amount from liquidity provider account to pallet
			// account
			T::Fungibles::transfer(
				asset_id.clone(),
				&provider,
				&pallet_account,
				asset_amount,
				true,
			)?;

			//mint liquidity assets to liquidity provider account
			T::Fungibles::mint_into(pool.liquidity_asset_id.clone(), &provider, liquidity_minted)?;

			//update pool's reserves
			pool.currency_reserve = pool
				.currency_reserve
				.checked_add(&currency_amount)
				.ok_or(Error::<T>::OperationOverflow)?;
			pool.asset_reserve = pool
				.asset_reserve
				.checked_add(&asset_amount)
				.ok_or(Error::<T>::OperationOverflow)?;

			//update pool in storage
			<PoolsMap<T>>::insert(asset_id.clone(), pool);

			Self::deposit_event(Event::LiquidityAdded {
				provider,
				asset_id,
				currency_amount,
				asset_amount,
				liquidity_minted,
			});

			Ok(())
		}

		pub fn remove_liquidity_helper(
			mut pool: PoolOf<T>,
			currency_amount: BalanceOf<T>,
			asset_amount: BalanceOf<T>,
			liquidity_amount: BalanceOf<T>,
			provider: AccountIdOf<T>,
		) -> DispatchResult {
			let asset_id = pool.asset_id.clone();
			let pallet_account = Self::account_id();

			//burn liquidity assets
			T::Fungibles::burn_from(pool.liquidity_asset_id.clone(), &provider, liquidity_amount)?;

			//transfer currency from pallet to provider
			<T as Config>::Currency::transfer(
				&pallet_account,
				&provider,
				currency_amount,
				ExistenceRequirement::AllowDeath,
			)?;

			//transfer asset from pallet to provider
			T::Fungibles::transfer(
				asset_id.clone(),
				&pallet_account,
				&provider,
				asset_amount,
				false,
			)?;

			//update pool's reserves
			pool.currency_reserve = pool
				.currency_reserve
				.checked_sub(&currency_amount)
				.ok_or(Error::<T>::OperationOverflow)?;
			pool.asset_reserve = pool
				.asset_reserve
				.checked_sub(&asset_amount)
				.ok_or(Error::<T>::OperationOverflow)?;

			//update pool in storage
			<PoolsMap<T>>::insert(asset_id.clone(), pool);

			//deposit event
			Self::deposit_event(Event::LiquidityRemoved {
				provider,
				asset_id,
				currency_amount,
				asset_amount,
				liquidity_amount,
			});

			Ok(())
		}

		pub fn get_input_convert(
			input_amount: BalanceOf<T>,
			input_reserve: BalanceOf<T>,
			output_reserve: BalanceOf<T>,
		) -> Result<BalanceOf<T>, Error<T>> {

			//Thousand and Fee are constants used to represent the percentage fee
			//The math function to calculate the asset_amount is:
			//asset_amount = ((Thousand - Fee) * ∆x * y) / ((Thousand - Fee) * x + (Thousand - Fee) * ∆x)

			//∆x = currency_amount (input_amount)
			//x = currency pool amount (input_reserve)
			//y = asset pool amount (output_reserve)

			//(Thousand - Fee)
			let percentage_less_fee = T::Thousand::get()
				.checked_sub(&T::Fee::get())
				.ok_or(Error::<T>::OperationOverflow)?;

			//(Thousand - Fee) * ∆x
			let mult_amount = percentage_less_fee
				.checked_mul(&input_amount)
				.ok_or(Error::<T>::OperationOverflow)?;

			//((Thousand - Fee) * ∆x * y)
			let numerator =
				mult_amount.checked_mul(&output_reserve).ok_or(Error::<T>::OperationOverflow)?;

			//(Thousand - Fee) * x
			let mult_reserve = percentage_less_fee
				.checked_mul(&input_reserve)
				.ok_or(Error::<T>::OperationOverflow)?;

			//((Thousand - Fee) * x + (Thousand - Fee) * ∆x)
			let denominator =
				mult_reserve.checked_add(&mult_amount).ok_or(Error::<T>::OperationOverflow)?;

			//((Thousand - Fee) * ∆x * y) / ((Thousand - Fee) * x + (Thousand - Fee) * ∆x)
			let final_amount =
			numerator.checked_div(&denominator).ok_or(Error::<T>::OperationOverflow)?;

			Ok(final_amount)
		}

		pub fn price_oracle(
			asset_id: AssetIdOf<T>,
		) -> Result<OraclePrice<AssetIdOf<T>, BalanceOf<T>>, Error<T>> {

			//get the pool associated to asset_id and extract reserves
			let pool = <PoolsMap<T>>::get(asset_id.clone()).ok_or(Error::<T>::PoolNotFound)?;
			let currency_amount = pool.currency_reserve.clone();
			let asset_amount = pool.asset_reserve.clone();

			//calculate the common minimum between both reserves 
			let minimum = currency_amount.min(asset_amount);

			//divide both reserves by the minimum
			let currency_amount =
				currency_amount.checked_div(&minimum).ok_or(Error::<T>::OperationOverflow)?;
			let asset_amount =
				asset_amount.checked_div(&minimum).ok_or(Error::<T>::OperationOverflow)?;

			//build oracle response
			let oracle = OraclePrice { asset_id, asset_amount, currency_amount };

			Ok(oracle)
		}
	}
}
