## Overview

`wacky` is a tool for injecting shims via modifying wac scripts for the purpose of restricting/modifying access control to resources and services provided to the restricted or "untrusted components".
 The tool is built ontop of `wac` which is used for orchestrating and is a main stop point in the composition process of Wasm Component model.
 `wacky` is configurable, giving users control over the composition process and as a result the access control.

 ## Demonstrative examples

 The following is an example of a wac file used to compose two components together, 

Before running `wacky`
```
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

```
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

 ```
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
```
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

```
package example:composition;


let untrustedcomponent = new component:fileloader {...};

export untrustedcomponent.run;

```
After running wacky implicit case:
```
package example:composition;

let untrustedcomponentimplicitshim1 = new shim:writershim { ... };

let untrustedcomponent = new component:fileloader {
    ...untrustedcomponentimplicitshim1,
};

export untrustedcomponent.run;

```

Fourth combined example:
Before running wasi
````
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
`````
After running wacky on combined:
````
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
````


A full example is already placed out when running wacky on compose.wac. 
All components used in compose.wac and output have their binaries placed in deps according to wac requirments and their source code in directory "Components Source Code". 

Samples of input, output and config files have been placed in respective directories.


 ## Dependencies
By default the program expects all of the files mentioned below to be placed at project root.
However an option to override the current file paths are provided with the use of `--cp` and `--wp` for  untrusted toml configuration file and an alternative wac path respectively.

The `wacky` tool has the following files:

* `config.toml` - Contains the untrusted component along with the shimming parameters. An example file has been placed in root directory.
* `compose.wac` - Original wac script file that we wish to investigate for untrusted components and shim if found.
* `shimed_script.wac` - Output of `wacky` , produced after running the program. It contains the updated component links needed to inject the shim.

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


### 2. Second the desired state and access control restrictions are set by tunning the shimming parameters in the `config.toml` as follows:
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
cargo run -- --cp 'Sample of configuration files/config.toml' --wp 'Sample of input/compose.wac'

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


## License
This project is licensed under the [Apache 2.0 License](LICENSE)





