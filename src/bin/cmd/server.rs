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

/// finn server commands processing
use std::process::exit;
use std::thread;
use std::time::Duration;

use clap::ArgMatches;
use ctrlc;

use crate::config::GlobalConfig;
use crate::core::global;
use crate::p2p::Seeding;
use crate::servers;
use crate::tui::ui;
use finn_p2p::msg::PeerAddrs;
use finn_p2p::PeerAddr;
use finn_util::logger::LogEntry;
use std::sync::mpsc;

/// wrap below to allow UI to clean up on stop
pub fn start_server(
	config: servers::ServerConfig,
	logs_rx: Option<mpsc::Receiver<LogEntry>>,
	allow_to_stop: bool,
) {
	start_server_tui(config, logs_rx, allow_to_stop);
	// Just kill process for now, otherwise the process
	// hangs around until sigint because the API server
	// currently has no shutdown facility
	exit(0);
}

fn start_server_tui(
	config: servers::ServerConfig,
	logs_rx: Option<mpsc::Receiver<LogEntry>>,
	allow_to_stop: bool,
) {
	// Run the UI controller.. here for now for simplicity to access
	// everything it might need
	if config.run_tui.unwrap_or(false) {
		warn!("Starting MWC in UI mode...");
		servers::Server::start(
			config,
			logs_rx,
			|serv: servers::Server, logs_rx: Option<mpsc::Receiver<LogEntry>>| {
				let mut controller = ui::Controller::new(logs_rx.unwrap()).unwrap_or_else(|e| {
					panic!("Error loading UI controller: {}", e);
				});
				controller.run(serv);
			},
			allow_to_stop,
		)
		.map_err(|e| error!("Unable to start MWC in UI mode, {}", e))
		.expect("Unable to start MWC in UI mode");
	} else {
		warn!("Starting MWC w/o UI...");
		servers::Server::start(
			config,
			logs_rx,
			|serv: servers::Server, _: Option<mpsc::Receiver<LogEntry>>| {
				ctrlc::set_handler(move || {
					global::request_server_stop();
				})
				.expect("Error setting handler for both SIGINT (Ctrl+C) and SIGTERM (kill)");
				while global::is_server_running() {
					thread::sleep(Duration::from_millis(300));
				}
				warn!("Received SIGINT (Ctrl+C) or SIGTERM (kill).");
				serv.stop();
			},
			allow_to_stop,
		)
		.map_err(|e| error!("Unable to start MWC w/o UI mode, {}", e))
		.expect("Unable to start MWC w/o UI mode");
	}
}

/// Handles the server part of the command line, mostly running, starting and
/// stopping the finn blockchain server. Processes all the command line
/// arguments to build a proper configuration and runs finn with that
/// configuration.
pub fn server_command(
	server_args: Option<&ArgMatches<'_>>,
	global_config: GlobalConfig,
	logs_rx: Option<mpsc::Receiver<LogEntry>>,
) -> i32 {
	// just get defaults from the global config
	let mut server_config = global_config.members.as_ref().unwrap().server.clone();
	let mut allow_to_stop = false;

	if let Some(a) = server_args {
		if let Some(port) = a.value_of("port") {
			server_config.p2p_config.port = port.parse().unwrap();
		}

		if let Some(api_port) = a.value_of("api_port") {
			let default_ip = "0.0.0.0";
			server_config.api_http_addr = format!("{}:{}", default_ip, api_port);
		}

		if let Some(wallet_url) = a.value_of("wallet_url") {
			server_config
				.stratum_mining_config
				.as_mut()
				.unwrap()
				.wallet_listener_url = wallet_url.to_string();
		}

		if let Some(seeds) = a.values_of("seed") {
			let peers = seeds
				.filter_map(|s| s.parse().ok())
				.map(PeerAddr::Ip)
				.collect();
			server_config.p2p_config.seeding_type = Seeding::List;
			server_config.p2p_config.seeds = Some(PeerAddrs { peers });
		}

		allow_to_stop = a.is_present("allow_to_stop");
	}

	if allow_to_stop {
		warn!("Starting server with activated stop_node API");
	}

	if let Some(a) = server_args {
		match a.subcommand() {
			("run", _) => {
				start_server(server_config, logs_rx, allow_to_stop);
			}
			("", _) => {
				println!("Subcommand required, use 'mwc help server' for details");
			}
			(cmd, _) => {
				println!(":: {:?}", server_args);
				panic!(
					"Unknown server command '{}', use 'mwc help server' for details",
					cmd
				);
			}
		}
	} else {
		start_server(server_config, logs_rx, allow_to_stop);
	}
	0
}
