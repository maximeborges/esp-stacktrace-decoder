## ESP Stack Trace Decoder

A Rust ESP stack trace decoder that can also **runs in your browser** thanks to WebAssembly.

It is composed of a ⌨️ Rust library, a 💻 Rust command line tool, and a 🌏 WebAssembly library with a HTML interface.

### Web tool

The web tool is hosted on [Github Pages here](https://maximeborges.github.io/esp-stacktrace-decoder/).

It is taking your `.elf` firmware and the stack trace, and outputs the list of functions and their locations, without uploading any of your data anywhere.

Everything run in your browser, ✨just like that✨.

![screenshot](https://user-images.githubusercontent.com/159235/136428494-4fdb6c69-74ca-42ab-8bf7-e26d1d625a28.png)

You can also deploy it yourself by hosting the content of the pre-compiled package `esp_exception_decoder_wasm.tar.gz` on the [release page](https://github.com/maximeborges/esp-stacktrace-decoder/releases), or by compiling the library in WebAssembly using `wasm-pack`:

    # Install the Rust toolchain by following the latest instructions from here: https://www.rust-lang.org/tools/install
    # Install wasm-pack by following the latest instructions from here: https://rustwasm.github.io/wasm-pack/installer
    # Build the WebAssembly library
    wasm-pack build --target web --out-dir web/

Note that only the `index.html`, `esp_exception_decoder_rs.js` and `esp_exception_decoder_rs_bg.wasm` from the `web/` directory are necessary.

Then you can host the content of the `web/` directory on any HTTP server. 

Here you can find a lot of different ways of starting a simple HTTP server that can be used to serves the `web/` folder: http://gist.github.com/willurd/5720255

Opening the `index.html` file from your filesystem in your browser will not work since we can't include JavaScript files without a HTTP server due to [default CORS policy](https://developer.mozilla.org/en-US/docs/Web/HTTP/CORS/Errors/CORSRequestNotHttp).

### Command line tool

A bit more boring command line tool is also available:

![esp_exception_decoder_rs_cli](https://user-images.githubusercontent.com/159235/136429806-48b82e04-cc55-4dda-84de-d143001165c3.png)

Get the latest binary release here: [Releases](https://github.com/maximeborges/esp-stacktrace-decoder/releases)

Or build it yourself: 

    # Install the Rust toolchain by following the latest instructions here: https://www.rust-lang.org/tools/install
    # Build the command line binary    
    cargo build --release

To run the command line tool, make sure that the binary is executable:

    chmod +x esp_exception_decoder

Then execute it like this, replacing `firmware.elf` with your `.elf` firmware and `stack_trace.txt` with the stack trace from your ESP:

    ./esp_exception_decoder firmware.elf stack_trace.txt

You can also ommit the stack trace file and use the standard input instead:

    cat stack_trace.txt | ./esp_exception_decoder firmware.elf

Or even use the tool semi-interactively by running the program without the stack trace file parameter, pasting the stack trace and pressing CTRL+D:

    ./esp_exception_decoder firmware.elf
    # The program is executing but not displaying anything
    # Paste the stack trace here
    # Then press CTRL+D

