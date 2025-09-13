## Overview

`wacky` is a tool for injecting shims via modifying wac scripts for the purpose of restricting/modifying access control to resources and services provided to the restricted or "untrusted components".
 The tool is built ontop of `wac` which is used for orchestrating and is a main stop point in the composition process of Wasm Component model.
 `wacky` is configurable, giving users control over the composition process and as a result the access control.

 ## Demonstrative example

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



 ## Dependencies
By default the program expects all of the files mentioned below to be placed at project root.
However an option to override the current file paths are provided with the use of `--cp` and `--wp` for  untrusted toml configuration file and an alternative wac path respectively.

The `wacky` tool has the following files:

* `untrusted_component.toml` - Contains the untrusted component along with the shimming parameters. An example file has been placed in root directory.
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
├─ Cargo.toml
├─ Cargo.lock
├─ compose.wac
├─ untrusted_component.wac
```


### 2. Second the desired state and access control restrictions are set by tunning the shimming parameters in the `untrusted_components.toml` as follows:
An example of untrusted Component along with the interface needed to shim and the shim component used for the job. 

````toml
["component:file"]                      <---- untrusted component
component_to_shim = "docs:writetwo"     <----  Package name of the component to be shimmed
interface_to_shim = "writer"            <----  Specific interface to shim
package_shim = "docs:writershim"        <----  The Shim Component package name
`````

### 3. Finally build & run the program 
````
`cargo run`
`````

A successful run will produce a shimmed wac should as follows.
````````
wac-main/
├─ crates/
│  ├─ wac-parser.rs      
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
├─ untrusted_component.toml
````````
The `wacky` output now serves as the new wac script to govern the component composition procedure. 


The remaining steps remain the same.
See `wac` README.md crate for compositions steps after this. [wac-main](./wac-main/README.md)


## License
This project is licensed under the [Apache 2.0 License](LICENSE)





