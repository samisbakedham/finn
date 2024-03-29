# Overview #
This page is intended to be the mining guide for finn. Since finn is a fork of [finn](https://github.com/mimblewimble/finn),
[finn Miner](https://github.com/mimblewimble/finn-miner) can be used to mine finn. finn miner must be pointed at an
[finn full node](https://github.com/finnproject/finn-node). finn supports two mining algorithms: C29 (ASIC resistant) and C31+
(ASIC friendly). finn supports both of those algorithms as well, but on launch, we will support C29d (a variant of the C29
algorithm). finn's mining algorithm allows for 90% of the block rewards to go to C29 miners initially and then gradually go
down to 0% C29 two years after its launch. Since we are launching one year later, we will start at 45% C29d and gradually go
to 0% one year after launch. This is intended to keep us inline with finn. finn launched with the intent of doing a hard fork
every 6 months. We want to avoid doing that so we are likely to keep the C29d algorithm for the duration of the year that C29
is supported. We do reserve the right to do a hard fork should asics become a problem, but 6 months after launch C29 will only
account for less than 25% of the network so we hope to avoid hard forks all together.

# Procedure to mine #

1.) Setup and install a finn miner: [finn Miner](https://github.com/mimblewimble/finn-miner). You will need a GPU that has
at least 5.5 GB of VRAM to effectively mine on the network. There are many discussions about which miners are best for finn
and they all apply equally to finn since we use the same mining algorithm. Nvidia RTX 2070 Ti is a good GPU for mining C29d
and Nvidia RTX 2080 Ti is a good GPU for mining C31+, but there are many other options and C31 ASICs are on the horizon.

2.) Setup an finn full node and wallet: [finn full node](https://github.com/finnproject/finn-node) and
[finn-wallet](https://github.com/finnproject/finn-wallet).

3.) Modify your finn-node's finn-server.toml file to enable stratum server (by default this file is in
~/.finn/main/finn-server.toml:
change:
enable_stratum_server = false
to:
enable_stratum_server = true
and restart the finn-node.

4.) Start your finn-wallet listener:

```# finn-wallet listen```

5.) Modify your finn-miner.toml to point to your finn-node:
stratum_server_addr = "127.0.0.1:3416" (that is the default port of the stratum server in the finn-node)

6.) start mining:

```# finn-miner```

Note: you must be using either one of the C31 plugins or the C29d plugin. C29 will not work.

You are done and the block rewards will go to the finn-wallet instance that you setup.

# Reward Schedule #

finn has a different reward schedule from finn. finn's block reward subsidy is 60 finns per block indefinitely. finn's
block reward subsidy starts at 2.380952380 finn per block and has a halving every 4 years. After 32 halvings, the reward
subsidy is 0 finn.
