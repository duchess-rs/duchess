//@run
use duchess::prelude::*;

// No `use duchess::java` here. This file's `java_package!` generates a local
// top-level `java` module, which would conflict with the explicit import. Plain
// `java::io::...` below refers to the local `mod java`.

// Redeclaring JDK classes exercises invocations with their own top-level
// `java` package: `java.io` must resolve to the locally declared `mod java`
// while `java.lang` keeps resolving to duchess's prelude.
duchess::java_package! {
    package java.io;

    public class java.io.OutputStream {
        public static java.io.OutputStream nullOutputStream();
    }

    public class java.io.InputStream {
    }

    public interface java.io.DataInput {
    }

    public class java.io.ByteArrayOutputStream extends java.io.OutputStream {
        public java.io.ByteArrayOutputStream(int);
        public byte[] toByteArray();
    }

    public class java.io.ByteArrayInputStream extends java.io.InputStream {
        public java.io.ByteArrayInputStream(byte[]);
    }

    public class java.io.DataOutputStream implements java.io.DataOutput {
        public java.io.DataOutputStream(java.io.OutputStream);
        public void writeInt(int);
        public void writeByte(int);
        public void writeFloat(float);
        public void writeDouble(double);
    }

    public interface java.io.DataOutput {
    }

    public class java.io.DataInputStream implements java.io.DataInput {
        public java.io.DataInputStream(java.io.InputStream);
        public int readInt();
        public byte readByte();
        public float readFloat();
        public double readDouble();
    }
}

fn main() -> duchess::Result<()> {
    duchess::Jvm::builder().try_launch()?;

    // Big-endian int roundtrip through real java.io streams. Static methods on
    // locally declared classes work too.
    let _null: Java<java::io::OutputStream> = java::io::OutputStream::null_output_stream()
        .assert_not_null()
        .execute()?;

    let baos: Java<java::io::ByteArrayOutputStream> =
        java::io::ByteArrayOutputStream::new(32).execute()?;
    let out: Java<java::io::DataOutputStream> = java::io::DataOutputStream::new(&baos).execute()?;
    out.write_int(0x12345678).execute()?;
    out.write_byte(127).execute()?;
    out.write_float(3.14159f32).execute()?;
    out.write_double(2.718281828459045).execute()?;

    let bytes: Vec<i8> = baos.to_byte_array().assert_not_null().execute()?;
    assert_eq!(
        &bytes.iter().map(|b| *b as u8).collect::<Vec<_>>()[..9],
        &[0x12, 0x34, 0x56, 0x78, 127, 0x40, 0x49, 0x0f, 0xd0]
    );

    let bais: Java<java::io::ByteArrayInputStream> =
        java::io::ByteArrayInputStream::new(bytes.to_java()).execute()?;
    let input: Java<java::io::DataInputStream> = java::io::DataInputStream::new(&bais).execute()?;
    assert_eq!(input.read_int().execute()?, 0x12345678);
    assert_eq!(input.read_byte().execute()?, 127);
    assert!((input.read_float().execute()? - 3.14159).abs() < 1e-6);
    assert!((input.read_double().execute()? - 2.718281828459045).abs() < 1e-15);

    Ok(())
}
