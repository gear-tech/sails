use clap::{Parser, Subcommand};
use sails_cli::program::ProgramGenerator;
use sails_client_gen::ClientGenerator;
use std::{path::PathBuf, str::FromStr};

#[derive(Parser)]
#[command(bin_name = "cargo")]
enum CliCommand {
    #[command(name = "sails", subcommand)]
    Sails(SailsCommands),
}

#[derive(Subcommand)]
enum SailsCommands {
    /// Create a new program from template
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

    /// Generate client code from IDL
    #[command(name = "client-rs")]
    ClientRs {
        #[arg(help = "Path to the IDL file", value_parser(PathBuf::from_str))]
        idl_path: PathBuf,
        #[arg(
            help = "Path to the output Rust client file",
            value_parser(PathBuf::from_str)
        )]
        out_path: Option<PathBuf>,
        #[arg(long, help = "Generate module with mocks with specified feature name")]
        mocks: Option<String>,
        #[arg(long, help = "Custom path to the sails-rs crate")]
        sails_crate: Option<String>,
    },
}

fn main() -> Result<(), i32> {
    let CliCommand::Sails(command) = CliCommand::parse();

    let result = match command {
        SailsCommands::NewProgram {
            path,
            name,
            no_client,
            no_gtest,
        } => {
            let program_generator = ProgramGenerator::new(path)
                .with_name(name)
                .with_client(!no_client)
                .with_gtest(!no_gtest);
            program_generator.generate()
        }
        SailsCommands::ClientRs {
            idl_path,
            out_path,
            mocks,
            sails_crate,
        } => {
            let mut client_gen = ClientGenerator::from_idl_path(idl_path.as_ref());
            if let Some(mocks) = mocks.as_ref() {
                client_gen = client_gen.with_mocks(mocks);
            }
            if let Some(sails_crate) = sails_crate.as_ref() {
                client_gen = client_gen.with_sails_crate(sails_crate);
            }
            let out_path = out_path.unwrap_or_else(|| idl_path.with_extension("rs"));
            client_gen.generate_to(out_path)
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {:#}", e);
        return Err(-1);
    }

    Ok(())
}
