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

//! Main crate putting together all the other crates that compose finn into a
//! binary.

#![deny(non_upper_case_globals)]
#![deny(non_camel_case_types)]
#![deny(non_snake_case)]
#![deny(unused_mut)]
#![warn(missing_docs)]

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;

use finn_api as api;
use finn_chain as chain;
use finn_core as core;
use finn_keychain as keychain;
use finn_p2p as p2p;
use finn_pool as pool;
use finn_store as store;
use finn_util as util;

mod error;
pub use crate::error::{Error, ErrorKind};

pub mod common;
mod finn;
mod mining;
mod tor;

pub use crate::common::stats::{DiffBlock, PeerStats, ServerStats, StratumStats, WorkerStats};
pub use crate::common::types::{ServerConfig, StratumServerConfig};
pub use crate::core::global::{FLOONET_DNS_SEEDS, MAINNET_DNS_SEEDS};
pub use crate::finn::server::{Server, ServerTxPool, ServerVerifierCache};
