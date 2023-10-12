use duchess::{java, prelude::*, Global};

#[test]
fn to_java_and_back() {
    for example in [
        //basic cases
        "",
        "abc\tdef",
        "hello from 🦀!",
        // various lengths
        "a".repeat(1).as_str(),
        "a".repeat(2).as_str(),
        "a".repeat(3).as_str(),
        "a".repeat(4).as_str(),
        "a".repeat(63).as_str(),
        "a".repeat(64).as_str(),
        "a".repeat(65).as_str(),
        "a".repeat(127).as_str(),
        "a".repeat(128).as_str(),
        "a".repeat(129).as_str(),
        "a".repeat(1024 * 1024 - 1).as_str(),
        "a".repeat(1024 * 1024).as_str(),
        "a".repeat(1024 * 1024 + 1).as_str(),
        // unicode code points of various UTF-8 lengths, some requiring surrogate pairs in Java
        "$", // 1
        "£", // 2
        "€", // 3
        "𐍈", // 4
        // combinations of various codepoint lengths
        "$£€𐍈€£$𐍈",
        // nul byte
        "\u{0000}",
    ] {
        let java: Global<java::lang::String> = example
            .to_java()
            .assert_not_null()
            .global()
            .execute()
            .unwrap();
        let and_back = (&*java).to_rust().execute().unwrap();
        assert_eq!(example, and_back);
    }
}
