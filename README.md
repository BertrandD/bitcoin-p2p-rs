A bitcoin p2p client

# Run
```bash
# First start the node:
docker-compose up -d
# Then start the client:
cargo run
```

If you want to trigger some events on the nodes to be caught by the client:
```bash
# Create wallet
docker-compose exec btcliked bitcoin-cli -regtest -rpcuser=user -rpcpassword=pass createwallet toto
# Generate an address
ADDRESS=$(docker-compose exec btcliked bitcoin-cli -regtest -rpcuser=user -rpcpassword=pass -rpcwallet=toto getnewaddress) && echo $ADDRESS
# Mine 1 block to this address 
docker-compose exec btcliked bitcoin-cli -regtest -rpcuser=user -rpcpassword=pass -rpcwallet=toto generatetoaddress 1 $ADDRESS
```
# Example output
```bash
 INFO  bitcoin_p2p_rs > Hello, world!
 INFO  bitcoin_p2p_rs > Connected to peer: 127.0.0.1:18444
 INFO  bitcoin_p2p_rs > Prepared version message (127.0.0.1:18444): VersionMessage { version: 70001, services: Ser
viceFlags(0), timestamp: 1716568240, receiver: Address {services: ServiceFlags(NONE), address: 127.0.0.1, port: 18
444}, sender: Address {services: ServiceFlags(NONE), address: 127.0.0.1, port: 48322}, nonce: 5297993460746133492,
 user_agent: "bitcoin-p2p-rs", start_height: 0, relay: false }
 INFO  bitcoin_p2p_rs > SEND: version
 DEBUG bitcoin_p2p_rs > read 342 bytes
 INFO  bitcoin_p2p_rs > RECV: version
 INFO  bitcoin_p2p_rs > SEND: verack
 DEBUG bitcoin_p2p_rs > read 32 bytes
 INFO  bitcoin_p2p_rs > RECV: ping
 INFO  bitcoin_p2p_rs > SEND: pong
 DEBUG bitcoin_p2p_rs > read 61 bytes
 INFO  bitcoin_p2p_rs > RECV: inv
 INFO  bitcoin_p2p_rs > Block: 017433e5627b6a986cd17077c97581ac3c6e7f3551408679e23a3fdb5d809882
 INFO  bitcoin_p2p_rs > SEND: getdata
 DEBUG bitcoin_p2p_rs > read 238 bytes
 INFO  bitcoin_p2p_rs > RECV: block
 INFO  bitcoin_p2p_rs > Block data: Block { header: Header { block_hash: 017433e5627b6a986cd17077c97581ac3c6e7f355
1408679e23a3fdb5d809882, version: Version(536870912), prev_blockhash: 29cd495f4d982299baeccd8d1bf6b842670188e93e2d
a04d221436a430b03add, merkle_root: 31ed20f949879b7f686a41fc58c0cc8f11b3e2c4cd97a4726683e61c6cc08805, time: 1716568
242, bits: CompactTarget(545259519), nonce: 0 }, txdata: [Transaction { version: Version(2), lock_time: 0 blocks, 
input: [TxIn { previous_output: OutPoint { txid: 0000000000000000000000000000000000000000000000000000000000000000,
 vout: 4294967295 }, script_sig: Script(OP_PUSHBYTES_1 6a OP_PUSHBYTES_1 01), sequence: Sequence(0xffffffff), witn
ess: Witness: { indices: 0, indices_start: 0, witnesses: [] } }], output: [TxOut { value: 5000000000 SAT, script_p
ubkey: Script(OP_0 OP_PUSHBYTES_20 9376e5724f5ab04b56180b073e0014e281acd9b2) }, TxOut { value: 0 SAT, script_pubke
y: Script(OP_RETURN OP_PUSHBYTES_36 aa21a9ede2f61c3f71d1defd3fa999dfa36953755c690689799962b48bebd836974e8cf9) }] }
] }
 DEBUG bitcoin_p2p_rs > read 32 bytes
 INFO  bitcoin_p2p_rs > RECV: ping
 INFO  bitcoin_p2p_rs > SEND: pong
```
