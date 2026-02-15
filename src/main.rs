use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;
use layer_pack::format::{PackManifest, PackType};
#[cfg(feature = "builder")]
use layer_pack::builder::PackBuilder;
use layer_pack::resolver::{Resolver, LoadedPack};

#[derive(Parser)]
#[command(name = "lpack")]
#[command(about = "Layered Asset Pack Tool", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new pack from a directory
    #[cfg(feature = "builder")]
    Create {
        /// Source directory
        source: PathBuf,
        /// Output file (.pack)
        output: PathBuf,
        /// Pack Name
        #[arg(long)]
        name: String,
        /// Pack Type
        #[arg(long, value_enum)]
        type_: PackTypeArg,
        /// Language (e.g., "en", "th")
        #[arg(long)]
        lang: Option<String>,
        /// Priority (higher wins)
        #[arg(long, default_value_t = 0)]
        /// Priority (higher wins)
        #[arg(long, default_value_t = 0)]
        priority: i32,
        /// Custom Reference/Hash
        #[arg(long)]
        r#ref: Option<String>,
        /// Author Name
        #[arg(long)]
        author: Option<String>,
        /// Website URL
        #[arg(long)]
        website: Option<String>,
    },
    /// List files in a pack
    List {
        /// Pack file
        pack: PathBuf,
    },
    /// Resolve a file path across multiple packs
    Resolve {
        /// Directory containing packs
        #[arg(short, long)]
        dir: Option<PathBuf>,
        /// Specific pack files
        #[arg(short, long)]
        packs: Vec<PathBuf>,
        /// Virtual path to resolve
        path: String,
    },
    /// Unpack files from a pack
    Unpack {
        /// Pack file
        pack: PathBuf,
        /// Output directory
        output: PathBuf,
    },
}

#[derive(Clone, ValueEnum)]
enum PackTypeArg {
    Base,
    Text,
    Image,
    Audio,
    Script,
    Mod,
}

impl From<PackTypeArg> for PackType {
    fn from(arg: PackTypeArg) -> Self {
        match arg {
            PackTypeArg::Base => PackType::Base,
            PackTypeArg::Text => PackType::Text,
            PackTypeArg::Image => PackType::Image,
            PackTypeArg::Audio => PackType::Audio,
            PackTypeArg::Script => PackType::Script,
            PackTypeArg::Mod => PackType::Mod,
        }
    }
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        #[cfg(feature = "builder")]
        Commands::Create { source, output, name, type_, lang, priority, r#ref, author, website } => {
            let manifest = PackManifest {
                name,
                pack_type: type_.into(),
                lang,
                priority,
                description: None,
                version: Some("1.0".to_string()),
                custom_ref: r#ref,
                author,
                website,
            };
            let mut output_path = output;
            if let Some(ext) = output_path.extension() {
                if ext != "lpack" {
                     output_path.set_extension("lpack");
                }
            } else {
                output_path.set_extension("lpack");
            }

            let builder = PackBuilder::new(manifest);
            builder.build(source, output_path)?;
            println!("Pack created successfully.");
        }
        #[cfg(not(feature = "builder"))]
        Commands::Create { .. } => {
            anyhow::bail!("The 'builder' feature is not enabled in this build.");
        }
        Commands::List { pack } => {
            let loaded = LoadedPack::load(pack)?;
            println!("Pack: {} (Type: {:?}, Lang: {:?}, Priority: {})", 
                loaded.manifest.name, 
                loaded.manifest.pack_type, 
                loaded.manifest.lang,
                loaded.manifest.priority
            );
            println!("MIME Type: {}", layer_pack::format::CONTENT_TYPE);
            if let Some(r) = &loaded.manifest.custom_ref {
                println!("Ref: {}", r);
            }
            if let Some(a) = &loaded.manifest.author {
                println!("Author: {}", a);
            }
            if let Some(w) = &loaded.manifest.website {
                println!("Website: {}", w);
            }
            println!("{:<50} | {:<10} | {:<10} | {:<10}", "Path", "Size", "CmpSize", "Method");
            println!("{:-<90}", "");
            
            let mut files = loaded.file_list();
            files.sort();
            
            for path in files {
                if let Some(entry) = loaded.get_entry(&path) {
                     println!("{:<50} | {:<10} | {:<10} | {:?}", 
                        path, 
                        entry.original_size, 
                        entry.compressed_size, 
                        entry.compression
                    );
                }
            }
        }
        Commands::Resolve { dir, packs, path } => {
            let mut resolver = Resolver::new();
            
            if let Some(d) = dir {
                if d.exists() && d.is_dir() {
                    for entry in std::fs::read_dir(d)? {
                        let entry = entry?;
                        let p = entry.path();
                        if p.extension().map_or(false, |e| e == "pack") {
                             match LoadedPack::load(&p) {
                                Ok(pack) => resolver.add_pack(pack),
                                Err(e) => eprintln!("Failed to load {:?}: {}", p, e),
                             }
                        }
                    }
                }
            }
            for p in packs {
                 match LoadedPack::load(&p) {
                    Ok(pack) => resolver.add_pack(pack),
                    Err(e) => eprintln!("Failed to load {:?}: {}", p, e),
                 }
            }

            let layers = resolver.list_layers(&path);
            if layers.is_empty() {
                println!("File '{}' not found in any pack.", path);
            } else {
                println!("File '{}' found in:", path);
                for layer in layers {
                    println!(" - {}", layer);
                }
                
                if let Some(data) = resolver.resolve(&path) {
                    println!("Resolved content size: {} bytes", data.len());
                    // Try to print as string if it looks like text
                    if let Ok(text) = String::from_utf8(data) {
                        println!("Content preview:\n---");
                        println!("{}", text.lines().take(10).collect::<Vec<_>>().join("\n"));
                        println!("---");
                    } else {
                        println!("(Binary content)");
                    }
                }
            }
        }
        Commands::Unpack { pack, output } => {
            let mut loaded = LoadedPack::load(&pack)?;
            println!("Unpacking {} to {:?}...", pack.display(), output);

            let files = loaded.file_list();
            for path in files {
                let content = loaded.read_file(&path)?;
                let output_path = output.join(&path);
                
                if let Some(parent) = output_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                
                std::fs::write(&output_path, content)?;
                println!("Extracted: {}", path);
            }
            println!("Unpack complete.");
        }
    }

    Ok(())
}