# rust-tcp-test

* Opens a server at `localhost:1234` that accepts any number of connections.
* Tries to connect to `localhost:1235`. Reconnects when connection lost.

All incoming messages over any connection are reversed and sent back.

