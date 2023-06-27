# THREAD POOL RUST LIB

This is a small lib to generate a pool containing as many threads as you need and send functions in the pool to run them in parallel.

The pool uses std::thread to run the functions and mpsc channel to communicate.

Please read the tests for examples of how to use the pool.
