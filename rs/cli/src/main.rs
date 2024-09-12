use clap::{Parser, Subcommand};
use sails_cli::{js::JsClientGenerator, program::ProgramGenerator};

#[derive(Parser)]
#[command(bin_name = "cargo")]
struct CargoCommands {
    #[command(subcommand)]
    cargo: SailsCommands,
}

#[derive(Subcommand)]
enum SailsCommands {
    #[command(name = "sails", subcommand)]
    Sails(Commands),
    #[command(name = "sails-js", subcommand)]
    SailsJs(JsCommands),
}

#[derive(Subcommand)]
enum JsCommands {
    #[command(name = "generate-client")]
    GenerateClient {
        #[arg(help = "Path to the IDL file")]
        idl: String,
        #[arg(
            short,
            long,
            help = "Path to the output directory",
            default_value = "."
        )]
        out: String,
        #[arg(short, long, help = "Name of the program", default_value = "program")]
        program_name: String,
    },
}

#[derive(Subcommand)]
enum Commands {
    #[command(name = "new-program")]
    NewProgram {
        #[arg(help = "Path to the new program")]
        path: String,
        #[arg(short, long, help = "Name of the new program")]
        name: Option<String>,
        #[arg(
            long,
            help = "Disable generation of client package alongside the program. Implies '--no-gtest'"
        )]
        no_client: bool,
        #[arg(long, help = "Disable generation of program tests using 'gtest'")]
        no_gtest: bool,
    },
}

fn main() -> Result<(), i32> {
    let command: CargoCommands = CargoCommands::parse();

    let result = match command.cargo {
        SailsCommands::Sails(Commands::NewProgram {
            path,
            name,
            no_client,
            no_gtest,
        }) => {
            let program_generator = ProgramGenerator::new(path)
                .with_name(name)
                .with_client(!no_client)
                .with_gtest(!no_gtest);
            program_generator.generate()
        }
        SailsCommands::SailsJs(JsCommands::GenerateClient {
            idl,
            out,
            program_name,
        }) => {
            let generator = JsClientGenerator::new(idl, out, program_name);
            generator.generate()
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {:#}", e);
        return Err(-1);
    }

    Ok(())
}
