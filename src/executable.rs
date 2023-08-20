use crate::ssh::SSHClient;
use crate::Result;
use anyhow::bail;
use bytes::Bytes;
use std::path::{Path, PathBuf};
use std::rc::Rc;

#[derive(Clone)]
pub(crate) struct Executable {
    client: Rc<SSHClient>,
    ///可执行文件的名称
    name: String,
    ///远程可执行文件的路径
    remote_path: PathBuf,
    ///本地可执行文件的名称
    local_path: PathBuf,
}

impl Executable {
    pub(crate) fn new(
        client: Rc<SSHClient>,
        name: String,
        remote_path: impl AsRef<Path>,
        local_path: impl AsRef<Path>,
    ) -> Self {
        let remote_path = remote_path.as_ref().to_path_buf();
        let local_path = local_path.as_ref().to_path_buf();
        if name.is_empty()
            || remote_path.to_string_lossy().is_empty()
            || local_path.to_string_lossy().is_empty()
        {
            panic!("name or remote_path or local_path is empty");
        }
        Self {
            client,
            name,
            remote_path,
            local_path,
        }
    }

    ///推送可执行文件到远程服务器
    pub(crate) fn push_executable_to_remote_server(&self) -> Result<()> {
        let from = self.local_path.join(self.name.as_str());
        let to = self
            .remote_path
            .join(format!("{}.temp", self.name).as_str());
        if !self.client.dir_exists(self.remote_path.as_path())? {
            self.client
                .sftp()?
                .mkdir(self.remote_path.as_path(), 0o777)?;
        }
        self.client
            .upload(to.as_path(), Bytes::from(std::fs::read(from)?), 0o777)?;
        Ok(())
    }

    ///kill远程服务器上的进程
    pub(crate) fn killall_remote_server_process(&self) -> Result<()> {
        if self.check_remote_server_process_is_running()? {
            self.client
                .exec(format!("killall -9 {}", self.name).as_str())?;
        }
        Ok(())
    }

    ///删除远程服务器上的文件并修改上传文件的文件名
    /// from: filename.tmp
    /// to: filename
    pub(crate) fn delete_remote_server_file_and_rename(&self) -> Result<()> {
        let from = self
            .remote_path
            .join(format!("{}.temp", self.name).as_str());
        let to = self.remote_path.join(self.name.as_str());
        if from.is_dir() || to.is_dir() {
            bail!("from: {} or to: {} is dir", from.display(), to.display());
        }
        self.client
            .exec(format!("rm -rf {}", to.display()).as_str())?;
        self.client
            .exec(format!("mv {} {}", from.display(), to.display()).as_str())?;
        Ok(())
    }

    ///启动远程服务器上的进程
    pub(crate) fn start_remote_server_process(&self) -> Result<()> {
        //nohup /mnt/usb/disk1/picture/aml-picture > /mnt/usb/disk1/picture/aml-picture.log 2>&1 &
        let cmd = format!(
            "nohup {0} > {0}.log 2>&1 &",
            self.remote_path.join(self.name.as_str()).display()
        );
        self.client.exec(cmd.as_str())?;
        Ok(())
    }

    ///查看程序是否在运行
    pub(crate) fn check_remote_server_process_is_running(&self) -> Result<bool> {
        let result = self
            .client
            .exec(format!("ps -ef | grep {0} | grep -v grep | wc -l", self.name).as_str())?;
        let is_running = result.trim() == "1";

        Ok(is_running)
    }
    ///查看程序日志
    pub(crate) fn show_remote_server_process_log(&self) -> Result<String> {
        let log = self.client.exec(
            format!(
                "cat {}.log",
                self.remote_path.join(self.name.as_str()).display()
            )
            .as_str(),
        )?;
        Ok(log)
    }
    ///查看程序日志的最后n行
    pub(crate) fn show_remote_server_process_log_last(&self, n: u32) -> Result<String> {
        let log = self.client.exec(
            format!(
                "tail -n {} {}.log",
                n,
                self.remote_path.join(self.name.as_str()).display()
            )
            .as_str(),
        )?;
        Ok(log)
    }
}
