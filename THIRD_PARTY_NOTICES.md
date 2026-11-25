# Third-Party Notices

## Horizon Android

Horizon Android is an independent upstream project:
https://github.com/MocLG/horizon-android

The upstream `LICENSE` identifies the copyright holders as:

> Copyright (c) 2026 Ludovic Marie and the Horizon contributors

The upstream project is licensed under the MIT License. This attribution and
license text are recorded here for clarity; Horizon source and its game client
are not included in the current RustCraft source tree.

```text
MIT License

Copyright (c) 2026 Ludovic Marie and the Horizon contributors

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

## Rust JNI dependency

`jni` 0.21.1 is declared in `rust/Cargo.toml` and licensed under MIT or
Apache-2.0. Its upstream MIT notice is:

```text
The MIT License (MIT)

Copyright (c) 2016 Prevoty, Inc. and jni-rs contributors

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

The `jni` Apache-2.0 option is covered by the complete Apache License 2.0 text
in the repository's [LICENSE](./LICENSE). Other direct and transitive Rust
dependencies and their upstream notices remain governed by the license
expressions in their package manifests; dependency versions are recorded in
`rust/Cargo.lock`.

## License scope notes

- The root `LICENSE` is Apache-2.0. Separately, `rust/Cargo.toml` currently
  declares the RustCraft crate as MIT, but this checkout contains no
  crate-specific MIT copyright notice or license text. The package author
  metadata is not used to infer a copyright holder.
- No “LP2.0” license text or component assignment was found in this repository
  or the checked Horizon upstream notices. Horizon's upstream notices identify
  GPL-2.0 with the Classpath Exception only for the OpenJDK 25 runtime carried
  by Horizon's Android build; that runtime is not included in this RustCraft
  source tree.
- Minecraft and its game assets are proprietary materials, and Minecraft,
  Mojang, and Microsoft names and marks are not covered by the licenses above.
