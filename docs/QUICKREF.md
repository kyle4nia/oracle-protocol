# Oracle Protocol — Quick Reference (copy-paste values)

## Covenant addresses (TN10, 68-byte declarative v3)
rep=100 genesis : kaspatest:pr64eayfczjzmrkk4s68cmt7kr7r7ejrx54sfvqvw86krlpst6u47xznrlg23
rep=105 target  : kaspatest:prdtal22v0mwz6l0gn3rl6krhpzx0q5l5qa2n2k3unrguxfr903egtn9c7y0s

## Genesis UTXO on TN10 (funded via faucet)
txid        : a20732bcf0bec7fca9079c5fe0ca0708f580511896ab5ea1df4a9e89c7af3761
index       : 0
outpoint    : a20732bcf0bec7fca9079c5fe0ca0708f580511896ab5ea1df4a9e89c7af3761:0
amount      : 100,000 TKAS  =  10,000,000,000,000 sompi
address     : rep=100 genesis (above)

## Covenant ABI (from silverc)
entrypoint     : __covenant_entrypoint_auth_update
call arg       : delta (int)
without_selector: true
state_layout   : start 0, len 9   (8 state bytes + 1-byte OpData8 prefix)
state encoding : little-endian; rep value in redeem-script byte index 1

## 68-byte redeem script (rep=100; rep=105 = same but byte[1]=105)
8,100,0,0,0,0,0,0,0,118,82,121,147,118,0,162,105,185,203,81,156,105,118,88,205,1,8,124,126,185,118,201,118,1,68,148,89,147,124,188,126,170,2,0,0,1,170,126,1,32,126,124,126,1,135,126,185,0,204,195,135,105,0,122,117,117,117,81

## Network endpoints
PsychoNode (TN10 node)   : 192.168.0.206:17210   (borsh wRPC)
big daddy (dev box)      : 192.168.0.120
karrrlskaspanode (mainnet bridge) : kaspad RPC localhost:16110, stratum 5555-5560

## rusty-kaspa pin
tag : tn10-toc3

## Key paths (big daddy)
source of truth   : C:\oracle-protocol
covenants (.sil)  : C:\oracle-protocol\covenants
rust harnesses    : C:\oracle-protocol\rust-examples
docs + helpers    : C:\oracle-protocol\docs
silverscript      : C:\kaspa-tn12\silverscript
ctor v3 / 105     : C:\kaspa-tn12\silverscript\oracle_ctor_v3.json  /  oracle_ctor_105.json
rusty-kaspa clone : C:\kaspa-tn12\rusty-kaspa
examples build dir: C:\kaspa-tn12\rusty-kaspa\crypto\txscript\examples

## Helper scripts (in C:\oracle-protocol\docs)
check-updates.bat              : pre-session upstream-change check
covenant-check.bat             : recompile + diff covenant vs baseline (IDENTICAL/DIFFERENT)
covenant-check.bat rebaseline  : update baseline after a verified intentional change
deploy-and-run.bat <example>   : copy harness to build dir + cargo run
