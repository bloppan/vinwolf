#[cfg(feature = "DB")]
use std::env;
#[cfg(feature = "DB")]
use utils::hex;
#[cfg(feature = "DB")]
use rocksdb::IteratorMode;
#[cfg(feature = "DB")]
use storage::{ancestors::{RocksDBWrapper}, Storage};  

fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "DB")]
    {
        let args: Vec<String> = env::args().collect();

        if args.len() < 2 {
            print_usage();
            return Ok(());
        }
        
        let command = &args[1];
        let mut path = String::from("./ancestors_db");  // Default path

        // Parse --path flag (simple: check if second arg is --path)
        let mut arg_idx = 2;
        if args.len() > arg_idx + 1 && args[arg_idx] == "--path" {
            path = args[arg_idx + 1].clone();
            arg_idx += 2;
        }

        let mut wrapper = RocksDBWrapper::new(&path);  

        match command.as_str() {
            "list" => {
                let cf = wrapper.cf_handle();
                let mut iter = wrapper.db.iterator_cf(cf, IteratorMode::Start);
                println!("Entries in DB:");
                while let Some(Ok((key, value))) = iter.next() {
                    // Fix: Manually copy first 4 bytes from key (Box<[u8]>) to [u8; 4]
                    let mut slot_bytes = [0u8; 4];
                    if key.len() >= 4 {
                        slot_bytes.copy_from_slice(&key[0..4]);
                    }
                    let slot = u32::from_be_bytes(slot_bytes);
                    let hash_hex = hex::encode(&value);
                    println!("  Slot {} -> Hash {}", slot, hash_hex);
                }
            }
            "get" => {
                if args.len() < arg_idx + 1 {
                    eprintln!("Error: 'get' requires a slot (u32)");
                    return Ok(());
                }
                let slot: u32 = args[arg_idx].parse().map_err(|_| "Invalid slot")?;
                if let Some(hash) = <RocksDBWrapper as Storage>::get(&wrapper, &slot) {
                    let hash_hex = hex::encode(&hash);  // Asume tu función manual
                    println!("Slot {} -> Hash {}", slot, hash_hex);
                } else {
                    println!("Slot {} not found", slot);
                }
            }
            "insert" => {
                if args.len() < 4 {
                    eprintln!("Error: 'insert' requires slot (u32) and hash_hex (64 chars)");
                    return Ok(());
                }
                let slot: u32 = args[2].parse().map_err(|_| "Invalid slot")?;
                let hash_hex = &args[3];
                let hash_bytes = hex::decode(hash_hex).map_err(|_| "Invalid hex (must be 64 chars, 32 bytes)")?;
                let hash = hash_bytes.try_into().map_err(|_| "Invalid hash length (must be 32 bytes)")?;
                wrapper.insert(slot, hash);
                println!("Inserted slot {} with hash {}", slot, hash_hex);
            }
            "delete" => {
                if args.len() < 3 {
                    eprintln!("Error: 'delete' requires a slot (u32)");
                    return Ok(());
                }
                let slot: u32 = args[2].parse().map_err(|_| "Invalid slot")?;
                wrapper.remove(&slot);
                println!("Deleted slot {}", slot);
            }
            _ => {
                eprintln!("Unknown command: {}", command);
                print_usage();
            }
        }
    }

    #[cfg(not(feature = "DB"))]
    {
        eprintln!("Error: DB feature not enabled. Compile with --features DB");
        std::process::exit(1);
    }

    #[cfg(feature = "DB")]
    Ok(())
}

#[cfg(feature = "DB")]
fn print_usage() {
    println!("Usage: storage_debug <command> [options]");
    println!("Commands:");
    println!("  list                    # List all entries");
    println!("  get <slot>              # Get hash for slot (u32)");
    println!("  insert <slot> <hash_hex> # Insert slot (u32) and hex hash (64 chars)");
    println!("  delete <slot>           # Delete slot (u32)");
    println!("Options:");
    println!("  --path <dir>            # DB path (default: ./ancestors_db)");
    println!("\nExample: storage_debug insert 1000 000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f");
}