# minactor

![Test Status](https://gist.githubusercontent.com/Danconnolly/a56c9e54e657e2f0daefa22f58a3df63/raw/badge.svg)
[![dependency status](https://deps.rs/repo/github/Danconnolly/minactor/status.svg)](https://deps.rs/repo/github/Danconnolly/minactor)

minactor is a minimal [actor](https://en.wikipedia.org/wiki/Actor_model) framework for [tokio](https://tokio.rs/).
The framework was inspired by  [Erlang](https://en.wikipedia.org/wiki/Erlang_(programming_language)) but adapted
to Rust paradigms. Other inspirations came from [Alice Ryhl](https://ryhl.io/blog/actors-with-tokio/), and 
[ractor](https://github.com/slawlor/ractor).

Actors created in minactor have a tiny overhead and messages are passed using tokio channels. Each instance of an actor
has a single thread of control (a tokio async task). Creating actors is simple. 

It is designed for single system implementations, not clusters of systems. If you need clusters, you're probably better
served by other frameworks such as [ractor](https://github.com/slawlor/ractor).
