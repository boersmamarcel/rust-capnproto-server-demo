# Introduction
In this tutorial, we will learn how to build the foundation of a fast and scalable asynchronous machine learning inference system using Tokio and Cap'n Proto. We will cover the basics of Tokio, including asynchronous I/O, concurrency, and error handling, and how to use Cap'n Proto to serialize and deserialize data. By the end of this tutorial, you will have a solid understanding of how to build asynchronous applications with Tokio and Cap'n Proto.

Note that this tutorial is part of a series of tutorials, and that in the final tutorial of the series we will build the complete system. In this part, we focus only on the asynchronous programming concepts and zero-copy data transfer. Therefore, our application will receive messages from a client and respond with the payload in reverse order.

See the Medium article for more details.

# Run the Application
To run the application, execute the following command in the terminal to start the server:

```bash
cargo run server 127.0.0.1:5000
```

To run the client, execute the following command in the terminal:

```bash
cargo run client 127.0.0.1:5000
```

Here is the output of an example server:

```
cargo run server 127.0.0.1:5000
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.03s
     Running `target/debug/inference server '127.0.0.1:5000'`
     Listening on 127.0.0.1:5000 with 28 worker threads...
     Dispatching new connection to worker 0
     [Worker 0] received a new connection from 127.0.0.1:50332
     [Worker 0] `spawn_local` is running on core [CoreId { id: 0 }]
     [Worker 0] RPC system started for peer: 127.0.0.1:50332
     Received content: Hello this is my first message, what will the result be?
     [Worker 0] RPC system for peer 127.0.0.1:50332 finished cleanly.
```

Here is the output of an example client:

```
cargo run client 127.0.0.1:5000
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.03s
     Running `target/debug/inference client '127.0.0.1:5000'`
Connected to inference server at 127.0.0.1:5000
Enter your message:
Hello this is my first message, what will the result be?
?eb tluser eht lliw tahw ,egassem tsrif ym si siht olleH
```

# References
- [Tokio Documentation](https://tokio.rs/docs/)
- [Cap'n Proto Documentation](https://capnproto.org/)
