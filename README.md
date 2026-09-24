## boa Native Messaging host

> [boa](https://github.com/boa-dev/boa)
>
> Boa is an experimental JavaScript lexer, parser and interpreter written in Rust 🦀, it has support for more than 90% of the latest ECMAScript specification. We continuously improve the conformance to keep up with the ever-evolving standard.

### Compile 
#### Native executable
```shell
cargo build --release
```

#### wasm32-wasip1
```shell
cargo build --release --target wasm32-wasip1
```

### Installation and usage on Chrome and Chromium

1. Navigate to `chrome://extensions`.
2. Toggle `Developer mode`.
3. Click `Load unpacked`.
4. Select `native-messaging-boa` folder.
5. Note the generated extension ID.
6. Open `nm_boa.json` in a text editor, set `"path"` to absolute path of `nm_boa` (native executable), or `nm_boa.sh` (shellscript to execute `wasmtime nm_boa.wasm`) and `chrome-extension://<ID>/` using ID from 5 in `"allowed_origins"` array; and make sure `wasmtime` is in `PATH` and `nm_boa.sh` is executable (when executing `nm_boa.wasm` with a WASM runtime).
7. Copy the `nm_boa.json` file to Chrome or Chromium configuration folder, e.g., Chromium on Linux `~/.config/chromium/NativeMessagingHosts`; Chrome dev channel on Linux `~/.config/google-chrome-unstable/NativeMessagingHosts`.
8. To test click `service worker` link in panel of unpacked extension which is DevTools for `background.js` in MV3 `ServiceWorker`, observe echo'ed message from `boa` Native Messaging host. To disconnect run `port.disconnect()`.

The Native Messaging host echoes back the message passed. 

For differences between OS and browser implementations see [Chrome incompatibilities](https://developer.mozilla.org/en-US/docs/Mozilla/Add-ons/WebExtensions/Chrome_incompatibilities#native_messaging).

## License
Do What the Fuck You Want to Public License [WTFPLv2](http://www.wtfpl.net/about/)
