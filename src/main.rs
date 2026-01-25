use clap::{Parser, Subcommand, ValueEnum};
use gho::error::AppError;
use gho::keychain;
use gho::models::{AccountKind, Protocol};
use gho::storage::FilesystemStorage;
use gho::{account, pr, repo};

#[derive(Parser)]
#[command(name = "gho")]
#[command(version)]
#[command(about = "GitHub operator CLI for multi-account workflows", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Manage GitHub accounts
    #[clap(visible_alias = "a")]
    Account {
        #[command(subcommand)]
        command: AccountCommands,
    },
    /// Manage repositories
    #[clap(visible_alias = "r")]
    Repo {
        #[command(subcommand)]
        command: RepoCommands,
    },
    /// Manage pull requests
    #[clap(visible_alias = "p")]
    Pr {
        #[command(subcommand)]
        command: PrCommands,
    },
}

#[derive(Subcommand)]
enum AccountCommands {
    /// Add a new account
    Add {
        /// Account identifier
        id: String,
        /// GitHub username
        #[clap(short, long)]
        username: String,
        /// Account kind
        #[clap(short, long, value_enum, default_value = "personal")]
        kind: AccountKindArg,
        /// GitHub personal access token
        #[clap(short, long)]
        token: String,
        /// Default organization
        #[clap(short = 'o', long)]
        default_org: Option<String>,
        /// Clone protocol
        #[clap(short, long, value_enum, default_value = "ssh")]
        protocol: ProtocolArg,
        /// Default clone directory
        #[clap(short = 'd', long)]
        clone_dir: Option<String>,
    },
    /// List all accounts
    #[clap(visible_alias = "ls")]
    List,
    /// Switch active account
    #[clap(visible_alias = "u")]
    Use {
        /// Account ID to switch to (interactive if omitted)
        id: Option<String>,
    },
    /// Show active account details
    Show,
    /// Remove an account
    #[clap(visible_alias = "rm")]
    Remove {
        /// Account ID to remove
        id: String,
    },
}

#[derive(Subcommand)]
enum RepoCommands {
    /// List repositories
    #[clap(visible_alias = "ls")]
    List {
        /// Organization to list repos from
        #[clap(short, long)]
        org: Option<String>,
        /// Maximum number of repositories
        #[clap(short, long, default_value = "30")]
        limit: usize,
        /// Output as JSON
        #[clap(long)]
        json: bool,
    },
    /// Clone a repository
    #[clap(visible_alias = "cl")]
    Clone {
        /// Repository to clone (owner/repo)
        repo: Option<String>,
        /// Organization to bulk clone from
        #[clap(long)]
        org: Option<String>,
        /// Maximum repos to clone (for bulk)
        #[clap(short, long, default_value = "10")]
        limit: usize,
    },
}

#[derive(Subcommand)]
enum PrCommands {
    /// List open pull requests
    #[clap(visible_alias = "ls")]
    List {
        /// Repository (owner/repo), detected from git if omitted
        repo: Option<String>,
        /// Maximum number of PRs
        #[clap(short, long, default_value = "30")]
        limit: usize,
    },
}

#[derive(Clone, ValueEnum)]
enum AccountKindArg {
    Personal,
    Work,
}

impl From<AccountKindArg> for AccountKind {
    fn from(arg: AccountKindArg) -> Self {
        match arg {
            AccountKindArg::Personal => AccountKind::Personal,
            AccountKindArg::Work => AccountKind::Work,
        }
    }
}

#[derive(Clone, ValueEnum)]
enum ProtocolArg {
    Ssh,
    Https,
}

impl From<ProtocolArg> for Protocol {
    fn from(arg: ProtocolArg) -> Self {
        match arg {
            ProtocolArg::Ssh => Protocol::Ssh,
            ProtocolArg::Https => Protocol::Https,
        }
    }
}

fn main() {
    let cli = Cli::parse();

    let result: Result<(), AppError> = run(cli);

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> Result<(), AppError> {
    let storage = FilesystemStorage::new_default()?;

    match cli.command {
        Commands::Account { command } => run_account_command(&storage, command),
        Commands::Repo { command } => run_repo_command(&storage, command),
        Commands::Pr { command } => run_pr_command(&storage, command),
    }
}

fn run_account_command(
    storage: &FilesystemStorage,
    command: AccountCommands,
) -> Result<(), AppError> {
    match command {
        AccountCommands::Add { id, username, kind, token, default_org, protocol, clone_dir } => {
            account::add(
                storage,
                &id,
                &username,
                kind.into(),
                &token,
                default_org,
                protocol.into(),
                clone_dir,
            )?;
            println!("✅ Added account '{id}'");
        }
        AccountCommands::List => {
            let accounts = account::list(storage)?;
            let all = accounts.all_accounts();

            if all.is_empty() {
                println!("No accounts configured.");
                return Ok(());
            }

            println!("📋 Accounts:");
            for acc in all {
                let active = accounts.active_account_id.as_deref() == Some(&acc.id);
                let marker = if active { " (active)" } else { "" };
                println!(
                    "  {} ({}) - {} [{}]{}",
                    acc.id, acc.kind, acc.username, acc.protocol, marker
                );
            }
        }
        AccountCommands::Use { id } => {
            let selected = match id {
                Some(id) => {
                    account::switch(storage, &id)?;
                    id
                }
                None => account::switch_interactive(storage)?,
            };
            println!("✅ Switched to account '{selected}'");
        }
        AccountCommands::Show => {
            let acc = account::show(storage)?;
            let token = keychain::get_token(&acc.id).unwrap_or_else(|_| "(not found)".to_string());
            let masked = keychain::mask_token(&token);

            println!("🔑 Active account:");
            println!("  ID:       {}", acc.id);
            println!("  Kind:     {}", acc.kind);
            println!("  Username: {}", acc.username);
            println!("  Protocol: {}", acc.protocol);
            println!("  Token:    {}", masked);
            if let Some(org) = &acc.default_org {
                println!("  Org:      {}", org);
            }
            if let Some(dir) = &acc.clone_dir {
                println!("  Clone:    {}", dir);
            }
        }
        AccountCommands::Remove { id } => {
            account::remove(storage, &id)?;
            println!("🗑️  Removed account '{id}'");
        }
    }
    Ok(())
}

fn run_repo_command(storage: &FilesystemStorage, command: RepoCommands) -> Result<(), AppError> {
    match command {
        RepoCommands::List { org, limit, json } => {
            let repos = repo::list(storage, org.as_deref(), limit)?;

            if json {
                for r in repos {
                    let output = serde_json::json!({
                        "name": r.name,
                        "url": r.html_url,
                        "pushed_at": r.pushed_at,
                        "owner": r.owner.login,
                    });
                    println!("{}", serde_json::to_string(&output).unwrap());
                }
            } else {
                for r in repos {
                    println!("{} {}", r.full_name, r.html_url);
                }
            }
        }
        RepoCommands::Clone { repo, org, limit } => {
            if let Some(org) = org {
                let cloned = repo::clone_org(storage, &org, limit)?;
                if cloned.is_empty() {
                    println!("No repositories cloned.");
                } else {
                    println!("✅ Cloned {} repositories:", cloned.len());
                    for name in cloned {
                        println!("  - {name}");
                    }
                }
            } else if let Some(repo_spec) = repo {
                repo::clone(storage, &repo_spec)?;
                println!("✅ Cloned '{repo_spec}'");
            } else {
                return Err(AppError::invalid_input(
                    "provide either a repo (owner/repo) or --org flag",
                ));
            }
        }
    }
    Ok(())
}

fn run_pr_command(storage: &FilesystemStorage, command: PrCommands) -> Result<(), AppError> {
    match command {
        PrCommands::List { repo, limit } => {
            let prs = pr::list(storage, repo.as_deref(), limit)?;

            for p in prs {
                let output = serde_json::to_string(&p).unwrap();
                println!("{output}");
            }
        }
    }
    Ok(())
}
