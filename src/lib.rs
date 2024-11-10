use std::{
    collections::HashMap,
    ffi::OsStr,
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf},
    str::FromStr,
};

pub type PreparedConfigMap = HashMap<String, LinkConfig>;

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct LinkConfig {
    #[serde(rename = "link-path")]
    pub link_path: PathBuf,

    #[serde(rename = "link-to")]
    pub link_to: PathBuf,

    cond: Option<String>,

    #[serde(skip)]
    pub apply: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct ConflinkConfig {
    #[serde(rename = "working-dir")]
    working_dir: Option<PathBuf>,
    #[serde(rename = "link-from-dir")]
    link_from_dir: Option<PathBuf>,
    #[serde(default, rename = "link-all")]
    link_all: bool,

    conflink: HashMap<String, LinkConfig>,

    #[serde(skip)]
    cache: Vec<LinkConfig>,
}

impl ConflinkConfig {
    pub fn prepare_links(&mut self) -> Vec<LinkConfig> {
        let mut prepared_configs: PreparedConfigMap = PreparedConfigMap::new();

        self.prepare_link_configs(&mut prepared_configs);

        self.prepare_general_links(&mut prepared_configs);

        prepared_configs.into_values().collect()
    }

    fn prepare_general_links(&mut self, config_map: &mut PreparedConfigMap) {
        if !self.link_all {
            println!("INFO: link-all is either not defined or set to false. Will not link all.");
            return;
        }

        let Some(working_dir) = self.working_dir.take() else {
            if self.link_from_dir.is_some() {
                eprintln!("ERROR: Missing working dir, can't link all files...");
            }

            return;
        };

        let working_dir = replace_envs_in_path(working_dir);

        let Some(from_dir) = self.link_from_dir.take() else {
            if self.working_dir.is_some() {
                eprintln!("ERROR: missing link-from-dir, can't link all files.");
            }
            // Nothing to do, no dir to link from...
            return;
        };
        let from_dir = replace_envs_in_path(from_dir);

        let read_dir = ignore::WalkBuilder::new(&from_dir)
            .max_depth(Some(1))
            .build();

        for entry in read_dir {
            let Ok(entry) = entry else {
                continue;
            };

            let path = entry.path();

            let Some(filename) = path.file_name() else {
                continue;
            };

            config_map
                .entry(filename.to_string_lossy().to_string())
                .or_insert_with(|| LinkConfig {
                    link_path: working_dir.join(filename),
                    link_to: from_dir.join(filename),
                    cond: None,
                    apply: true,
                });
        }
    }

    fn prepare_link_configs(&mut self, config_map: &mut PreparedConfigMap) {
        if self.conflink.is_empty() {
            return;
        }

        for (key, mut link_config) in self.conflink.drain() {
            if let Some(conditional) = link_config.cond.as_ref() {
                let should_apply = match Self::eval_cond(conditional) {
                    Some(should_apply) => should_apply,
                    None => {
                        eprintln!("Invalid conditional: '{conditional}', won't apply {key}.");
                        false
                    }
                };

                link_config.apply = should_apply;
            }

            if link_config.apply {
                link_config.link_path = replace_envs_in_path(link_config.link_path);
                link_config.link_to = replace_envs_in_path(link_config.link_to);
                config_map.insert(key, link_config);
            }
        }
    }

    fn eval_cond(cond: &str) -> Option<bool> {
        let open_paren = cond.find("(")?;
        let comma = cond.find(',')?;
        let close_paren = cond.find(")")?;
        let operation = cond.get(0..open_paren)?;

        let operation = Operation::from_str(operation).ok()?;
        let first_operand = cond.get(open_paren + 1..comma)?.trim();
        let second_operand = cond.get(comma + 1..close_paren)?.trim();

        let result = operation.eval(first_operand, second_operand);

        Some(result)
    }
}

#[derive(Debug)]
enum Operation {
    Eq,
}

impl FromStr for Operation {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "eq" => Ok(Self::Eq),
            _ => Err(format!("Unsupported operation: '{s}'")),
        }
    }
}

impl Operation {
    fn eval(&self, first_operand: &str, second_operand: &str) -> bool {
        println!("\toperation = {self:?}");
        println!("\tfirst_operand = {first_operand}");
        println!("\tsecond_operand = {second_operand}");

        let Some(env_variable) = first_operand.strip_prefix('$') else {
            return false;
        };

        if env_variable == "hostname" {
            let uname = rustix::system::uname();
            let hostname = uname.nodename().to_string_lossy();
            return hostname == second_operand;
        }

        match std::env::var(env_variable) {
            Ok(env_variable_value) => {
                return env_variable_value == second_operand;
            }
            Err(err) => {
                eprintln!("Error = {err}");
            }
        }

        false
    }
}

fn replace_envs_in_path(path: PathBuf) -> PathBuf {
    if !path.as_os_str().as_bytes().contains(&b'$') {
        return path;
    }

    let mut res = PathBuf::new();

    for component in path.components() {
        if component.as_os_str().as_bytes().first() == Some(&b'$') {
            let value = component.as_os_str().as_bytes().get(1..).and_then(|bytes| {
                let value = std::env::var_os(OsStr::from_bytes(bytes))?;
                Some(value)
            });

            let Some(value) = value else {
                continue;
            };

            res.push(value);
        } else {
            res.push(component);
        }
    }

    res
}
