//! Benchmarking setup for pallet-dpos

use super::*;

#[allow(unused)]
use crate::Pallet as Dpos;
use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_system::RawOrigin;

benchmarks! {
	do_something {
	}: { }
	verify {
	}

	impl_benchmark_test_suite!(Dpos, crate::mock::new_test_ext(), crate::mock::Test);
}
