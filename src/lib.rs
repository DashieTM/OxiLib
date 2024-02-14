use serde::{self, de::DeserializeOwned};
use std::fs;

pub trait Config {
    fn create_from_optional(optional: impl ConfigOptional) -> impl Config;
}

pub trait ConfigOptional {
    fn create() -> Self;
}

pub fn create_config<ConcreteConfig, OptionalConfig>(
    config_file_name: &'static str,
    config_path: &'static str,
    default_config: &'static str,
) -> impl Config
where
    ConcreteConfig: Config,
    OptionalConfig: ConfigOptional + DeserializeOwned,
{
    let base = directories_next::BaseDirs::new().unwrap();
    let config_dir = base.config_dir();
    if !config_dir.is_dir() {
        fs::create_dir(config_dir).expect("Could not create config folder");
    }
    let config_file = &config_dir.join(config_path).join(config_file_name);
    if !config_file.is_file() {
        fs::File::create(config_file).expect("Could not create config file");
    }
    let contents = match fs::read_to_string(config_file) {
        Ok(c) => c,
        Err(_) => default_config.to_string(),
    };
    let parsed_conf: OptionalConfig = match toml::from_str(&contents) {
        Ok(d) => d,
        Err(_) => toml::from_str(default_config).unwrap(),
    };
    ConcreteConfig::create_from_optional(parsed_conf)
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn it_works() {
//         let result = add(2, 2);
//         assert_eq!(result, 4);
//     }
// }
