use clap::{Parser, Subcommand};
use sails_cli::{idlgen::CrateIdlGenerator, program::ProgramGenerator, solgen::SolidityGenerator};
use sails_client_gen::ClientGenerator;
use std::{error::Error, path::PathBuf};

#[derive(Parser)]
#[command(bin_name = "cargo")]
enum CliCommand {
    #[command(name = "sails", subcommand)]
    Sails(SailsCommands),
}

#[derive(Subcommand)]
enum SailsCommands {
    /// Create a new program from template
    #[command(name = "program")]
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
        #[arg(long, help = "Use 'sails-rs' crate of the specified version")]
        sails_version: Option<String>,
    },

    /// Generate client code from IDL
    #[command(name = "client-rs")]
    ClientRs {
        /// Path to the IDL file
        #[arg(value_hint = clap::ValueHint::FilePath)]
        idl_path: PathBuf,
        /// Path to the output Rust client file
        #[arg(value_hint = clap::ValueHint::FilePath)]
        out_path: Option<PathBuf>,
        /// Generate module with mocks with specified feature name
        #[arg(long)]
        mocks: Option<String>,
        /// Custom path to the sails-rs crate
        #[arg(long)]
        sails_crate: Option<String>,
        /// Map type from IDL to crate path, separated by `=`, example `-T Part=crate::parts::Part`
        #[arg(long, short = 'T', value_parser = parse_key_val::<String, String>)]
        external_types: Vec<(String, String)>,
        /// Derive only nessessary [`parity_scale_codec::Encode`], [`parity_scale_codec::Decode`] and [`scale_info::TypeInfo`] traits for the generated types
        #[arg(long)]
        no_derive_traits: bool,
    },

    /// Generate IDL from Cargo manifest
    #[command(name = "idl")]
    IdlGen {
        /// Path to the crate with program
        #[arg(long, value_hint = clap::ValueHint::FilePath)]
        manifest_path: Option<PathBuf>,
        /// Directory for all generated artifacts
        #[arg(long, value_hint = clap::ValueHint::DirPath)]
        target_dir: Option<PathBuf>,
        /// Level of dependencies to look for program implementation. Default: 1
        #[arg(long)]
        deps_level: Option<usize>,
    },

    #[command(name = "sol")]
    SolGen {
        /// Path to the IDL file
        #[arg(long, value_hint = clap::ValueHint::FilePath)]
        idl_path: PathBuf,
        /// Directory for all generated artifacts
        #[arg(long, value_hint = clap::ValueHint::DirPath)]
        target_dir: Option<PathBuf>,
        /// Name of the contract to generate
        #[arg(long, short = 'n')]
        contract_name: Option<String>,
    },
}

/// Parse a single key-value pair
fn parse_key_val<T, U>(s: &str) -> Result<(T, U), Box<dyn Error + Send + Sync + 'static>>
where
    T: std::str::FromStr,
    T::Err: Error + Send + Sync + 'static,
    U: std::str::FromStr,
    U::Err: Error + Send + Sync + 'static,
{
    let pos = s
        .find('=')
        .ok_or_else(|| format!("invalid KEY=value: no `=` found in `{s}`"))?;
    Ok((s[..pos].parse()?, s[pos + 1..].parse()?))
}

fn main() -> Result<(), i32> {
    let CliCommand::Sails(command) = CliCommand::parse();

    let result = match command {
        SailsCommands::NewProgram {
            path,
            name,
            no_client,
            no_gtest,
            sails_version,
        } => {
            let program_generator = ProgramGenerator::new(path)
                .with_name(name)
                .with_client(!no_client)
                .with_gtest(!no_gtest)
                .with_sails_version(sails_version);
            program_generator.generate()
        }
        SailsCommands::ClientRs {
            idl_path,
            out_path,
            mocks,
            sails_crate,
            external_types,
            no_derive_traits,
        } => {
            let mut client_gen = ClientGenerator::from_idl_path(idl_path.as_ref());
            if let Some(mocks) = mocks.as_ref() {
                client_gen = client_gen.with_mocks(mocks);
            }
            if let Some(sails_crate) = sails_crate.as_ref() {
                client_gen = client_gen.with_sails_crate(sails_crate);
            }
            for (name, path) in external_types.iter() {
                client_gen = client_gen.with_external_type(name, path);
            }
            if no_derive_traits {
                client_gen = client_gen.with_no_derive_traits();
            }
            let out_path = out_path.unwrap_or_else(|| idl_path.with_extension("rs"));
            client_gen.generate_to(out_path)
        }
        SailsCommands::IdlGen {
            manifest_path,
            target_dir,
            deps_level,
        } => CrateIdlGenerator::new(manifest_path, target_dir, deps_level).generate(),
        SailsCommands::SolGen {
            idl_path,
            target_dir,
            contract_name,
        } => SolidityGenerator::new(idl_path, target_dir, contract_name).generate(),
    };

    if let Err(e) = result {
        eprintln!("Error: {e:#}");
        return Err(-1);
    }

    Ok(())
}
