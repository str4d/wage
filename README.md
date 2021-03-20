# wage

A WASM package and web app for encrypting and decrypting age-encrypted files,
powered by [rage](https://github.com/str4d/rage).

Currently in beta. The WASM library and webapp are mostly complete, but their
APIs and UX are still expected to change as improvements are made. Known missing
features:

- [ ] Multi-file archive-and-encrypt.
- [ ] Optional armoring for encryption.

## Development

First, build `wage` itself as a Rust WASM package:
```
wasm-pack build
```

Then set up and run the webapp:
```
cd www
npm install
npm run serve
```

The webapp server will hot-reload on changes to the webapp itself. After
making changes to the Rust WASM package, rebuild the package and restart
the server:
```
[Ctrl+C]
cd ..
wasm-pack build
cd www
npm run serve
```

File downloading for encrypted or decrypted files will likely not work unless
you are either accessing the webapp via localhost, or have configured it with a
TLS certificate.

## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.

