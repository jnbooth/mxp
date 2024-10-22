# mxp - MXP Protocol Handler

## Crates

- [mxp](mxp/): Low-level parser for the [MUD eXtension Protocol](https://www.zuggsoft.com/zmud/mxp.htm).
- [mud-transformer](mud-transformer/): Transforms bytes from a Telnet server into rich output. Automatically handles Telnet negotiation and compression.
- [mud-stream](mud-stream/): Wraps around a TCP stream and transforms all input and output through mud-transformer. Supports both sync and async (via Tokio).

## Example Usage

- Synchronous: [mud-bin-examples/src/bin/sync.rs](mud-bin-examples/src/bin/sync.rs)
- Asynchronous: [mud-bin-examples/src/bin/async.rs](mud-bin-examples/src/bin/async.rs)

Full output format: [mud-transformer/src/output/fragment.rs](mud-transformer/src/output/fragment.rs)
