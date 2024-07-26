## 1.0.0-beta-0.0.2
- Update cosmrs dependency to 0.15 in order to get rustls updated upstream. Ran into a bug related to validating timestamps for certain certificates in one of our consumer crates. Avoided annoying pem read/write API breakages by not using the re-exported bip32 and k256 crates and depending on the old versions directly.
