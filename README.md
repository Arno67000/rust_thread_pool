# THREAD POOL RUST LIB

This is a small lib to generate a pool containing as many threads as you need and send functions in the pool to run them in parallel.

The pool uses std::thread to run the functions and mpsc channel to communicate.

To use FnMut you'd have to implement the mutating variables into Arc to move references through te threads.

Please read the tests if you need an example of how to use the pool.
