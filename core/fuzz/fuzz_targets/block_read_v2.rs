#![no_main]
extern crate finn_core;
#[macro_use]
extern crate libfuzzer_sys;

use finn_core::core::UntrustedBlock;
use finn_core::ser;

fuzz_target!(|data: &[u8]| {
	let mut d = data.clone();
	let _t: Result<UntrustedBlock, ser::Error> = ser::deserialize(&mut d, ser::ProtocolVersion(2));
});
