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
		storage::unhashed::get,
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

		/// Type for tokens
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

	//pool type for genesis config
	type PoolData<T> = (AccountIdOf<T>, AssetIdOf<T>, AssetIdOf<T>, BalanceOf<T>, BalanceOf<T>);

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
		fn build(&self) {}
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
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored {
			something: u32,
			who: T::AccountId,
		},

		PoolCreated(AssetIdOf<T>, AssetIdOf<T>),

		AddedLiquidity(T::AccountId, AssetIdOf<T>, BalanceOf<T>, BalanceOf<T>, BalanceOf<T>),

		RemovedLiquidity(T::AccountId, AssetIdOf<T>, BalanceOf<T>, BalanceOf<T>, BalanceOf<T>),
	}

	//pallet errors
	#[pallet::error]
	pub enum Error<T> {
		NoneValue,

		StorageOverflow,

		AssetNotFound,

		AssetLiquidityIDTaken,

		PoolAlreadyExists,

		AssetAmountZero,

		BalanceTooLow,

		AssetAlreadyExists,

		PoolNotFound,

		InsufficientAssetBalance,

		InsufficientCurrencyBalance,

		CurrencyAmountZero,

		DivisionFailed,

		MultiplicationFailed,

		LiqAmountZero,

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
				// exists instead
				!(T::Fungibles::asset_exists(liquidity_asset_id.clone())),
				Error::<T>::AssetAlreadyExists
			);

			//verify that the asset_id is created
			ensure!(
				// exists again
				!(T::Fungibles::total_issuance(asset_id.clone()).is_zero()),
				Error::<T>::AssetNotFound
			);

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

			Self::deposit_event(Event::PoolCreated(asset_id, liquidity_asset_id));

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

			//verify the pool exists
			let pool = <PoolsMap<T>>::get(asset_id.clone()).ok_or(Error::<T>::PoolNotFound)?;

			//verify that the currency amount is not zero
			ensure!(!currency_amount.is_zero(), Error::<T>::CurrencyAmountZero);

			//TODO: verify the sender has enough both currency and assets
			/* ensure!(
				!(T::Fungibles::balance(asset_id, &sender).is_zero()),
				Error::<T>::InsufficientAssetBalance
			); */
			ensure!(
				//free balance
				(T::Currency::total_balance(&sender).ge(&currency_amount)),
				Error::<T>::InsufficientCurrencyBalance
			);

			//get the total issuance of liquidity token from the pool
			let asset_total_issuance =
				T::Fungibles::total_issuance(pool.liquidity_asset_id.clone());
			ensure!(!asset_total_issuance.clone().is_zero(), Error::<T>::AssetNotFound);

			//calculate the asset amount starting from de currency amount
			let currency_amount = currency_amount.clone();
			let pool_currency_reserve = pool.currency_reserve.clone();

			let currency_div_currency_reserve = currency_amount
				.checked_div(&pool_currency_reserve)
				.ok_or(Error::<T>::DivisionFailed)?;
			let asset_amount = currency_div_currency_reserve
				.checked_mul(&pool.asset_reserve.clone())
				.ok_or(Error::<T>::MultiplicationFailed)?;

			let liquidity_to_mint = currency_div_currency_reserve
				.checked_mul(&asset_total_issuance)
				.ok_or(Error::<T>::MultiplicationFailed)?;

			//TODO: verify that the user's asset balance is higher than asset amount calculated
			// previously

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

			//verify that sender has enough liquidity assets
			ensure!(
				!(T::Fungibles::balance(pool.liquidity_asset_id.clone(), &sender)
					.ge(&liquidity_amount)),
				Error::<T>::InsufficientAssetBalance
			);

			//get the total issuance of liquidity token from the pool
			let asset_total_issuance =
				T::Fungibles::total_issuance(pool.liquidity_asset_id.clone());

			//perform the calculation of the asset amount and currency amount to withdraw
			let liquidity_amount_div = liquidity_amount
				.checked_div(&asset_total_issuance)
				.ok_or(Error::<T>::DivisionFailed)?;

			let currency_amount = liquidity_amount_div
				.checked_mul(&pool.currency_reserve.clone())
				.ok_or(Error::<T>::MultiplicationFailed)?;

			//final asset amount to withdraw from the pool
			let asset_amount = liquidity_amount_div
				.checked_mul(&pool.asset_reserve.clone())
				.ok_or(Error::<T>::MultiplicationFailed)?;

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

			//verify the asset exists
			ensure!(
				// exists instead
				(T::Fungibles::asset_exists(asset_id.clone())),
				Error::<T>::AssetNotFound
			);

			//verify the pool exists
			let pool = <PoolsMap<T>>::get(asset_id.clone()).ok_or(Error::<T>::PoolNotFound)?;

			//TODO?: verify the pool has enough assets
			//TODO: how do we manage fees? Are they already included in the helper?

			//verify the currency amount is not zero
			ensure!(!currency_amount.is_zero(), Error::<T>::CurrencyAmountZero);

			//call convert helper function
			let asset_amount = Self::get_input_convert(
				currency_amount,
				pool.currency_reserve.clone(),
				pool.asset_reserve.clone(),
			)?;

			//TODO: transfer currency from sender to pallet
			T::Currency::transfer(
				&sender,
				&pallet_account,
				currency_amount,
				ExistenceRequirement::KeepAlive,
			)?;

			//TODO: transfer assets from pallet to sender
			T::Fungibles::transfer(
				asset_id.clone(),
				&pallet_account,
				&sender,
				asset_amount,
				true,
			)?;

			Ok(())
		}

		#[pallet::call_index(4)]
		#[pallet::weight(0)]
		pub fn asset_to_currency(
			origin: OriginFor<T>,
			asset_amount: BalanceOf<T>,
			asset_id: AssetIdOf<T>,
		) -> DispatchResult{

			//verify origin signature
			let sender = ensure_signed(origin)?;




			Ok(())

		}
	}

	//helper pallet functions
	impl<T: Config> Pallet<T> {
		pub fn account_id() -> T::AccountId {
			T::PalletId::get().into_account_truncating()
		}

		pub fn create_asset_helper(asset_id: AssetIdOf<T>) -> DispatchResult {
			/* ensure!(
				//exists
				T::Fungibles::total_issuance(asset_id.clone()).is_zero(),
				Error::<T>::AssetAlreadyExists
			); */
			T::Fungibles::create(asset_id, Self::account_id(), false, <BalanceOf<T>>::one())?;
			Ok(())
		}

		pub fn add_liquidity_helper(
			pool: PoolOf<T>,
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
			pool.currency_reserve.checked_add(&currency_amount);
			pool.asset_reserve.checked_add(&asset_amount);

			//update pool in storage
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

		pub fn remove_liquidity_helper(
			pool: PoolOf<T>,
			currency_amount: BalanceOf<T>,
			asset_amount: BalanceOf<T>,
			liquidity_amount: BalanceOf<T>,
			provider: AccountIdOf<T>,
		) -> DispatchResult {
			let asset_id = pool.asset_id.clone();
			let pallet_account = Self::account_id();

			T::Fungibles::burn_from(pool.liquidity_asset_id.clone(), &provider, liquidity_amount)?;

			<T as Config>::Currency::transfer(
				&pallet_account,
				&provider,
				currency_amount,
				ExistenceRequirement::AllowDeath,
			)?;

			T::Fungibles::transfer(
				asset_id.clone(),
				&pallet_account,
				&provider,
				asset_amount,
				false,
			)?;

			//update pool's reserves
			pool.currency_reserve.checked_sub(&currency_amount);
			pool.asset_reserve.checked_sub(&asset_amount);

			//update pool in storage
			<PoolsMap<T>>::insert(asset_id.clone(), pool);

			Self::deposit_event(Event::RemovedLiquidity(
				provider,
				asset_id,
				currency_amount,
				asset_amount,
				liquidity_amount,
			));

			Ok(())
		}

		pub fn get_input_convert(
			input_amount: BalanceOf<T>,
			input_reserve: BalanceOf<T>,
			output_reserve: BalanceOf<T>,
		) -> Result<BalanceOf<T>, Error<T>> {
			//(997 * ∆x * y) / (1000 * x + 997 * ∆x)
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
	}
}
