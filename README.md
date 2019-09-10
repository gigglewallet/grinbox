[![Build Status](https://img.shields.io/travis/gottstech/grinrelay/master.svg)](https://travis-ci.org/gottstech/grinrelay)

# Transaction Relay Service of Grin Wallet

## Bech32 address

https://github.com/gottstech/grinrelay/wiki/Bech32-Grin-Relay-Address

## Privacy considerations

* **The relay does not store data.** Grinbox does not store any data on completed transactions by design, but it would be possible for a modified version of a relay to do so and as a result build a graph of activity between addresses. Federation means that a relay only sees transactions related to its own users.

* **Your IP is your responsibility.** When you communicate with a grinbox relay, you are exposing your IP to the relay. You can obfuscate your real IP address using services such as a VPN and/or TOR or i2p.

## Documents

https://github.com/gottstech/grinrelay/wiki

## Credits

The code is based on the [Vault713 Grinbox](https://github.com/vault713/grinbox) but the main difference between them is:
- Grinbox use Base58 address format
- GrinRelay use Base32/Bech32 address format which has 4 main benefits than Base58. Please detail info [here](https://github.com/gottstech/grinrelay/wiki/Bech32-Grin-Relay-Address).

The related code taken with thanks and respect, with license details in all derived source files.

Both Vault713 Gringox and this project are using same open source licence: Apache Licence v2.0.

## License

Apache License v2.0. 
