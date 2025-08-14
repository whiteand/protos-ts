mod args;
mod proto;
use args::CliArguments;
use clap::Parser;
use path_clean::clean;
use proto::compiler::ts::ast::Folder;
use proto::compiler::ts::commit_folder::commit_folder;
use proto::compiler::ts::scope_to_folder::root_scope_to_folder;
use proto::folder::read_proto_folder;
use std::process;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use proto::package::read_root_scope;

fn main() -> () {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::filter::EnvFilter::from_default_env())
        .init();

    let CliArguments { src, out } = CliArguments::parse();
    let cwd = std::env::current_dir().unwrap();

    let src = clean(cwd.join(src));
    let out = clean(cwd.join(out));
    tracing::trace!(?src, ?out, "resolved args");

    let proto_folder = match read_proto_folder(src) {
        Err(e) => {
            eprintln!("{}", e);
            process::exit(2);
        }
        Ok(r) => r,
    };

    let root_scope = match read_root_scope(&proto_folder.files) {
        Err(e) => {
            eprintln!("{}", e);
            process::exit(3);
        }
        Ok(r) => r,
    };

    let root_file_name: String = out.file_name().map(|s| s.to_string_lossy()).unwrap().into();

    let folder: Folder = match root_scope_to_folder(&root_scope, root_file_name) {
        Err(e) => {
            eprintln!("{}", e);
            process::exit(4);
        }
        Ok(r) => r,
    };

    match commit_folder(&folder) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{}", e);
            process::exit(4);
        }
    }
}
