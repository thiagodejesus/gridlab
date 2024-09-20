To run test coverage

Install (grcov)[https://github.com/mozilla/grcov]

run
`CARGO_INCREMENTAL=0 RUSTFLAGS='-Cinstrument-coverage' LLVM_PROFILE_FILE='cargo-test-%p-%m.profraw' cargo test`

then
`grcov . --binary-path ./target/debug/deps/ -s . -t html --branch --ignore-not-existing --ignore '../*' --ignore "/*" -o target/coverage/html`

then open the index.html at target/coverage/

on wasm_bindings run `wasm-pack build --target nodejs`

on gridlab-ts run `yarn add ../crates/wasm_bindings/pkg` then `yarn start:dev`

# Handle changes algorithm

Receive changes
Validate that all changes are able to be applied
What makes a change available to be applied is, the item that is being interacted is on the same way of the expected, and if going to somewhere, its an empty place.
If can't be applied
Unapply those changes

# Client sending changes

On the attempt to add changes, create a second grid from the actual, apply the changes, collect the events and send to the server.
The server will try to apply on the the changes and returns back what is successfuly applied



Client send change
Server receive and apply
Server broadcast changes
Client that sent the changes receives then and apply

Every change could have a hash of the grid state before and after
If the grid state is equal the after on the change, does not apply, its already synced.
Else If the grid state its equal the before on the change, apply the change
Else ask for a resync


Server_grid:
    change_history: Change[] // Could store the last x changes

Change:
    grid_hash_before: string
    grid_hash_after: string
    changes: any[]

Server receive change:
    IF change.grid_hash_before === server.grid.current_hash:
        Apply changes;
    ELIF change.grid_hash_after === server.grid.current_hash:
        By some reason a change already applied was sent by the client, just ignore;
    ELSE:
        Get all the changes on the server from the point of the hash_before of the receivedCange to now;
        IF the current_change affects the same item of any of the already applied changes || Trying to apply the current_change generates another change:
            Does not apply, as its a real conflict, and force resync of that client;
        ELSE:
            Apply that change to the current server grid, and force a resync of that client as it can be desynced;

Client receive change:
    IF grid_hash different from what is expected from that change:
        ask for a resync;