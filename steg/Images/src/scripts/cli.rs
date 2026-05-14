use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "steg")]
#[command(about = "DCT steganography + encryption CLI", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Run transform/crypto/stego demo
    Demo,

    /// Hide plaintext inside an image
    Embed {
        /// Input cover image
        #[arg(short = 'i', long)]
        input: String,

        /// Output stego image (use .png)
        #[arg(short = 'o', long)]
        output: String,

        /// Encryption password
        #[arg(short = 'p', long)]
        password: String,

        /// Message to hide
        #[arg(short = 'm', long)]
        message: String,
    },

    /// Extract plaintext from a stego image
    Extract {
        /// Input stego image
        #[arg(short = 'i', long)]
        input: String,

        /// Encryption password
        #[arg(short = 'p', long)]
        password: String,
    },

    /// Hide a file (e.g. butterfly.png) inside an image
    EmbedFile {
        /// Input cover image
        #[arg(short = 'i', long)]
        input: String,

        /// Output stego image (use .png)
        #[arg(short = 'o', long)]
        output: String,

        /// Encryption password
        #[arg(short = 'p', long)]
        password: String,

        /// File to hide
        #[arg(short = 'f', long)]
        secret_file: String,
    },

    /// Extract hidden file from a stego image
    ExtractFile {
        /// Input stego image
        #[arg(short = 'i', long)]
        input: String,

        /// Encryption password
        #[arg(short = 'p', long)]
        password: String,

        /// Output file path (optional). If omitted, embedded filename is used.
        #[arg(short = 'o', long)]
        output: Option<String>,
    },
}
