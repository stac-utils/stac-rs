use crate::Result;
use stac_server::PgstacBackend;

#[derive(Debug, Clone, clap::Args)]
pub struct Args {
    /// The pgstac subcommand
    #[command(subcommand)]
    subcommand: Subcommand,
}

#[derive(clap::Subcommand, Debug, Clone)]
pub enum Subcommand {
    /// Loads data into the pgstac database
    Load(LoadArgs),
}

#[derive(clap::Args, Debug, Clone)]
pub struct LoadArgs {
    /// The connection string.
    dsn: String,

    /// Hrefs to load into the database.
    ///
    /// If not provided or `-`, data will be read from standard input.
    hrefs: Vec<String>,

    /// Load in all "item" links on collections.
    #[arg(short, long)]
    load_collection_items: bool,

    /// Auto-generate collections for any collection-less items.
    #[arg(short, long)]
    create_collections: bool,
}

impl crate::Args {
    pub async fn pgstac(&self, args: &Args) -> Result<()> {
        match &args.subcommand {
            Subcommand::Load(load_args) => {
                let mut backend = PgstacBackend::new_from_stringlike(&load_args.dsn).await?;
                let load = self
                    .load(
                        &mut backend,
                        load_args.hrefs.iter().map(|h| h.as_str()),
                        load_args.load_collection_items,
                        load_args.create_collections,
                    )
                    .await?;
                eprintln!(
                    "Loaded {} collection(s) and {} item(s)",
                    load.collections, load.items
                );
                Ok(())
            }
        }
    }
}
