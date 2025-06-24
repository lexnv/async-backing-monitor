
### Async Backing Monitor

A simple asynchronous backing monitor for Substrate-based chains, designed to track block production and relay chain interactions.

The tool inspects the backed blocks of a parachain and the corresponding relay chain blocks, providing insights into block authorship and timestamps.

```
AssetHubKusama: Block #9809277, hash=0x493c58445aeb4c7f1f822a763639933ebf847275117903373a5ae2200580b85d (elasped 4.734257689s)
  |--> Same Author: "0661757261206339b20800000000"
  |--> Timestamp.Set: "280503000b101842a29701"

  Relay Block #28933297, hash=0xb70a7ad6d7ba1894a8f78a6c077b862927a716a6e9bc70c791ff03f54ca84430 (elasped 5.172395487s)
   |--> CandidateBacked: para_head=0xbcad980e7803227417a35f364d7b2aeb5937ace90e906522ac6e95893bcf9c3d relay_parent=0x3d6b2a0fa3ea896d0e1b035633386e40a2f5e4085f1733a3571fffd98cff59f4

AssetHubKusama: Block #9809278, hash=0x5a01eb3ad02a97e9d357c376bfc8d0e0325b512e1214c63e7441f69a1893b972 (elasped 5.314589762s)
  |--> New Author: "0661757261206439b20800000000"
  |--> Timestamp.Set: "280503000b802f42a29701"

  Relay Block #28933298, hash=0x630987cbdf9a9b931af5425b0480b5e1ffa52012014ca6382828a4621d4efd40 (elasped 6.046702786s)
   |--> CandidateBacked: para_head=0x493c58445aeb4c7f1f822a763639933ebf847275117903373a5ae2200580b85d relay_parent=0x282f882c2b0c114b45e7a0d86bbf476812912b2c8cfa0f7370db79d3bfb58600

AssetHubKusama: Block #9809279, hash=0x932fcfda3619ec80d80d98322360e39d142503f59ceaed9e6f450cce6ac2ecb0 (elasped 5.994044762s)
  |--> Same Author: "0661757261206439b20800000000"
  |--> Timestamp.Set: "280503000bf04642a29701"

  Relay Block #28933299, hash=0xb2a25190efdca4687b3c626e5bceaaa948e20c8871dee3a90d641d393ab1c61a (elasped 6.032928135s)
   |--> CandidateBacked: para_head=0x5a01eb3ad02a97e9d357c376bfc8d0e0325b512e1214c63e7441f69a1893b972 relay_parent=0xb70a7ad6d7ba1894a8f78a6c077b862927a716a6e9bc70c791ff03f54ca84430

AssetHubKusama: Block #9809280, hash=0x09325afe883ce83602829f8410fa53a65076fe903373c1bbdc12d3ea8a738f2d (elasped 6.026213032s)
  |--> New Author: "0661757261206539b20800000000"
  |--> Timestamp.Set: "280503000b605e42a29701"

  Relay Block #28933300, hash=0x2d425640cf3aefd2b36cd085c9fb94f8eb078f1cccbb75f653cdc7a0cbf67233 (elasped 5.897497328s)
   |--> CandidateBacked: para_head=0x932fcfda3619ec80d80d98322360e39d142503f59ceaed9e6f450cce6ac2ecb0 relay_parent=0x630987cbdf9a9b931af5425b0480b5e1ffa52012014ca6382828a4621d4efd40

AssetHubKusama: Block #9809281, hash=0x3bc2d466da78084d9a94098d86d7df8ff122b31d110699b06a32932238b43755 (elasped 5.837205226s)
  |--> Same Author: "0661757261206539b20800000000"
  |--> Timestamp.Set: "280503000bd07542a29701"

  Relay Block #28933301, hash=0xacc7115ab7e3050bb24d72b4af245a092604a8b743532a900ebf2e1e9bd343e0 (elasped 6.694967332s)
   |--> CandidateBacked: para_head=0x09325afe883ce83602829f8410fa53a65076fe903373c1bbdc12d3ea8a738f2d relay_parent=0xb2a25190efdca4687b3c626e5bceaaa948e20c8871dee3a90d641d393ab1c61a

  Relay Block #28933301, hash=0x4f49937d6e0500ed2cb73e195f297fec28e25ce906e04a454240fcb412b52182 (elasped 420.592981ms)
   |--> CandidateBacked: para_head=0x09325afe883ce83602829f8410fa53a65076fe903373c1bbdc12d3ea8a738f2d relay_parent=0xb2a25190efdca4687b3c626e5bceaaa948e20c8871dee3a90d641d393ab1c61a

AssetHubKusama: Block #9809282, hash=0xc4614981aece577adec7c454bff5eef7d19d2fd50fb5b891d594143b9355e322 (elasped 6.697549975s)
  |--> New Author: "0661757261206639b20800000000"
  |--> Timestamp.Set: "280503000b408d42a29701"

  Relay Block #28933302, hash=0x547f2e8c9d2f24982841573d1ff5a78c6289d2bddd48542040fb917838ea6b1c (elasped 4.980345382s)
   |--> CandidateBacked: para_head=0x3bc2d466da78084d9a94098d86d7df8ff122b31d110699b06a32932238b43755 relay_parent=0x2d425640cf3aefd2b36cd085c9fb94f8eb078f1cccbb75f653cdc7a0cbf67233

  Relay Block #28933302, hash=0x3cc8b3862ca2ab17a3c979ecbefc4cad65895bce879ccd51e74bd5fdc13889f1 (elasped 627.771477ms)
   |--> CandidateBacked: para_head=0x3bc2d466da78084d9a94098d86d7df8ff122b31d110699b06a32932238b43755 relay_parent=0x2d425640cf3aefd2b36cd085c9fb94f8eb078f1cccbb75f653cdc7a0cbf67233

AssetHubKusama: Block #9809283, hash=0x601ee6b8e07fdea34888d2f883ea834300ffe5abdbe69ee09b8838b99c4feb22 (elasped 5.88773886s)
  |--> Same Author: "0661757261206639b20800000000"
  |--> Timestamp.Set: "280503000bb0a442a29701"
```