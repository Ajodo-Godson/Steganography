// Helpful papers: 
// - https://thesai.org/Downloads/Volume7No1/Paper_46-Enhanced_Audio_LSB_Steganography_for_Secure_Communication.pdf
// 


mod scripts;

use clap::{Parser, Subcommand};
use scripts::error::StegoError;
use scripts::stego::{embed_message, extract_message};

#[derive(Parser)]
#[command(name = "audio-steg")]
#[command(about = "Hide and recover text in 16-bit PCM WAV files")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Embed {
        input: String,
        output: String,
        message: String,
    },
    Extract {
        input: String,
    },
}

fn main() -> Result<(), StegoError> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Embed {
            input,
            output,
            message,
        } => {
            embed_message(input, output, &message)?;
            println!("Embed successful");
        }
        Commands::Extract { input } => {
            let message = extract_message(input)?;
            println!("{message}");
        }
    }

    Ok(())
}
