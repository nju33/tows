use std::env;
use std::path::PathBuf;
use tows::{cli, node_module, terminal};

fn main() {
    let matches = cli::build();
    let mut current_dir: PathBuf = env::current_dir().expect("failed when to get current dir");

    if let Some(cwd) = matches.value_of("cwd") {
        if env::set_current_dir(cwd).is_ok() {
            current_dir = env::current_dir().unwrap();
        } else {
            panic!(format!("The cwd value is imcorrect path: {}", cwd));
        }
    }

    let current_dir = current_dir;
    let filename = matches
        .value_of("filenamme")
        .unwrap_or(cli::DEFAULT_FILE_NAME);
    let dependency_map = node_module::collect_dependencies(&current_dir, &filename);
    let mut dependency_list = dependency_map
        .values()
        .collect::<Vec<&node_module::NodeModule>>();
    dependency_list.sort_by(|lhs, rhs| lhs.cmp(rhs));
    let dependency_list = dependency_list
        .into_iter()
        .rev()
        .collect::<Vec<&node_module::NodeModule>>();

    if dependency_list.len() == 0 {
        eprintln!(
            "Warning: {} is not found one, even though it has looked for from {:?}",
            filename, current_dir
        );
        std::process::exit(1);
    }

    terminal::render(&dependency_list);

    let package_list = dependency_list
        .into_iter()
        .filter(|node_module| node_module.selected.get())
        .map(|node_module| format!("{}@{}", node_module.name, node_module.version))
        .collect::<Vec<String>>();

    print!("{}", package_list.join(" "));
}
