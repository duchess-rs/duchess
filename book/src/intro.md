# Duchess: silky smooth Java integration

[<img src="https://img.shields.io/badge/chat-on%20Zulip-green"></img>][Zulip]

Duchess is a Rust crate that makes it [safe](./safety.md), ergonomic, and efficient to interoperate with Java code.

<img src="duchess.svg" width="300"></img>

<a href="./coverage">Coverage Report</a>


## TL;DR

Duchess permits you to reflect Java classes into Rust and easily invoke methods on Java objects. For example the following Java code...

```rust,ignore
Logger logger = new log.Logger();
logger.addEvent(
    Event.builder()
        .withTime(new Date())
        .withName("foo")
        .build()
);
```

...could be executed in Rust as follows:

```rust,ignore
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

* [Rustdoc](./rustdoc/doc/duchess/index.html)
* The [examples](https://github.com/duchess-rs/duchess/tree/main/test-crates/duchess-java-tests/tests/ui/examples)
* The [tutorials](https://duchess-rs.github.io/duchess/tutorials.html) chapter

## Curious to get involved?

Look for [issues tagged with good first issue][] and join the [Zulip][].

[issues tagged with good first issue]: https://github.com/duchess-rs/duchess/issues?q=is%3Aopen+is%3Aissue+label%3A%22good+first+issue%22
[Zulip]: https://duchess.zulipchat.com/
