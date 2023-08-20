use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::exit;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SSHAccount {
    pub(crate) addr: String,
    pub(crate) username: String,
    pub(crate) password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Executable {
    pub(crate) id: String,
    pub(crate) executable_file_name: String,
    pub(crate) local_path: Box<Path>,
    pub(crate) remote_path: Box<Path>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Config {
    pub(crate) ssh_account: SSHAccount,
    pub(crate) executable_config_list: Vec<Executable>,
}

impl Config {
    pub(crate) async fn load(config_path: impl AsRef<Path>) -> crate::Result<Config> {
        let mut config = Config {
            ssh_account: SSHAccount {
                addr: "ip:port".to_owned(),
                username: "username".to_owned(),
                password: "password".to_owned(),
            },
            executable_config_list: vec![Executable {
                id: "default".to_owned(),
                executable_file_name: "executable_file_name".to_owned(),
                local_path: Box::from(Path::new("local_path")),
                remote_path: Box::from(Path::new(
                    "remote_path # need end with /(unix) or \\(windows)",
                )),
            }],
        };
        let config_path = config_path.as_ref();
        if config_path.exists() {
            config = serde_yaml::from_str(&std::fs::read_to_string(config_path)?)?;
        } else {
            let data = serde_yaml::to_string(&config)?;
            std::fs::write(config_path, data)?;
        };

        //判断配置未修改 就退出
        if config.ssh_account.username == "username"
            || config.ssh_account.password == "password"
            || config.ssh_account.addr == "ip:port"
        {
            println!(
                "请修改{}中的 username, password 或 addr",
                config_path.file_name().unwrap().to_str().unwrap()
            );
            exit(0);
        };

        if let Some(e) = config.executable_config_list.get(0) {
            if e.executable_file_name == "executable_file_name"
                || e.local_path.as_os_str() == "local_path"
                || e.remote_path.as_os_str() == "remote_path"
            {
                println!(
                    "请修改{}中的 executable_file_name, local_path 或 remote_path",
                    config_path.file_name().unwrap().to_str().unwrap()
                );
                exit(0);
            }
        }
        Ok(config)
    }
}
