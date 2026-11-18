# Splyce

Hi, I'm thebothyt, and Splyce is my take on a video and motion graphics editor for Android, written in Rust. If you like making edits on your phone, you're welcome here, whether as a creator or a contributor.

Right now Splyce has a working project hub: create projects with an aspect ratio, resolution, frame rate and canvas background, search and filter them, duplicate or delete them, and they're saved on your device. The editor itself is being built up one piece at a time.

## Planned features

- Project hub with setup presets
- Multi-track timeline with trim, split and ripple delete
- Keyframes and a curve editor
- Speed ramps and a non-destructive effect stack
- Text, captions, color grading, audio tools
- Hardware-accelerated export

## Build the APK

### With GitHub Actions (no PC needed)

1. Fork or push this repo to your GitHub account.
2. Open the **Actions** tab, pick **Build APK**, and press **Run workflow**.
3. When it finishes, download `splyce-debug-apk` from the run's artifacts.

### Locally

You'll need JDK 17, the Android SDK with NDK 26, Rust, and Gradle 8.9+.

```sh
rustup target add aarch64-linux-android armv7-linux-androideabi
cargo install cargo-ndk
cargo ndk -t arm64-v8a -t armeabi-v7a -o app/src/main/jniLibs build --release
gradle assembleDebug
```

The APK lands in `app/build/outputs/apk/debug/`.

## Contributing & Feedback

Bug reports, ideas and pull requests are all welcome. Open an issue and tell me what you'd like to see.

## License

MIT License

Copyright (c) 2026 thebothyt

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
