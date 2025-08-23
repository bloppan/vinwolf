use clap::{Parser, Subcommand};
use rocksdb::{DB, IteratorMode};

#[derive(Parser)]
#[command(name = "cli")]
#[command(about = "CLI RocksDB")]
struct Cli {
    #[arg(long, default_value = "_kvdb")]
    db_path: String,
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    Scan,
}

fn main() -> Result<(), rocksdb::Error> {
    let cli = Cli::parse();
    let db = DB::open_default(cli.db_path)?;
    match cli.cmd {
        Cmd::Scan => {
            for item in db.iterator(IteratorMode::Start) {
                let (k, v) = item?;
                println!("{:?}\t{:?}", k, v);
            }
        }
    }
    Ok(())
}
