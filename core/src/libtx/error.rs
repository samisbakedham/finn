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

//! libtx specific errors
use crate::core::transaction;
use failure::{Backtrace, Context, Fail};
use keychain;
use std::fmt::{self, Display};
use util::secp;

/// Lib tx error definition
#[derive(Debug)]
pub struct Error {
	inner: Context<ErrorKind>,
}

#[derive(Clone, Debug, Eq, Fail, PartialEq, Serialize, Deserialize)]
/// Libwallet error types
pub enum ErrorKind {
	/// SECP error
	#[fail(display = "LibTx Secp Error, {}", _0)]
	Secp(secp::Error),
	/// Keychain error
	#[fail(display = "LibTx Keychain Error, {}", _0)]
	Keychain(keychain::Error),
	/// Transaction error
	#[fail(display = "LibTx Transaction Error, {}", _0)]
	Transaction(transaction::Error),
	/// Signature error
	#[fail(display = "LibTx Signature Error, {}", _0)]
	Signature(String),
	/// Rangeproof error
	#[fail(display = "LibTx Rangeproof Error, {}", _0)]
	RangeProof(String),
	/// Other error
	#[fail(display = "LibTx Other Error, {}", _0)]
	Other(String),
}

impl Fail for Error {
	fn cause(&self) -> Option<&dyn Fail> {
		self.inner.cause()
	}

	fn backtrace(&self) -> Option<&Backtrace> {
		self.inner.backtrace()
	}
}

impl Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		Display::fmt(&self.inner, f)
	}
}

impl Error {
	/// Return errorkind
	pub fn kind(&self) -> ErrorKind {
		self.inner.get_context().clone()
	}
}

impl From<ErrorKind> for Error {
	fn from(kind: ErrorKind) -> Error {
		Error {
			inner: Context::new(kind),
		}
	}
}

impl From<Context<ErrorKind>> for Error {
	fn from(inner: Context<ErrorKind>) -> Error {
		Error { inner }
	}
}

impl From<secp::Error> for Error {
	fn from(error: secp::Error) -> Error {
		Error {
			inner: Context::new(ErrorKind::Secp(error)),
		}
	}
}

impl From<keychain::Error> for Error {
	fn from(error: keychain::Error) -> Error {
		Error {
			inner: Context::new(ErrorKind::Keychain(error)),
		}
	}
}

impl From<transaction::Error> for Error {
	fn from(error: transaction::Error) -> Error {
		Error {
			inner: Context::new(ErrorKind::Transaction(error)),
		}
	}
}
