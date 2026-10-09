## Overview

`wacky` is a tool for injecting shims into WebAssembly by modifying the wac scripts that generate them, mainly in order to insert access control layers around untrusted components.
 The tool is built on top of `wac`, which composes subcomponents into a single large component.

This is a very early prototype and should not be used for critical tasks.

## Examples

 The following is an example of a wac file used to compose two components together, 

Before running `wacky`
```wit
package example:composition;


// Instantiate the `exporter` component
let exporter = new docs:writetwo {...};

// Instantiate the `untrusted component` 
let importer = new component:file {
  writer: exporter.writer,
  ...,
};

export importer.run;
```
After running `wacky` with configuartion of adding shim to "component:file", the above wac file becomes:

```wit
package example:composition;
// Instantiate the `exporter` component
let exporter = new docs:writetwo { ... };

// Instantiate the `shim` component instead of previous exporter 
let exportershim1 = new shim:writershim {
    writer: exporter.writer,
    ...
};

// Instantiate the untrusted component with shim
let importer = new component:file {
    writer: exportershim1.writer,
    ...
};

export importer.run;

```
 Example Two: Another example showing nested layers of wacky.
 Before running wacky:

```wit
package example:composition;

let exporter = new component:trustedwriter {...};

let untrustedcomponent = new component:fileloader {
  writer: exporter.writer,
  ...,
};


let trustedcomponent = new component:trusted {
  writer: exporter.writer,
  fileaccess: untrustedcomponent.fileaccess,
  ...,
};

export trustedcomponent.run;

```

After running wacky:
```wit
package example:composition;

let exporter = new component:trustedwriter { ... };

let exportershim1 = new shim:writershim {
    writer: exporter.writer,
    ...
};

let untrustedcomponent = new component:fileloader {
    writer: exportershim1.writer,
    ...
};

let trustedcomponent = new component:trusted {
    writer: exporter.writer,
    fileaccess: untrustedcomponent.fileaccess,
    ...
};

export trustedcomponent.run;

```

Third example to showcase wacky run on implicit interfaces:
Before running wacky implicit case:

```wit
package example:composition;


let untrustedcomponent = new component:fileloader {...};

export untrustedcomponent.run;

```
After running wacky implicit case:
```wit
package example:composition;

let untrustedcomponentimplicitshim1 = new shim:writershim { ... };

let untrustedcomponent = new component:fileloader {
    ...untrustedcomponentimplicitshim1,
};

export untrustedcomponent.run;

```

Fourth combined example:
Before running wasi
```wit
package example:composition;

let exporter = new component:trustedwriter {...};

let untrustedcomponent = new component:fileloader {
  writer: exporter.writer,
  ...,
};


let readertest = new component:filereading  {...};

let trustedcomponent = new component:trustedwriter {
  writer: exporter.writer,
  fileaccess: untrustedcomponent.fileaccess,
  ...,
};

export trustedcomponent.run;
```
After running wacky on combined:
```wit
package example:composition;

let exporter = new component:trustedwriter { ... };

let exportershim1 = new shim:writershim {
    writer: exporter.writer,
    ...
};

let readertestimplicitshim1 = new docs:readershim { ... };

let untrustedcomponent = new component:fileloader {
    writer: exportershim1.writer,
    ...
};

let readertest = new component:filereading {
    ...readertestimplicitshim1,
};

let trustedcomponent = new component:trustedwriter {
    writer: exporter.writer,
    fileaccess: untrustedcomponent.fileaccess,
    ...
};

export trustedcomponent.run;
```


A full example is already placed out when running wacky on compose.wac. 
All components used in compose.wac and output have their binaries placed in deps according to wac requirments and their source code in directory "Components Source Code". 

Samples of input, output and config files have been placed in respective directories.


 ## Dependencies
The program expects all of the files mentioned below to be provided with the use of `--cp` and `--wp` for  untrusted toml configuration file and an alternative wac path respectively.

The `wacky` tool has the following files:

* `config.toml` - Contains the untrusted component along with the shimming parameters. An example file has been placed in root directory.
* `compose.wac` - Original wac script file that we wish to investigate for untrusted components and shim if found.
* `shimmed_script.wac` - Output of `wacky` , produced after running the program. It contains the updated component links needed to inject the shim.

This project also uses a fork of [wac](https://github.com/bytecodealliance/wac). 
Specifically in the wac-parser crate , `printer.rs` has been changed for specific `wacky` use cases.

## Usage
### 1. First all files listed above must be provided for the program to run.
An example directory layout:

```
wac-main/
├─ crates/
│  ├─ wac-parser.rs     <-----forked&customized for wacky use case
wacky/
├─ src/
│  ├─ lib.rs
│  ├─ main.rs
│  ├─ wac_read.rs
│  ├─ wac_write.rs
│  ├─ wac_write_implicit.rs
├─ Cargo.toml
├─ Cargo.lock
├─ compose.wac
├─ implicitcompose.wac
├─ config.toml
├─ config_implicit.toml
```


### 2. Second the desired state and access control restrictions are set by tuning the shimming parameters in the `config.toml` as follows:
An example of untrusted Component along with the interface needed to shim and the shim component used for the job. 

````toml
["component:file"]                      <---- untrusted component
component_to_shim = "docs:writetwo"     <----  Package name of the component to be shimmed
interface_to_shim = "writer"            <----  Specific interface to shim
package_shim = "docs:writershim"        <----  The Shim Component package name
`````

When an interface is provided wacky can shim that specific interface and its considered the explicit case opposing the implicit one where an interface isnt provided just as shim. A `config_implicit.toml` is provided below to showcase those instances,

````toml
["component:file"]                      <---- untrusted component
package_shim = "docs:writershim"        <----  The Shim Component package name
`````



### 3. Finally build & run the program 
````
#Explicit Interface Example 
cargo run -- --cp 'Config_files/config.toml' --wp 'input_files/compose.wac'

#It will produce output as "shimmed_script.wac", use this to compose the components together:
wac compose -o output.wasm shimmed_script.wac

#Then run the output of the components with runtime of choice. If using Wasmtime:
wasmtime run --dir test-dir output.wasm

`````
All examples were run on Windows.

A successful run will produce a shimmed wac should as follows.
````````
wac-main/    
wacky/
├─ src/
│  ├─ lib.rs
│  ├─ main.rs
│  ├─ wac_read.rs
│  ├─ wac_write.rs
├─ Cargo.toml
├─ Cargo.lock
├─ compose.wac
├─ shimmed_script.wac   <---- Success!
├─ config.toml
````````
The `wacky` output now serves as the new wac script to govern the component composition procedure. 

The remaining steps remain the same.
See `wac` README.md crate for compositions steps after this. [wac-main](./wac-main/README.md)

## Troubleshooting
| Symptom | Cause / fix |
|---|---|
| No output, no log messages | Logging uses `env_logger` at `info` level. Run with `RUST_LOG=info cargo run -- --cp ... --wp ...`. "No untrusted components found" means no `[section]` name in the config matches a `new <package>` in the WAC script. |
| Can't find `shimmed_script.wac` | The output path is hard-coded: it is always written to `shimmed_script.wac` in the **current working directory**, not in `output_files/`. |
| Config entry silently ignored | An entry is **explicit** only if it has *both* `component_to_shim` and `interface_to_shim`. It is **implicit** only if it has *neither*. An entry with just one of the two is dropped. Unknown keys cause a parse error (`deny_unknown_fields`). Section names are lower-cased before matching, so package names in the WAC script must be lower-case. |
| Only one untrusted component got shimmed | Every explicit match re-reads the original `--wp` file and overwrites `shimmed_script.wac`. With several explicit entries matched, only the last one written survives (HashMap order). The same applies to several implicit entries. Run `wacky` once per untrusted component and feed each output back in as `--wp`. |
| Output contains `shim1` / `.writer` with an empty name, or `wac compose` fails | `component_to_shim` does not match any `new <package>` in the script. Check the package name exactly (e.g. `component:trustedwriter`). |
| Shim is instantiated but the untrusted component still uses the original provider | Explicit mode only rewires arguments written as `iface: provider.iface` with a plain identifier name (not quoted `"ns:pkg/iface"` names). Implicit mode only rewires a `...` fill argument, so the untrusted component's `new` must contain `...`. Always inspect the generated WAC. |
| `wac compose` can't find a package | Binaries must be at `deps/<namespace>/<name>.wasm` (e.g. `deps/shim/writershim.wasm` for `shim:writershim`). |
| Shim fails to link / type mismatch | The shim world must **import and export the same interface** it mediates (see `writershim/wit/world.wit`). Generate a correct skeleton with `shimmer -c <file.wit> -i <interface>`. |
| WASI version mismatch / `wit-bindgen` errors | WASI 0.2.x versions differ between components. Declare only the WASI interfaces you actually use, and use the **implicit** mode for WASI shims. |
| Build fails | `wac-parser` is a path dependency on the forked `../wac-main` (custom `printer.rs`), so keep the directory layout. The crate uses Rust edition 2024 (Rust ≥ 1.85). |


## Performance
The performance of `wacky` is measured with Criterion on Wasmtime 36 (Ryzen 7 5800H, 16 GB, Windows 11). 
The baseline is a two-component composition.

| Metric | Baseline | With shim | Overhead |
|---|---|---|---|
| Instantiation (one-time) | 68.39 µs | 122.09 µs | +53.7 µs (+78.5%) |
| 100 calls | 11.32 µs | 11.88 µs | +5.0% (5.6 ns/call) |
| 1,000 calls | 114.49 µs | 118.97 µs | +3.9% (4.5 ns/call) |
| 10,000 calls | 1.14 ms | 1.18 ms | +3.1% (3.5 ns/call) |
| 100,000 calls | 11.35 ms | 11.77 ms | +3.7% (4.2 ns/call) |

## Security considerations
`wacky` rewires an untrusted component's imports to a shim at composition time, without changing the component's code or otherwise affecting its interfaces.  It does not block all external interfaces in the same way as e.g. WASI Virt.  The shim itself must be trusted to incorporate any policies that might be desired.

## License
This project is licensed under the [Apache 2.0 License](LICENSE)

## Funding
This work has been partially supported by the [ELASTIC project](https://elasticproject.eu/), which received funding from the [Smart Networks and Services Joint Undertaking](https://smart-networks.europa.eu/) (SNS JU) under the European Union’s [Horizon Europe](https://research-and-innovation.ec.europa.eu/funding/funding-opportunities/funding-programmes-and-open-calls/horizon-europe_en) research and innovation programme under [Grant Agreement No. 101139067](https://cordis.europa.eu/project/id/101139067). Views and opinions expressed are however those of the author(s) only and do not necessarily reflect those of the European Union. Neither the European Union nor the granting authority can be held responsible for them.





