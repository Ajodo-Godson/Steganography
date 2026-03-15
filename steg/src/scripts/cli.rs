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
    /// Hide plaintext inside an image
    Embed {
        /// Input cover image path
        #[arg(short, long)]
        input: String,

        /// Output stego image path
        #[arg(short, long)]
        output: String,

        /// Password for encryption
        #[arg(short, long)]
        password: String,

        /// Plaintext to embed
        #[arg(short, long)]
        message: String,
    },

    /// Extract and decrypt plaintext from a stego image
    Extract {
        /// Input stego image path
        #[arg(short, long)]
        input: String,

        /// Password for decryption
        #[arg(short, long)]
        password: String,
    },

    /// Run transform/crypto self-check demos
    Demo,
}