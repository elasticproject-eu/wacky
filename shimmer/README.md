## Overview

`shimmer` is a tool for creating shim scaffolds.


 ## Demonstrative examples

`addershim` is an example of the output of `shimmer`, created by passing the wit `componenta.wit` with interface `add` to be shimmed. 



 ## Dependencies

The program expects the file mentioned below to be provided with the use of `-c`and the interface to be shimmed `-i`
The `shimmer` tool input file:

* `Wit File` - A wit file of the component to be shimmed.


## Usage

To run shimmer

`````
cargo run -- -c componenta.wit -i add

````````
where componenta.wit = wit file to be shimmed
add = the particular interface to be shimmed
## License
This project is licensed under the [Apache 2.0 License](LICENSE)





