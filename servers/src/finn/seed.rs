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

//! Seeds a server with initial peers on first start and keep monitoring
//! peer counts to connect to more if neeed. Seedin strategy is
//! configurable with either no peers, a user-defined list or a preset
//! list of DNS records (the default).

use chrono::prelude::{DateTime, Utc};
use chrono::{Duration, MIN_DATE};
use finn_p2p::PeerAddr::Onion;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::collections::HashMap;
use std::net::ToSocketAddrs;
use std::sync::{mpsc, Arc};
use std::{cmp, str, thread, time};

use crate::core::global;
use crate::core::global::{FLOONET_DNS_SEEDS, MAINNET_DNS_SEEDS};
use crate::p2p;
use crate::p2p::libp2p_connection;
use crate::p2p::types::PeerAddr;
use crate::p2p::ChainAdapter;
use crate::util::StopState;

pub fn connect_and_monitor(
	p2p_server: Arc<p2p::Server>,
	capabilities: p2p::Capabilities,
	seed_list: Box<dyn Fn() -> Vec<PeerAddr> + Send>,
	preferred_peers: &[PeerAddr],
	stop_state: Arc<StopState>,
	header_cache_size: u64,
) -> std::io::Result<thread::JoinHandle<()>> {
	let preferred_peers = preferred_peers.to_vec();

	thread::Builder::new()
		.name("seed".to_string())
		.spawn(move || {
			let peers = p2p_server.peers.clone();
			let mut connect_all = false;

			// open a channel with a listener that connects every peer address sent below
			// max peer count
			let (tx, rx) = mpsc::channel();
			let seed_list = seed_list();

			// check seeds first
			connect_to_seeds_and_preferred_peers(
				peers.clone(),
				tx.clone(),
				seed_list.clone(),
				&preferred_peers,
			);

			libp2p_connection::set_seed_list(&seed_list, true);

			let mut prev = MIN_DATE.and_hms(0, 0, 0);
			let mut prev_expire_check = MIN_DATE.and_hms(0, 0, 0);
			let mut prev_ping = Utc::now();
			let mut start_attempt = 0;
			let mut connecting_history: HashMap<PeerAddr, DateTime<Utc>> = HashMap::new();
			loop {
				if stop_state.is_stopped() {
					break;
				}
				let peer_count = peers.all_peers().len();
				// Pause egress peer connection request. Only for tests.
				if stop_state.is_paused() {
					thread::sleep(time::Duration::from_secs(1));
					continue;
				}

				let connected_peers = if peer_count == 0 {
					0
				} else {
					peers.connected_peers().len()
				};

				if connected_peers == 0 {
					info!("No peers connected, trying to reconnect to seeds!");
					connect_to_seeds_and_preferred_peers(
						peers.clone(),
						tx.clone(),
						seed_list.clone(),
						&preferred_peers,
					);

					thread::sleep(time::Duration::from_secs(1));
					start_attempt = 0;
					connect_all = true;
				}

				// Check for and remove expired peers from the storage
				if peer_count > 0 && Utc::now() - prev_expire_check > Duration::hours(1) {
					peers.remove_expired();

					prev_expire_check = Utc::now();
				}

				// make several attempts to get peers as quick as possible
				// with exponential backoff
				if Utc::now() - prev > Duration::seconds(cmp::min(20, 1 << start_attempt)) {
					// try to connect to any address sent to the channel
					listen_for_addrs(
						peers.clone(),
						p2p_server.clone(),
						capabilities,
						&rx,
						&mut connecting_history,
						header_cache_size,
						connect_all,
					);
					prev = Utc::now();
					start_attempt = cmp::min(6, start_attempt + 1);

					if peer_count != 0 && connected_peers != 0 {
						connect_all = false;
					}

					if peer_count == 0 {
						// if on initial loop, we don't want to continue
						// bad network connection can cause segfaults
						// possibly linked to store unsafe calls
						// TODO: investigate root cause
						thread::sleep(time::Duration::from_secs(1));
						continue;
					}

					// monitor additional peers if we need to add more
					monitor_peers(
						peers.clone(),
						p2p_server.config.clone(),
						tx.clone(),
						&preferred_peers,
					);
				}

				if peer_count == 0 {
					thread::sleep(time::Duration::from_secs(1));
					continue;
				}

				// Ping connected peers on every 10s to monitor peers.
				if Utc::now() - prev_ping > Duration::seconds(10) {
					let total_diff = peers.total_difficulty();
					let total_height = peers.total_height();
					if let (Ok(total_diff), Ok(total_height)) = (total_diff, total_height) {
						peers.check_all(total_diff, total_height);
						prev_ping = Utc::now();
					} else {
						error!("failed to get peers difficulty and/or height");
					}
				}

				thread::sleep(time::Duration::from_secs(1));
			}
		})
}

fn monitor_peers(
	peers: Arc<p2p::Peers>,
	config: p2p::P2PConfig,
	tx: mpsc::Sender<PeerAddr>,
	preferred_peers: &[PeerAddr],
) {
	// regularly check if we need to acquire more peers  and if so, gets
	// them from db
	let total_count = peers.all_peers().len();
	let mut healthy_count = 0;
	let mut banned_count = 0;
	let mut defuncts = vec![];

	for x in peers.all_peers() {
		match x.flags {
			p2p::State::Banned => {
				let interval = Utc::now().timestamp() - x.last_banned;
				// Unban peer
				if interval >= config.ban_window() {
					if let Err(e) = peers.unban_peer(x.addr.clone()) {
						error!("failed to unban peer {}: {:?}", x.addr, e);
					}
					debug!(
						"monitor_peers: unbanned {} after {} seconds",
						x.addr, interval
					);
				} else {
					banned_count += 1;
				}
			}
			p2p::State::Healthy => healthy_count += 1,
			p2p::State::Defunct => defuncts.push(x),
		}
	}

	debug!(
		"monitor_peers: on {}:{}, {} connected ({} most_work). \
		 all {} = {} healthy + {} banned + {} defunct",
		config.host,
		config.port,
		peers.peer_count(),
		peers.most_work_peers().len(),
		total_count,
		healthy_count,
		banned_count,
		defuncts.len(),
	);

	// maintenance step first, clean up p2p server peers
	peers.clean_peers(
		config.peer_max_inbound_count() as usize,
		config.peer_max_outbound_count() as usize,
		preferred_peers,
	);

	if peers.enough_outbound_peers() {
		return;
	}

	// loop over connected peers
	// ask them for their list of peers
	let mut connected_peers: Vec<PeerAddr> = vec![];
	for p in peers.connected_peers() {
		trace!(
			"monitor_peers: {}:{} ask {} for more peers",
			config.host,
			config.port,
			p.info.addr,
		);
		let _ = p.send_peer_request(p2p::Capabilities::PEER_LIST);
		connected_peers.push(p.info.addr.clone())
	}

	// Attempt to connect to any preferred peers.
	for p in preferred_peers {
		if !connected_peers.is_empty() {
			if !connected_peers.contains(p) {
				tx.send(p.clone()).unwrap();
			}
		} else {
			tx.send(p.clone()).unwrap();
		}
	}

	// take a random defunct peer and mark it healthy: over a long period any
	// peer will see another as defunct eventually, gives us a chance to retry
	if !defuncts.is_empty() {
		defuncts.shuffle(&mut thread_rng());
		let _ = peers.update_state(defuncts[0].addr.clone(), p2p::State::Healthy);
	}

	// find some peers from our db
	// and queue them up for a connection attempt
	// intentionally make too many attempts (2x) as some (most?) will fail
	// as many nodes in our db are not publicly accessible
	let max_peer_attempts = 128;
	let new_peers = peers.find_peers(
		p2p::State::Healthy,
		p2p::Capabilities::UNKNOWN,
		max_peer_attempts as usize,
	);

	// Only queue up connection attempts for candidate peers where we
	// are confident we do not yet know about this peer.
	// The call to is_known() may fail due to contention on the peers map.
	// Do not attempt any connection where is_known() fails for any reason.
	for p in new_peers {
		if let Ok(false) = peers.is_known(p.addr.clone()) {
			tx.send(p.addr.clone()).unwrap();
		}
	}
}

// Check if we have any pre-existing peer in db. If so, start with those,
// otherwise use the seeds provided.
fn connect_to_seeds_and_preferred_peers(
	peers: Arc<p2p::Peers>,
	tx: mpsc::Sender<PeerAddr>,
	seed_list: Vec<PeerAddr>,
	peers_preferred: &[PeerAddr],
) {
	// check if we have some peers in db
	// look for peers that are able to give us other peers (via PEER_LIST capability)
	let peers = peers.find_peers(p2p::State::Healthy, p2p::Capabilities::PEER_LIST, 100);

	// if so, get their addresses, otherwise use our seeds
	let mut peer_addrs = if peers.len() > 3 {
		peers.iter().map(|p| p.addr.clone()).collect::<Vec<_>>()
	} else {
		seed_list
	};

	// If we have preferred peers add them to the initial list
	peer_addrs.extend_from_slice(peers_preferred);

	if peer_addrs.is_empty() {
		warn!("No seeds were retrieved.");
	}

	// connect to this first set of addresses
	for addr in peer_addrs {
		tx.send(addr).unwrap();
	}
}

/// Regularly poll a channel receiver for new addresses and initiate a
/// connection if the max peer count isn't exceeded. A request for more
/// peers is also automatically sent after connection.
fn listen_for_addrs(
	peers: Arc<p2p::Peers>,
	p2p: Arc<p2p::Server>,
	capab: p2p::Capabilities,
	rx: &mpsc::Receiver<PeerAddr>,
	connecting_history: &mut HashMap<PeerAddr, DateTime<Utc>>,
	header_cache_size: u64,
	attempt_all: bool,
) {
	// Pull everything currently on the queue off the queue.
	// Does not block so addrs may be empty.
	// We will take(max_peers) from this later but we want to drain the rx queue
	// here to prevent it backing up.
	let mut addrs: Vec<PeerAddr> = rx.try_iter().collect();

	if attempt_all {
		for x in peers.all_peers() {
			match x.flags {
				p2p::State::Banned => {}
				_ => {
					addrs.push(x.addr);
				}
			}
		}
	}

	// If we have a healthy number of outbound peers then we are done here.
	if peers.enough_outbound_peers() {
		return;
	}
	// Note: We drained the rx queue earlier to keep it under control.
	// Even if there are many addresses to try we will only try a bounded number of them for safety.
	let connect_min_interval = 30;
	let max_outbound_attempts = 128;
	for addr in addrs.into_iter().take(max_outbound_attempts) {
		// ignore the duplicate connecting to same peer within 30 seconds
		let now = Utc::now();
		if let Some(last_connect_time) = connecting_history.get(&addr) {
			if *last_connect_time + Duration::seconds(connect_min_interval) > now {
				debug!(
					"peer_connect: ignore a duplicate request to {}. previous connecting time: {}",
					addr,
					last_connect_time.format("%H:%M:%S%.3f").to_string(),
				);
				continue;
			} else if let Some(history) = connecting_history.get_mut(&addr) {
				*history = now;
			}
		}

		connecting_history.insert(addr.clone(), now);

		let peers_c = peers.clone();
		let p2p_c = p2p.clone();
		thread::Builder::new()
			.name("peer_connect".to_string())
			.spawn(move || {
				// if we don't have a socks port, and it's onion, don't set as defunct because
				// we don't know.
				let update_possible = if p2p_c.socks_port == 0 {
					match addr.clone() {
						Onion(_) => false,
						_ => true,
					}
				} else {
					true
				};

				if update_possible {
					match p2p_c.connect(addr.clone(), header_cache_size) {
						Ok(p) => {
							debug!("Sending peer request to {}", addr);
							if p.send_peer_request(capab).is_ok() {
								match addr {
									PeerAddr::Onion(_) => {
										if let Err(_) = libp2p_connection::add_new_peer(&addr) {
											error!("Unable to add libp2p peer {}", addr);
										}
									}
									_ => (),
								};
								let _ = peers_c.update_state(addr, p2p::State::Healthy);
							}
						}
						Err(e) => {
							debug!("Connection to the peer {} was rejected, {}", addr, e);
							let _ = peers_c.update_state(addr, p2p::State::Defunct);
						}
					}
				}
			})
			.expect("failed to launch peer_connect thread");
	}

	// shrink the connecting history.
	// put a threshold here to avoid frequent shrinking in every call
	if connecting_history.len() > 100 {
		let now = Utc::now();
		let old: Vec<_> = connecting_history
			.iter()
			.filter(|&(_, t)| *t + Duration::seconds(connect_min_interval) < now)
			.map(|(s, _)| s.clone())
			.collect();
		for addr in old {
			connecting_history.remove(&addr);
		}
	}
}

pub fn default_dns_seeds() -> Box<dyn Fn() -> Vec<PeerAddr> + Send> {
	Box::new(|| {
		let net_seeds = if global::is_floonet() {
			FLOONET_DNS_SEEDS
		} else {
			MAINNET_DNS_SEEDS
		};
		resolve_dns_to_addrs(
			&net_seeds
				.iter()
				.map(|s| {
					if s.ends_with(".onion") {
						s.to_string()
					} else {
						s.to_string()
							+ if global::is_floonet() {
								":13414"
							} else {
								":3414"
							}
					}
				})
				.collect(),
		)
	})
}

fn resolve_dns_to_addrs(dns_records: &Vec<String>) -> Vec<PeerAddr> {
	let mut addresses: Vec<PeerAddr> = vec![];
	for dns in dns_records {
		if dns.ends_with(".onion") {
			addresses.push(PeerAddr::from_str(&dns))
		} else {
			debug!("Retrieving addresses from dns {}", dns);
			match dns.to_socket_addrs() {
				Ok(addrs) => addresses.append(
					&mut addrs
						.map(PeerAddr::Ip)
						.filter(|addr| !addresses.contains(addr))
						.collect(),
				),
				Err(e) => debug!("Failed to resolve dns {:?} got error {:?}", dns, e),
			};
		}
	}
	debug!("Resolved addresses: {:?}", addresses);
	addresses
}

/// Convenience function when the seed list is immediately known. Mostly used
/// for tests.
pub fn predefined_seeds(addrs: Vec<PeerAddr>) -> Box<dyn Fn() -> Vec<PeerAddr> + Send> {
	Box::new(move || addrs.clone())
}
