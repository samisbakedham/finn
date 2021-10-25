#![no_main]
#[macro_use]
extern crate libfuzzer_sys;
extern crate finn_core;
extern crate finn_p2p;

use finn_core::ser;
use finn_p2p::types::PeerAddr;

fuzz_target!(|data: &[u8]| {
	let mut d = data.clone();
	let _t: Result<PeerAddr, ser::Error> = ser::deserialize(&mut d);
});
