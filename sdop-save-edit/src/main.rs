use std::{
    io::{Read, Write},
    path::PathBuf,
    str::FromStr,
};

use clap::{Parser, Subcommand};
use sdop_game::{ALL_ITEMS, SaveFile};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[clap(subcommand)]
    subcommand: AppSubCommand,
}

#[derive(Debug, Subcommand, Clone)]
enum AppSubCommand {
    Decode {
        #[arg(short, long)]
        source: PathBuf,
        #[arg(short, long)]
        unlock_all: bool,
    },
    Encode {
        #[arg(short, long)]
        source: PathBuf,
    },
}

fn main() {
    let args = Args::parse();

    match args.subcommand {
        AppSubCommand::Decode { source, unlock_all } => {
            println!("Loading {:?}", source.as_os_str());

            if !source.exists() {
                eprintln!("Must be a valid path");
                std::process::exit(1);
            }

            let mut save = {
                let mut file = std::fs::File::open(source).unwrap();
                let mut bytes = vec![];
                file.read_to_end(&mut bytes).unwrap();
                match SaveFile::from_bytes(&bytes) {
                    Ok(save) => save,
                    Err(err) => {
                        eprintln!("Error decoding save file {}", err);
                        std::process::exit(1);
                    }
                }
            };

            if unlock_all {
                for item in ALL_ITEMS {
                    save.inventory.add_item(item, 1);
                }
            }

            println!("Loaded save");

            let ron_str =
                ron::ser::to_string_pretty(&save, ron::ser::PrettyConfig::default()).unwrap();

            let target = PathBuf::from_str("sdop-sav.ron").unwrap();

            let mut file = std::fs::File::create(target.clone()).unwrap();
            file.write_all(ron_str.as_bytes()).unwrap();

            println!("Wrote decoded save to {:?}", target);
        }
        AppSubCommand::Encode { source } => {
            if !source.exists() {
                eprintln!("Must be a valid path");
                std::process::exit(1);
            }

            let save: SaveFile = {
                let contents = std::fs::read_to_string(source).unwrap();
                match ron::from_str(&contents) {
                    Ok(save) => save,
                    Err(err) => {
                        eprintln!("Error decoding ron {}", err);
                        std::process::exit(1);
                    }
                }
            };

            println!("Loaded save");

            let bytes = save.to_bytes().unwrap();

            let target = PathBuf::from_str("sdop.sav").unwrap();

            let mut file = std::fs::File::create(target.clone()).unwrap();
            file.write_all(&bytes).unwrap();

            println!("Wrote encoded save to {:?}", target);
        }
    }
}
