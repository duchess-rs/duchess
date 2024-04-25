# Duchess: silky smooth Java integration

[<img src="https://img.shields.io/badge/chat-on%20Zulip-green"></img>][Zulip]
[<img src="https://img.shields.io/badge/Coverage-green"></img>][Coverage]

Duchess is a Rust crate that makes it easy, ergonomic, and efficient to interoperate with Java code.

<img src="book/src/duchess.svg" width="300"></img>


## TL;DR

Duchess permits you to reflect Java classes into Rust and easily invoke methods on Java objects. For example the following Java code...

```rust
Logger logger = new log.Logger();
logger.addEvent(
    Event.builder()
        .withTime(new Date())
        .withName("foo")
        .build()
);
```

...could be executed in Rust as follows:

```rust
let logger = log::Logger::new().global().execute()?;
logger
    .add_event(
        log::Event::builder()
            .with_time(java::util::Date::new())
            .with_name("foo")
            .build(),
    )
    .execute()?;
```

## Curious to learn more?

Check out the...

* The [examples](https://github.com/duchess-rs/duchess/tree/main/test-crates/duchess-java-tests/tests/ui/examples)
* The [tutorials](https://duchess-rs.github.io/duchess/tutorials.html) chapter

## Curious to get involved?

Look for [issues tagged with good first issue][] and join the [Zulip][]. For more information on how to develop duchess, 
see [Contributing][]. You may also be able to improve test [coverage].

[issues tagged with good first issue]: https://github.com/duchess-rs/duchess/issues?q=is%3Aopen+is%3Aissue+label%3A%22good+first+issue%22
[Zulip]: https://duchess.zulipchat.com/
[Contributing]: CONTRIBUTING.md
[Coverage]: https://duchess-rs.github.io/duchess/coverage
