// Copyright 2020 The finn Developers
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use finn_core as core;
use finn_store as store;
use finn_util as util;

use crate::core::global;
use crate::core::ser::{self, Readable, Reader, Writeable, Writer};
use std::fs;

const WRITE_CHUNK_SIZE: usize = 20;
const TEST_ALLOC_SIZE: usize = store::lmdb::ALLOC_CHUNK_SIZE_DEFAULT / 8 / WRITE_CHUNK_SIZE;

#[derive(Clone)]
struct PhatChunkStruct {
	phatness: u64,
}

impl PhatChunkStruct {
	/// create
	pub fn new() -> PhatChunkStruct {
		PhatChunkStruct { phatness: 0 }
	}
}

impl Readable for PhatChunkStruct {
	fn read<R: Reader>(reader: &mut R) -> Result<PhatChunkStruct, ser::Error> {
		let mut retval = PhatChunkStruct::new();
		for _ in 0..TEST_ALLOC_SIZE {
			retval.phatness = reader.read_u64()?;
		}
		Ok(retval)
	}
}

impl Writeable for PhatChunkStruct {
	fn write<W: Writer>(&self, writer: &mut W) -> Result<(), ser::Error> {
		// write many times
		for _ in 0..TEST_ALLOC_SIZE {
			writer.write_u64(self.phatness)?;
		}
		Ok(())
	}
}

fn clean_output_dir(test_dir: &str) {
	let _ = fs::remove_dir_all(test_dir);
}

fn setup(test_dir: &str) {
	global::set_local_chain_type(global::ChainTypes::Mainnet);
	util::init_test_logger();
	clean_output_dir(test_dir);
}

#[test]
fn lmdb_allocate() -> Result<(), store::Error> {
	let test_dir = "test_output/lmdb_allocate";
	setup(test_dir);
	// Allocate more than the initial chunk, ensuring
	// the DB resizes underneath
	{
		let store = store::Store::new(test_dir, Some("test1"), None, None)?;

		for i in 0..WRITE_CHUNK_SIZE * 2 {
			println!("Allocating chunk: {}", i);
			let chunk = PhatChunkStruct::new();
			let key_val = format!("phat_chunk_set_1_{}", i);
			let batch = store.batch()?;
			let key = store::to_key(b'P', &key_val);
			batch.put_ser(&key, &chunk)?;
			batch.commit()?;
		}
	}
	println!("***********************************");
	println!("***************NEXT*****************");
	println!("***********************************");
	// Open env again and keep adding
	{
		let store = store::Store::new(test_dir, Some("test1"), None, None)?;
		for i in 0..WRITE_CHUNK_SIZE * 2 {
			println!("Allocating chunk: {}", i);
			let chunk = PhatChunkStruct::new();
			let key_val = format!("phat_chunk_set_2_{}", i);
			let batch = store.batch()?;
			let key = store::to_key(b'P', &key_val);
			batch.put_ser(&key, &chunk)?;
			batch.commit()?;
		}
	}

	Ok(())
}
