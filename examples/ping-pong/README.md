This is an example of how to encode and decode SV2 binary frames.

We establish a simple `Ping`-`Pong` protocol with a server and a client communicating over a TCP socket.

The server expects to receive a `Ping` message encoded as a SV2 binary frame.
The `Ping` message contains a `nonce`, which is a `u8` generated randomly by the client.

The client expects to get a `Pong` message in response, also encoded as a SV2 binary frame, with the same `nonce`.