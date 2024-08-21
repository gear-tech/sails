use clap::Parser;
use sails_cli::program::ProgramGenerator;

#[derive(Parser)]
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

fn main() {
    let command = Commands::parse();

    let result = match command {
        Commands::NewProgram {
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
    };

    if let Err(e) = result {
        eprintln!("Error: {:#}", e);
    }
}
