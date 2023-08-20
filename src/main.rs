mod cli;
mod config;
mod executable;
mod ssh;

use crate::cli::Commands;
use crate::config::Config;
use crate::executable::Executable;
use crate::ssh::SSHClient;
pub(crate) use anyhow::Result;
use clap::Parser;
use cli::Cli;
use console::style;
use std::env;
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::rc::Rc;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Run(run) => {
            if run.all_id == true {
                let exe_list = get_all_executable(run.config.clone()).await?;
                exe_list.iter().for_each(|(id, x)| {
                    run_exe(x).unwrap();
                    let is_running = x.check_remote_server_process_is_running().unwrap();
                    let log = x.show_remote_server_process_log().unwrap();

                    let is_running = if is_running {
                        style(is_running).green()
                    } else {
                        style(is_running).red()
                    };

                    println!("log:\n{}\n", log);
                    println!("is running: {}\n", style(is_running).cyan());
                });
            } else {
                let x = get_executable(run.config.clone(), run.id.clone()).await?;
                run_exe(&x).unwrap();
                let is_running = x.check_remote_server_process_is_running().unwrap();
                let log = x.show_remote_server_process_log().unwrap();

                let is_running = if is_running {
                    style(is_running).green()
                } else {
                    style(is_running).red()
                };

                println!("log:\n{}\n", log);
                println!("is running: {}\n", style(is_running).cyan());
            }
        }
        Commands::Log(log) => {
            if log.all_id == true {
                let exe_list = get_all_executable(log.config.clone()).await?;

                let mut running_list = Vec::new();

                exe_list.iter().for_each(|(id, x)| {
                    println!("id: {}", style(id).cyan());
                    let is_running = x.check_remote_server_process_is_running().unwrap();

                    running_list.push((id, is_running));

                    let is_running = if is_running {
                        style(is_running).green()
                    } else {
                        style(is_running).red()
                    };
                    println!(
                        "log: last {} line\n{}",
                        10,
                        x.show_remote_server_process_log_last(10).unwrap()
                    );
                    println!("is running: {}\n", is_running);
                });
                println!("state list:");
                running_list.iter().for_each(|(id, is_running)| {
                    let is_running = if *is_running {
                        style(is_running).green()
                    } else {
                        style(is_running).red()
                    };
                    println!("id: {} is running: {}", style(id).cyan(), is_running);
                });
            } else {
                let x = get_executable(log.config.clone(), log.id.clone()).await?;
                let is_running = x.check_remote_server_process_is_running()?;
                let log = x.show_remote_server_process_log()?;

                let is_running = if is_running {
                    style(is_running).green()
                } else {
                    style(is_running).red()
                };

                println!("log:\n{}\n", log);
                println!("is running: {}\n", is_running);
            }
        }
    }
    Ok(())
}

fn get_config_and_id(config: Option<Box<Path>>, id: Option<String>) -> Result<(Box<Path>, String)> {
    let mut default_config = get_executable_dir()?;
    default_config.push("exe-default-config.yaml");
    let config = config.unwrap_or(Box::from(default_config.as_path()));
    let id = id.unwrap_or("default".to_string());
    Ok((config, id))
}

fn get_executable_dir() -> Result<PathBuf> {
    Ok(env::current_dir()?)
}

fn run_exe(executable: &Executable) -> Result<()> {
    executable.push_executable_to_remote_server()?;
    executable.killall_remote_server_process()?;
    executable.delete_remote_server_file_and_rename()?;
    executable.start_remote_server_process()?;
    Ok(())
}

fn get_ssh_client(host: &str, username: &str, password: &str) -> Result<SSHClient> {
    let tcp = TcpStream::connect(host).unwrap();
    let client = SSHClient::new(tcp);
    client.auth_by_password(username, password);
    Ok(client)
}

async fn get_executable(config: Option<Box<Path>>, id: Option<String>) -> Result<Executable> {
    let config_and_id = get_config_and_id(config, id)?;
    let config = Config::load(config_and_id.0.as_ref()).await?;
    let ssh_account = config.ssh_account;
    let ssh_client = Rc::new(get_ssh_client(
        &ssh_account.addr,
        &ssh_account.username,
        &ssh_account.password,
    )?);
    let executable_config = config
        .executable_config_list
        .iter()
        .find(|x| x.id == config_and_id.1)
        .ok_or(anyhow::anyhow!("not found this id"))?;
    let executable = Executable::new(
        ssh_client,
        executable_config.executable_file_name.clone(),
        executable_config.remote_path.clone(),
        executable_config.local_path.clone(),
    );
    Ok(executable)
}

async fn get_all_executable(config: Option<Box<Path>>) -> Result<Vec<(String, Executable)>> {
    let mut default_config = get_executable_dir()?;
    default_config.push("exe-default-config.yaml");
    let config_path = config.unwrap_or(Box::from(default_config.as_path()));

    let config = Config::load(config_path.as_ref()).await?;
    let ssh_account = config.ssh_account;
    let ssh_client = Rc::new(get_ssh_client(
        &ssh_account.addr,
        &ssh_account.username,
        &ssh_account.password,
    )?);

    let executable_list: Vec<_> = config
        .executable_config_list
        .into_iter()
        .map(|e| {
            (
                e.id,
                Executable::new(
                    ssh_client.clone(),
                    e.executable_file_name,
                    e.remote_path,
                    e.local_path,
                ),
            )
        })
        .collect();

    Ok(executable_list)
}
