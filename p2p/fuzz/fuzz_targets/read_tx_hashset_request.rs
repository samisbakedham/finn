#![no_main]
#[macro_use]
extern crate libfuzzer_sys;
extern crate finn_core;
extern crate finn_p2p;

use finn_core::ser;
use finn_p2p::msg::TxHashSetRequest;

fuzz_target!(|data: &[u8]| {
	let mut d = data.clone();
	let _t: Result<TxHashSetRequest, ser::Error> = ser::deserialize(&mut d);
});
