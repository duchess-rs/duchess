use core::fmt;
use std::io::Write;

pub struct CodeWriter<'w> {
    writer: &'w mut dyn Write,
    indent: usize,
}

impl<'w> CodeWriter<'w> {
    pub fn new(writer: &'w mut dyn Write) -> Self {
        CodeWriter { writer, indent: 0 }
    }

    pub fn write_fmt(&mut self, fmt: std::fmt::Arguments<'_>) -> anyhow::Result<()> {
        let mut string = String::new();
        fmt::write(&mut string, fmt).unwrap();

        if string.starts_with("}") || string.starts_with(")") || string.starts_with("]") {
            self.indent -= 1;
        }

        write!(
            self.writer,
            "{:indent$}{}\n",
            "",
            string,
            indent = self.indent * 4
        )?;

        if string.ends_with("{") || string.ends_with("(") || string.ends_with("[") {
            self.indent += 1;
        }

        Ok(())
    }
}
