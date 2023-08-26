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
use tokio::fs;

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
            if log.all_id {
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
                return Ok(());
            }
            if log.all_config.is_some() {
                let exe_list = get_all_config_executable(log.all_config.clone().unwrap()).await?;
                let mut all_no_running = Vec::new();
                let mut all_task_count = 0;
                let mut all_no_running_task_count = 0;

                exe_list.iter().for_each(|e| {
                    println!("\nconfig: {}", style(e.0.as_str()).yellow());
                    let mut no_running = Vec::new();

                    e.1.iter().for_each(|(id, x)| {
                        all_task_count += 1;
                        let is_running = x.check_remote_server_process_is_running().unwrap();
                        let is_running = if is_running {
                            style(is_running).green()
                        } else {
                            all_no_running_task_count += 1;
                            no_running.push(id);
                            style(is_running).red()
                        };
                        println!("id: {} is running: {}", style(id).cyan(), is_running);
                    });

                    if no_running.len() == 0 {
                        println!("There are a {} of ten tasks, all of which are running.", style(e.1.len()).cyan());
                    } else {
                        println!("There are a total of {} tasks, of which {} are not running.", style(e.1.len()).cyan(), style(no_running.len()).red());
                        print!("id: ");
                        no_running.iter().for_each(|id| {
                            print!("{} ", style(id).red());
                        });
                        println!("are not running.");
                        all_no_running.push((e.0.as_str().to_owned(), no_running));
                    }
                    println!("---------------------");
                });

                if all_no_running.len() == 0 {
                    println!("There are a {} of ten tasks, all of which are running.", style(all_task_count).cyan());
                }else {
                    println!("There are a total of {} tasks, of which {} are not running.", style(all_task_count).cyan(), style(all_no_running_task_count).red());
                    all_no_running.iter().for_each(|(config_name, no_running_id)| {
                        println!("config: {}", style(config_name).yellow());
                        print!("id: ");
                        no_running_id.iter().for_each(|id| {
                            print!("{} ", style(id).red());
                        });
                        println!("\n");
                    });
                }
                return Ok(());
            }
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

async fn get_all_config_executable(
    config_dir: Box<Path>,
) -> Result<Vec<(String, Vec<(String, Executable)>)>> {
    let mut config_files = Vec::new();

    let mut dir = fs::read_dir(config_dir.as_ref()).await?;
    while let Some(d) = dir.next_entry().await? {
        let path = d.path();
        if path.is_file() {
            let filename = path.file_name().unwrap().to_str().unwrap();
            if filename.starts_with("exe-") && filename.ends_with("yaml") {
                config_files.push(path);
            }
        }
    }

    let mut config_and_ssh_client_list = Vec::new();

    for config_file in config_files {
        let config = Config::load(config_file.as_path()).await?;
        let ssh_account = &config.ssh_account;
        let ssh_client = Rc::new(get_ssh_client(
            &ssh_account.addr,
            &ssh_account.username,
            &ssh_account.password,
        )?);
        config_and_ssh_client_list.push((config_file, config, ssh_client));
    }

    let executable_list: Vec<_> = config_and_ssh_client_list
        .into_iter()
        .map(|(config_filepath, config, ssh_client)| {
            let config_name = config_filepath
                .file_stem()
                .unwrap()
                .to_str()
                .unwrap()
                .to_owned();

            let exe = config
                .executable_config_list
                .into_iter()
                .map(|e| {
                    let executable = Executable::new(
                        ssh_client.clone(),
                        e.executable_file_name,
                        e.remote_path,
                        e.local_path,
                    );
                    (e.id, executable)
                })
                .collect();
            (config_name, exe)
        })
        .collect();

    Ok(executable_list)
}

#[cfg(test)]
mod test {
    use crate::get_all_config_executable;
    use std::path::Path;

    #[tokio::test]
    async fn t() {}
}
