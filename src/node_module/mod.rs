// use lazy_static::lazy_static;
use serde::Deserialize;
use std::cell::Cell;
use std::cmp::{Ord, Ordering};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

pub enum NodeModuleTypes {
    Production,
    Development,
    Peer,
}

///
/// A NodeJS module type. The `name` takes any of the following.
/// - production
/// - development
/// - peer
///
#[derive(PartialEq, Eq, PartialOrd, Default, Debug)]
pub struct NodeModuleType {
    pub name: String,
    pub short: char,
}

impl From<NodeModuleTypes> for NodeModuleType {
    fn from(types: NodeModuleTypes) -> Self {
        match types {
            NodeModuleTypes::Production => NodeModuleType {
                name: "production".to_owned(),
                short: 'S',
            },
            NodeModuleTypes::Development => NodeModuleType {
                name: "development".to_owned(),
                short: 'D',
            },
            NodeModuleTypes::Peer => NodeModuleType {
                name: "peer".to_owned(),
                short: 'P',
            },
        }
    }
}

impl Ord for NodeModuleType {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.short == 'S' {
            if other.short == 'S' {
                Ordering::Equal
            } else {
                Ordering::Greater
            }
        } else if self.short == 'D' {
            if other.short == 'P' {
                Ordering::Greater
            } else if other.short == 'D' {
                Ordering::Equal
            } else {
                Ordering::Less
            }
        } else {
            if other.short == 'P' {
                Ordering::Equal
            } else {
                Ordering::Less
            }
        }
    }
}

// lazy_static! {
//   pub static ref PRODUCTION_TYPE: NodeModuleType = NodeModuleType {
//     name: String::from("production"),
//     short: 'S',
//   };
//   pub static ref DEVELOPMENT_TYPE: NodeModuleType = NodeModuleType {
//     name: String::from("development"),
//     short: 'D',
//   };
//   pub static ref PEER_TYPE: NodeModuleType = NodeModuleType {
//     name: String::from("peer"),
//     short: 'P',
//   };
// }

///
/// A NodeJS module
///
#[derive(PartialEq, Eq, PartialOrd, Default, Debug)]
pub struct NodeModule {
    pub r#type: NodeModuleType,
    pub name: String,
    pub version: String,
    pub path: PathBuf,
    pub selected: Cell<bool>,
}

impl NodeModule {
    pub fn new_production(name: String, version: String, path: PathBuf) -> Self {
        NodeModule {
            r#type: NodeModuleType::from(NodeModuleTypes::Production),
            name: name,
            version: version,
            path: path,
            ..Default::default()
        }
    }

    pub fn new_development(name: String, version: String, path: PathBuf) -> Self {
        NodeModule {
            r#type: NodeModuleType::from(NodeModuleTypes::Development),
            name: name,
            version: version,
            path: path,
            ..Default::default()
        }
    }

    pub fn new_peer(name: String, version: String, path: PathBuf) -> Self {
        NodeModule {
            r#type: NodeModuleType::from(NodeModuleTypes::Peer),
            name: name,
            version: version,
            path: path,
            ..Default::default()
        }
    }
}

impl Ord for NodeModule {
    fn cmp(&self, other: &Self) -> Ordering {
        let type_ordering = self.r#type.cmp(&other.r#type);
        let name_ordering = self.name.cmp(&other.name);

        match (type_ordering, name_ordering) {
            (Ordering::Less, _) | (Ordering::Equal, Ordering::Less) => Ordering::Less,
            (Ordering::Greater, _) | (Ordering::Equal, Ordering::Greater) => Ordering::Greater,
            _ => Ordering::Equal,
        }
    }
}

///
/// The type of the package.json or like
///
#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
struct Package {
    dependencies: Option<HashMap<String, String>>,
    devDependencies: Option<HashMap<String, String>>,
    peerDependencies: Option<HashMap<String, String>>,
}

///
/// Function collects the dependencies of the nodejs project recursively
///
/// ```no_run
/// use tows::node_module::collect_dependencies;
///
/// collect_dependencies(
///   &std::env::current_dir().unwrap().join("path/to/nodejs-project"),
///   "package.json",
/// );
///```
///
pub fn collect_dependencies(current_dir: &Path, filename: &str) -> HashMap<String, NodeModule> {
    let mut dependency_map: HashMap<String, NodeModule> = HashMap::new();
    let mut absolute_filepath = current_dir.join(filename);

    let mut dirname = current_dir;
    while let Ok(json_str) = fs::read_to_string(&absolute_filepath) {
        let deserialize: Package =
            serde_json::from_str(&json_str).expect("The package.json seems not json file");

        if let Some(dependencies) = deserialize.dependencies {
            for key in dependencies.keys() {
                let key_string = key.to_string();

                if let None = dependency_map.get(&key_string) {
                    if let Some(key_value) = dependencies.get(&key_string) {
                        dependency_map.insert(
                            key_string.clone(),
                            NodeModule::new_production(
                                key_string.clone(),
                                key_value.to_owned(),
                                absolute_filepath.to_owned(),
                            ),
                        );
                    }
                }
            }
        }

        if let Some(dependencies) = deserialize.devDependencies {
            for key in dependencies.keys() {
                let key_string = key.to_string();

                if let None = dependency_map.get(&key_string) {
                    if let Some(key_value) = dependencies.get(&key_string) {
                        dependency_map.insert(
                            key_string.clone(),
                            NodeModule::new_development(
                                key_string.clone(),
                                key_value.to_owned(),
                                absolute_filepath.to_owned(),
                            ),
                        );
                    }
                }
            }
        }

        if let Some(dependencies) = deserialize.peerDependencies {
            for key in dependencies.keys() {
                let key_string = key.to_string();

                if let None = dependency_map.get(&key_string) {
                    if let Some(key_value) = dependencies.get(&key_string) {
                        dependency_map.insert(
                            key_string.clone(),
                            NodeModule::new_peer(
                                key_string.clone(),
                                key_value.to_owned(),
                                absolute_filepath.to_owned(),
                            ),
                        );
                    }
                }
            }
        }

        if let Some(parent_dir) = &dirname.parent() {
            dirname = parent_dir;
        } else {
            break;
        }

        absolute_filepath = dirname.join(filename);
    }

    dependency_map
}

#[test]
fn collect_dependencies_returns_correct_map() {
    use std::env;

    let result = collect_dependencies(
        &env::current_dir()
            .unwrap()
            .join("reproduce/parent-node-project/child-node-project"),
        "package.json",
    );

    assert_eq!(
        result.get("typescript"),
        Some(&NodeModule {
            r#type: NodeModuleType::from(NodeModuleTypes::Production),
            name: "typescript".to_owned(),
            version: "^3.8.3".to_owned(),
            path: env::current_dir()
                .unwrap()
                .join("reproduce/parent-node-project/child-node-project/package.json"),
            selected: Cell::new(false),
        })
    );

    assert_eq!(
        result.get("redux"),
        Some(&NodeModule {
            r#type: NodeModuleType::from(NodeModuleTypes::Production),
            name: "redux".to_owned(),
            version: "^4.0.5".to_owned(),
            path: env::current_dir()
                .unwrap()
                .join("reproduce/parent-node-project/package.json"),
            selected: Cell::new(false),
        })
    );

    assert_eq!(
        result.get("eslint"),
        Some(&NodeModule {
            r#type: NodeModuleType::from(NodeModuleTypes::Development),
            name: "eslint".to_owned(),
            version: "^6.8.0".to_owned(),
            path: env::current_dir()
                .unwrap()
                .join("reproduce/parent-node-project/child-node-project/package.json"),
            selected: Cell::new(false),
        })
    );

    assert_eq!(
        result.get("react"),
        Some(&NodeModule {
            r#type: NodeModuleType::from(NodeModuleTypes::Peer),
            name: "react".to_owned(),
            version: "^16.13.0".to_owned(),
            path: env::current_dir()
                .unwrap()
                .join("reproduce/parent-node-project/package.json"),
            selected: Cell::new(false),
        })
    );

    assert_eq!(
        result.get("react-dom"),
        Some(&NodeModule {
            r#type: NodeModuleType::from(NodeModuleTypes::Peer),
            name: "react-dom".to_owned(),
            version: "^16.13.0".to_owned(),
            path: env::current_dir()
                .unwrap()
                .join("reproduce/parent-node-project/package.json"),
            selected: Cell::new(false),
        })
    );

    assert_eq!(
        result.get("next"),
        Some(&NodeModule {
            r#type: NodeModuleType::from(NodeModuleTypes::Peer),
            name: "next".to_owned(),
            version: "^9.3.0".to_owned(),
            path: env::current_dir()
                .unwrap()
                .join("reproduce/parent-node-project/child-node-project/package.json"),
            selected: Cell::new(false),
        })
    );
}
