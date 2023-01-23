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
		pallet_prelude::*,
		traits::{Currency, FindAuthor, LockableCurrency, ReservableCurrency},
	};
	use frame_system::pallet_prelude::*;
	use sp_consensus_aura::{Slot, AURA_ENGINE_ID};
	use sp_runtime::traits::Convert;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_aura::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Type to access the Balances Pallet.
		type Currency: Currency<Self::AccountId>
			+ ReservableCurrency<Self::AccountId>
			+ LockableCurrency<Self::AccountId>;

		/// A conversion which takes an authority id, and returns the associated account id.
		type AuthorityToAccount: Convert<Self::AuthorityId, Self::AccountId>;
	}

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Too many authorities for Aura's limits.
		TooManyAuthorities,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example of directly updating the authorities for Aura.
		#[pallet::call_index(0)]
		#[pallet::weight(0)]
		pub fn force_change_authorities(
			origin: OriginFor<T>,
			who: T::AuthorityId,
		) -> DispatchResult {
			ensure_root(origin)?;
			let mut authorities = BoundedVec::<T::AuthorityId, T::MaxAuthorities>::default();
			authorities.try_push(who).map_err(|_| Error::<T>::TooManyAuthorities)?;
			pallet_aura::Pallet::<T>::change_authorities(authorities);
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		// A function to get you an account id for the current block author.
		fn find_author() -> Option<T::AccountId> {
			let digest = frame_system::Pallet::<T>::digest();
			let pre_runtime_digests = digest.logs.iter().filter_map(|d| d.as_pre_runtime());

			for (id, mut data) in pre_runtime_digests {
				if id == AURA_ENGINE_ID {
					let slot = Slot::decode(&mut data).ok()?;
					let author_index = *slot % pallet_aura::Pallet::<T>::authorities().len() as u64;
					let validators = pallet_aura::Pallet::<T>::authorities();
					let authority = validators.get(author_index as usize).cloned()?;
					return Some(T::AuthorityToAccount::convert(authority))
				}
			}

			None
		}
	}
}
