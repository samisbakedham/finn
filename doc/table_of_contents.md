# Documentation structure

*Read this in other languages: [Korean](translations/table_of_contents_KR.md), [简体中文](translations/table_of_contents_ZH-CN.md).*

## Explaining finn

- [intro](intro.md) - Technical introduction to finn
- [finn4bitcoiners](finn4bitcoiners.md) - Explaining finn from a bitcoiner's perspective

## Understand the finn implementation

- [chain_sync](chain/chain_sync.md) - About how finn's blockchain is synchronized
- [blocks_and_headers](chain/blocks_and_headers.md) - How finn tracks blocks and headers on the chain
- [contract_ideas](contract_ideas.md) - Ideas on how to implement contracts
- [dandelion/dandelion](dandelion/dandelion.md) - About transaction propagation and cut-through. Stemming and fluffing!
- [dandelion/simulation](dandelion/simulation.md) - Dandelion simulation - aggregating transaction without lock_height Stemming and fluffing!
- [internal/pool](internal/pool.md) - Technical explanation of the transaction pool
- [merkle](merkle.md) - Technical explanation of finn's favorite kind of merkle trees
- [merkle_proof graph](merkle_proof/merkle_proof.png) - Example merkle proof with pruning applied
- [pruning](pruning.md) - Technical explanation of pruning
- [stratum](stratum.md) - Technical explanation of finn Stratum RPC protocol
- [transaction UML](https://github.com/mimblewimble/finn-wallet/blob/master/doc/transaction/basic-transaction-wf.png) - UML of an interactive transaction (aggregating transaction without `lock_height`)
- [rangeproof output format](rangeproof_byte_format.md) - Explanation of the byte output of a range proof in a finn transaction

## Build and use

- [api](api/api.md) - Explaining the different APIs in finn and how to use them
- [build](build.md) - Explaining how to build and run the finn binaries
- [release](release_instruction.md) - Instructions of making a release
- [usage](usage.md) - Explaining how to use finn in Testnet3
- [wallet](wallet/usage.md) - Explains the wallet design and `finn wallet` sub-commands

## External (wiki)

- [FAQ](https://github.com/mimblewimble/docs/wiki/FAQ) - Frequently Asked Questions
- [Building finn](https://github.com/mimblewimble/docs/wiki/Building)
- [How to use finn](https://github.com/mimblewimble/docs/wiki/How-to-use-finn)
- [Hacking and contributing](https://github.com/mimblewimble/docs/wiki/Hacking-and-contributing)
