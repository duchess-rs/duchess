use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "cargo-duchess")]
enum Opt {
    /// Initialize something
    Init {
        #[structopt(flatten)]
        options: init::InitOptions,
    },
    /// Package something
    Package {
        /// Path to the package
        #[structopt(short, long)]
        path: String,
    },
}

mod init;

fn main() -> anyhow::Result<()> {
    let opt = Opt::from_args();
    match opt {
        Opt::Init { options } => {
            init::init(options)?;
        }
        Opt::Package { path: _ } => todo!(),
    }
    Ok(())
}
