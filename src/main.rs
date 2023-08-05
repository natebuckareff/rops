use std::{
    fs::{self, create_dir_all},
    path::PathBuf,
};

use age::secrecy::ExposeSecret;
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: CliCommands,
}

#[derive(Subcommand, Debug)]
enum CliCommands {
    Identity {
        #[command(subcommand)]
        command: IdentityCommands,
    },
}

#[derive(Subcommand, Debug, Clone)]
enum IdentityCommands {
    /// List all identities
    Ls,

    /// Derive public key from identity
    Pubkey { name: String },

    /// Generate new public/private identity key pair
    Keygen { name: String },

    /// Import an existing age identity file
    Import {
        name: String,

        #[arg(short, long)]
        file: String,
    },

    /// Delete an identity
    Delete { name: String },
}

fn main() {
    let cli = Cli::parse();

    // println!("{:?}", cli);

    match cli.command {
        CliCommands::Identity { ref command } => cli_identity(command),
        _ => {}
    }

    initialize_config();
    initialize_vault(&".".into());

    /* * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * */

    // let key = age::x25519::Identity::generate();
    // let public_key = key.to_public();
    // let plaintext = b"Hello, world!\n";

    // let encrypted = {
    //     let encryptor = age::Encryptor::with_recipients(vec![Box::new(public_key)])
    //         .expect("we provided a recipient");

    //     let mut encrypted = vec![];
    //     let mut writer = encryptor.wrap_output(&mut encrypted).unwrap();
    //     writer.write_all(plaintext).unwrap();
    //     writer.finish().unwrap();

    //     encrypted
    // };

    // let decrypted = {
    //     let decryptor = match age::Decryptor::new(&encrypted[..]).unwrap() {
    //         age::Decryptor::Recipients(d) => d,
    //         _ => unreachable!(),
    //     };

    //     // decryptor.into::<age::BaseDecryptor>>();

    //     let mut decrypted = vec![];
    //     let mut reader = decryptor
    //         .decrypt(iter::once(&key as &dyn age::Identity))
    //         .unwrap();

    //     reader.read_to_end(&mut decrypted);

    //     decrypted
    // };
}

fn cli_identity(command: &IdentityCommands) {
    match command {
        IdentityCommands::Ls => cli_identity_ls(),
        IdentityCommands::Pubkey { name } => cli_identity_pubkey(name),
        IdentityCommands::Keygen { name } => cli_identity_keygen(name),
        IdentityCommands::Import { name, file } => cli_identity_import(name, file),
        IdentityCommands::Delete { name } => {
            todo!()
        }
    }
}

fn cli_identity_ls() {
    initialize_config();

    let home_dir = dirs::home_dir().expect("user's home directory");
    let identity_dir = home_dir.join(".config/rops/identities");
    let paths = fs::read_dir(&identity_dir).expect("identity config directory");

    let mut width_name = "NAME".len();
    let mut width_public_key = "PUBLIC KEY".len();

    let mut rows = vec![];
    // let mut cell_name = vec![];
    // let mut cell_public_key = vec![];
    // let mut cell_created = vec![];

    for path in paths {
        let entry = path.unwrap();
        let filename = entry.file_name().into_string().expect("identity filename");
        let filepath = entry.path(); //identity_dir.join(&filename);

        let filepath_string = filepath
            .into_os_string()
            .into_string()
            .expect("filepath as string");

        let identity_file =
            age::IdentityFile::from_file(filepath_string).expect("age identity file");

        let identities = identity_file.into_identities();

        assert!(
            identities.len() == 1,
            "only one age identity per identity file is supported"
        );

        let identity = &identities[0];

        let public_key = match identity {
            age::IdentityFileEntry::Native(id) => id.to_public().to_string(),
        };

        let created = entry
            .metadata()
            .expect("identity file metadata")
            .created()
            .expect("identity file creation time")
            .elapsed()
            .unwrap();

        // let created_exact_string = humantime::format_duration(created_duration).to_string();

        // let created =
        //     format_duration(true, &created_exact_string).expect("formatted creation timestamp");

        width_name = filename.len().max(width_name);
        width_public_key = public_key.len().max(width_public_key);
        rows.push((filename, public_key, created));
    }

    rows.sort_by(|a, b| {
        return a.2.cmp(&b.2);
    });

    println!(
        "{0: <width_name$}   {1: <width_public_key$}   {2:}",
        "NAME",
        "PUBLIC KEY",
        "CREATED",
        width_name = width_name,
        width_public_key = width_public_key,
    );

    for (name, public_key, created) in rows {
        let created_exact_string = humantime::format_duration(created).to_string();

        let created =
            format_duration(true, &created_exact_string).expect("formatted creation timestamp");

        println!(
            "{0: <width_name$}   {1: <width_public_key$}   {2:}",
            name,
            public_key,
            created,
            width_name = width_name,
            width_public_key = width_public_key,
        );
    }
}

fn cli_identity_pubkey(name: &str) {
    initialize_config();

    let home_dir = dirs::home_dir().expect("user's home directory");
    let identity_dir = home_dir.join(".config/rops/identities");
    let identity_path = identity_dir.join(name);

    if !identity_path.exists() {
        panic!("identity not found");
    }

    let filepath_string = identity_path.into_os_string().into_string().unwrap();
    let identity_file = age::IdentityFile::from_file(filepath_string).expect("age identity file");
    let identities = identity_file.into_identities();

    assert!(
        identities.len() == 1,
        "only one age identity per identity file is supported"
    );

    let identity = &identities[0];

    let public_key = match identity {
        age::IdentityFileEntry::Native(id) => id.to_public().to_string(),
    };

    println!("{}", public_key);
}

fn cli_identity_keygen(name: &str) {
    initialize_config();

    let home_dir = dirs::home_dir().expect("user's home directory");
    let identity_dir = home_dir.join(".config/rops/identities");
    let identity_path = identity_dir.join(name);

    if identity_path.exists() {
        panic!("identity already exists");
    }

    let identity = age::x25519::Identity::generate();
    let pubkey = identity.to_public();
    let created = humantime::format_rfc3339_millis(std::time::SystemTime::now());

    let mut identity_file = std::fs::File::options()
        .create_new(true)
        .write(true)
        .open(identity_path)
        .expect("create new identity file");

    use std::io::Write;

    let pubkey_string = pubkey.to_string();

    write!(identity_file, "# created: {}\n", created).unwrap();
    write!(identity_file, "# public key: {}\n", pubkey_string).unwrap();
    write!(identity_file, "{}\n", identity.to_string().expose_secret()).unwrap();

    println!("{}", pubkey_string);
}

fn cli_identity_import(name: &str, file: &str) {
    initialize_config();

    let home_dir = dirs::home_dir().expect("user's home directory");
    let identity_dir = home_dir.join(".config/rops/identities");
    let identity_path = identity_dir.join(name);

    if identity_path.exists() {
        panic!("identity already exists");
    }

    if file == "-" {
        todo!()
    } else {
        // let identity_file =
        //     age::IdentityFile::from_file(file.to_string()).expect("age identity file");
        todo!()
    }
}

fn initialize_config() {
    let home_dir = dirs::home_dir().expect("user's home directory");
    create_dir_all(home_dir.join(".config/rops/identities"))
        .expect("created local configuration directory");
}

fn initialize_vault(project_dir: &PathBuf) {
    create_dir_all(project_dir.join(".rops/vault/recipients"))
        .and_then(|_| create_dir_all(project_dir.join("vault/secrets")))
        .expect("created vault directory");
}

// Adapted from: https://github.com/lryong/timediff/blob/main/src/locale/en_us.rs
pub fn format_duration(
    before_current_ts: bool,
    duration_str: &String,
) -> Result<String, humantime::DurationError> {
    if before_current_ts {
        match humantime::parse_duration(duration_str) {
            Ok(v) => match v.as_secs() {
                x if x <= 44 => {
                    return Ok("a few seconds ago".to_string());
                }
                x if x <= 89 => {
                    return Ok("a minute ago".to_string());
                }
                x if x <= 44 * 60 => {
                    let m = x as f32 / 60_f32;
                    return Ok(format!("{:.0} minutes ago", m.ceil()));
                }
                x if x <= 89 * 60 => {
                    return Ok("an hour ago".to_string());
                }
                x if x <= 21 * 60 * 60 => {
                    let h = x as f32 / 60_f32 / 60_f32;
                    return Ok(format!("{:.0} hours ago", h.ceil()));
                }
                x if x <= 35 * 60 * 60 => {
                    return Ok("a day ago".to_string());
                }
                x if x <= 25 * 24 * 60 * 60 => {
                    let d = x as f32 / 24_f32 / 60_f32 / 60_f32;
                    return Ok(format!("{:.0} days ago", d.ceil()));
                }
                x if x <= 45 * 24 * 60 * 60 => {
                    return Ok("a month ago".to_string());
                }
                x if x <= 10 * 30 * 24 * 60 * 60 => {
                    let m = x as f32 / 30_f32 / 24_f32 / 60_f32 / 60_f32;
                    return Ok(format!("{:.0} months ago", m));
                }
                x if x <= 17 * 30 * 24 * 60 * 60 => {
                    return Ok("a year ago".to_string());
                }
                _ => {
                    let y = v.as_secs_f32() / 12_f32 / 30_f32 / 24_f32 / 60_f32 / 60_f32;
                    return Ok(format!("{:.0} years ago", y));
                }
            },
            Err(e) => return Err(e),
        }
    }
    match humantime::parse_duration(duration_str) {
        Ok(v) => match v.as_secs() {
            x if x <= 44 => Ok("in a few seconds".to_string()),
            x if x <= 89 => Ok("in a minute".to_string()),
            x if x <= 44 * 60 => {
                let m = x as f32 / 60_f32;
                return Ok(format!("in {:.0} minutes", m.ceil()));
            }
            x if x <= 89 * 60 => Ok("in an hour".to_string()),
            x if x <= 21 * 60 * 60 => {
                let h = x as f32 / 60_f32 / 60_f32;
                return Ok(format!("in {:.0} hours", h.ceil()));
            }
            x if x <= 35 * 60 * 60 => Ok("in a day".to_string()),
            x if x <= 25 * 24 * 60 * 60 => {
                let d = x as f32 / 24_f32 / 60_f32 / 60_f32;
                return Ok(format!("in {:.0} days", d.ceil()));
            }
            x if x <= 45 * 24 * 60 * 60 => Ok("in a month".to_string()),
            x if x <= 10 * 30 * 24 * 60 * 60 => {
                let m = x as f32 / 30_f32 / 24_f32 / 60_f32 / 60_f32;
                return Ok(format!("in {:.0} months", m));
            }
            x if x <= 17 * 30 * 24 * 60 * 60 => Ok("in a year".to_string()),
            _ => {
                let y = v.as_secs_f32() / 12_f32 / 30_f32 / 24_f32 / 60_f32 / 60_f32;
                return Ok(format!("in {:.0} years", y));
            }
        },
        Err(e) => Err(e),
    }
}
