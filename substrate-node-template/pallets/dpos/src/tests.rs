use crate::{mock::*, Error, Event};
use frame_support::{assert_noop, assert_ok};

#[test]
fn it_works() {
	new_test_ext().execute_with(|| assert!(true));
}
