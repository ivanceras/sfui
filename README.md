# SFUI
Sauron futuristic User interface
an experimental user interface using sauron to do composable components
and write interactive components in a readable rust code, without resulting to usage of convuloted css code.


## Demo
To run the demo. You need to install a static file server `basic-http-server`
`cargo install basic-http-server`

Go into the demo folder
`cd examples/demo`

Build the demo package
`wasm-pack build --release --target web`

Serve the `index.html` with the compiled parts in `./pkg` using the `basic-http-server`
`basic-http-server ./ -a 0.0.0.0:3337`
