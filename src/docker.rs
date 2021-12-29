use crate::{project::VirtualEnv, TrampolineResource, TrampolineResourceType};
use std::collections::HashMap;
use std::fmt::Formatter;
use std::io::Write;
use std::marker::PhantomData;
use std::process::Command;

use std::path::{Path, PathBuf};

use thiserror::Error;

use std::process::Stdio;

use std::string::ToString;

pub const DOCKER_BIN: &str = "docker";
pub const IMAGE_NAME: &str = "iamm/trampoline-env:latest";
#[derive(Debug, Error)]
pub enum DockerError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Any(#[from] anyhow::Error),
    #[error("No image set")]
    NoImage,
}
type DockerResult<T> = std::result::Result<T, DockerError>;

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub struct Port(usize);

impl From<usize> for Port {
    fn from(port_num: usize) -> Self {
        Self(port_num)
    }
}

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub struct VolumePath(String);

impl From<String> for VolumePath {
    fn from(path: String) -> Self {
        Self(path)
    }
}

impl From<PathBuf> for VolumePath {
    fn from(buf: PathBuf) -> Self {
        Self(buf.to_str().unwrap().to_string())
    }
}

impl std::fmt::Display for VolumePath {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::fmt::Display for Port {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct Volume<'a> {
    host: &'a Path,
    container: &'a Path,
}
#[derive(Debug, Clone)]
pub struct DockerPort {
    pub host: usize,
    pub container: usize,
}

impl std::fmt::Display for DockerPort {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.host, self.container)
    }
}

#[derive(Debug, Clone)]
pub struct DockerContainer<'a> {
    pub name: String,
    pub port_bindings: Vec<DockerPort>,
    pub volumes: Vec<Volume<'a>>,
    pub env_vars: HashMap<String, String>,
    pub image: &'a DockerImage,
}

impl std::fmt::Display for DockerContainer<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let port_bindings_string = self
            .port_bindings
            .iter()
            .map(|port| format!("-p {}", port))
            .collect::<Vec<String>>()
            .join(" ");
        let image_string = {
            if let Some(tag) = self.image.tag.as_ref() {
                format!("{}:{}", self.image.name, tag)
            } else {
                self.image.name.clone()
            }
        };
        write!(
            f,
            "{} --name {} {}",
            port_bindings_string, self.name, image_string
        )
    }
}

#[derive(Debug, Clone, Default)]
pub struct DockerImage {
    pub name: String,
    pub tag: Option<String>,
    pub file_path: Option<String>,
    pub host_mappings: Vec<Port>,
    pub build_args: HashMap<String, String>,
}

impl std::fmt::Display for DockerImage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut fmt_string = "".to_string();
        if self.file_path.is_some() {
            fmt_string.push_str(&format!("{}", self.file_path.as_ref().unwrap()));
        }
        if self.tag.is_some() {
            fmt_string.push_str(&format!(" -t {}:{}", self.name, self.tag.as_ref().unwrap()));
        } else {
            fmt_string.push_str(&format!("{}", self.name));
        }

        write!(f, "{}", fmt_string)
    }
}

#[derive(Debug, Default)]
pub struct DockerCommand<C> {
    _docker: PhantomData<C>,
    pub command_string: Option<String>,
}

impl<T> Into<String> for DockerCommand<T> {
    fn into(self) -> String {
        self.command_string.unwrap_or_default()
    }
}

impl<T> DockerCommand<T> {
    pub fn execute(&self) -> DockerResult<()> {
        if let Some(cmd_str) = &self.command_string {
            let mut cmd = Command::new(cmd_str);
            cmd.stdout(Stdio::null())
                .stderr(Stdio::null())
                .stdin(Stdio::null())
                .spawn()?;
            Ok(())
        } else {
            Err(DockerError::NoImage)
        }
    }
}
impl DockerCommand<DockerImage> {
    fn format_command(
        &self,
        image: &DockerImage,
        command_string: &str,
        flags: Vec<String>,
    ) -> String {
        let flags_string = flags
            .iter()
            .map(|flag| format!("--{}", flag))
            .collect::<Vec<String>>()
            .join(" ");

        if (&flags).len() > 0 {
            format!("docker image {} {} {}", command_string, flags_string, image)
        } else {
            format!("docker image {} {}", command_string, image)
        }
    }

    pub fn build(&self, image: &DockerImage, rm: bool) -> DockerResult<DockerCommand<DockerImage>> {
        let mut flags = vec![];
        if rm {
            flags.push("rm".to_string());
        }
        let build_command_string = self.format_command(image, "build", flags);
        Ok(DockerCommand::<DockerImage> {
            command_string: Some(build_command_string),
            _docker: PhantomData::<DockerImage>,
        })
    }

    pub fn remove(&self, image: &DockerImage) -> DockerResult<DockerCommand<DockerImage>> {
        let build_command_string = self.format_command(image, "rm", vec![]);
        Ok(DockerCommand::<DockerImage> {
            command_string: Some(build_command_string),
            _docker: PhantomData::<DockerImage>,
        })
    }

    pub fn prune() -> DockerResult<()> {
        Ok(())
    }
}

impl DockerCommand<DockerContainer<'_>> {
    fn format_command(
        &self,
        container: &DockerContainer,
        command_string: &str,
        flags: Vec<String>,
    ) -> String {
        let flags_string = flags
            .iter()
            .map(|flag| format!("--{}", flag))
            .collect::<Vec<String>>()
            .join(" ");

        if (&flags).len() > 0 {
            format!(
                "docker container {} {} {}",
                command_string, flags_string, container
            )
        } else {
            format!("docker container {} {}", command_string, container)
        }
    }
    pub fn run(
        &self,
        container: &DockerContainer,
        rm: bool,
        detach: bool,
    ) -> DockerResult<DockerCommand<DockerContainer>> {
        let mut flags = vec![];
        if rm {
            flags.push("rm".into());
        }
        if detach {
            flags.push("detach".into());
        }
        let run_cmd_str = self.format_command(container, "run", flags);

        Ok(DockerCommand::<DockerContainer> {
            command_string: Some(run_cmd_str),
            _docker: PhantomData::<DockerContainer>,
        })
    }

    pub fn exec(container: &DockerContainer) -> DockerResult<()> {
        Ok(())
    }

    pub fn cp(container: &DockerContainer) -> DockerResult<()> {
        todo!()
    }

    pub fn start(container: &DockerContainer) -> DockerResult<()> {
        todo!()
    }

    pub fn stop(container: &DockerContainer) -> DockerResult<()> {
        todo!()
    }

    pub fn pause(container: &DockerContainer) -> DockerResult<()> {
        todo!()
    }

    pub fn unpause(container: &DockerContainer) -> DockerResult<()> {
        todo!()
    }

    pub fn restart(container: &DockerContainer) -> DockerResult<()> {
        todo!()
    }
}

#[derive(Debug, Default)]
pub struct Docker {
    name: Option<String>,
    port_bindings: Vec<(Port, Port)>,
    env_vars: HashMap<String, String>,
    volumes: HashMap<VolumePath, VolumePath>,
    virtuals: Vec<VirtualEnv>,
}

impl Docker {
    pub fn add_service(mut self, env: VirtualEnv) -> DockerResult<Self> {
        let VirtualEnv {
            host: _,
            container_port,
            local_binding,
            container_mount,
            host_port,
        } = env.clone();
        self.port_bindings
            .push((host_port.into(), container_port.into()));
        let local_binding = local_binding.canonicalize()?;
        self.volumes
            .insert(local_binding.into(), container_mount.into());
        self.virtuals.push(env.clone());

        Ok(self)
    }

    pub fn build(&self) -> DockerResult<()> {
        let mut cmd = Command::new(DOCKER_BIN);
        cmd.arg("build");
        cmd.arg(".");
        cmd.arg("-t");
        cmd.arg(IMAGE_NAME);
        let output = cmd
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()?;
        std::io::stdout().write_all(&output.stdout)?;
        std::io::stdout().write_all(&output.stderr)?;

        Ok(())
    }

    pub fn env_var(mut self, key: String, val: String) -> DockerResult<Self> {
        self.env_vars.insert(key, val);
        Ok(self)
    }

    pub fn name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }

    pub fn exec(container_name: &str, exec_args: Vec<&str>, work_dir: &str) -> DockerResult<()> {
        let mut cmd = Command::new(DOCKER_BIN);
        cmd.args(&["exec", "-d", container_name, "bash", "-c"]);

        let args_string = exec_args.join(" ");
        let args_string = format!(r#"{}"#, args_string);
        println!("Args string: {}", args_string);
        cmd.arg(args_string.as_str());

        println!("{:?}", cmd);
        cmd.stdout(Stdio::null())
            .stderr(Stdio::inherit())
            .stdin(Stdio::null())
            .spawn()?;

        Ok(())
    }

    pub fn restart(&self) -> DockerResult<()> {
        let mut cmd = Command::new(DOCKER_BIN);
        cmd.args(&["restart", self.name.as_ref().unwrap().as_str()]);

        let _child = cmd.stdout(Stdio::null()).stderr(Stdio::inherit()).spawn()?;

        Ok(())
    }
    pub fn run(
        &self,
        exec_args: Option<Vec<String>>,
        name_mod: Option<&str>,
        additional_ports: Vec<(Port, Port)>,
    ) -> DockerResult<()> {
        let mut cmd = Command::new(DOCKER_BIN);
        cmd.args(&["run", "--rm", "-d", "-eCKB_CHAIN:dev"]);
        let mut container_name = None;

        if let Some(name) = self.name.as_ref() {
            if let Some(name_mod) = name_mod {
                let display_name = format!("{}-{}", &name, name_mod);
                container_name = Some(display_name.clone());
                cmd.arg("--name").arg(container_name.as_ref().unwrap());
            } else {
                container_name = Some(name.clone());
                cmd.arg("--name").arg(name);
            }
        }
        self.port_bindings.iter().for_each(|bind| {
            cmd.arg(format!("-p{}:{}", bind.0, bind.1).as_str());
        });
        additional_ports.iter().for_each(|bind| {
            cmd.arg(format!("-p{}:{}", bind.0, bind.1).as_str());
        });
        self.volumes.iter().for_each(|bind| {
            cmd.arg(format!("-v{}:{}", bind.0, bind.1).as_str());
        });

        cmd.arg(IMAGE_NAME);
        if exec_args.is_some() {
            cmd.args(exec_args.unwrap().as_slice());
        }

        let _child = cmd.stdout(Stdio::null()).stderr(Stdio::null()).spawn()?;

        println!("Successfully started network!\n");
        if container_name.is_some() {
            println!("Running in container: {}", container_name.unwrap());
        }
        self.virtuals.iter().for_each(|e| println!("{}", e));

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn image() -> DockerImage {
        DockerImage {
            name: "trampoline".to_string().to_string(),
            tag: Some("latest".to_string().to_string()),
            file_path: Some("./docker".to_string().to_string()),
            host_mappings: vec![],
            build_args: HashMap::new(),
        }
    }

    fn image_2() -> DockerImage {
        DockerImage {
            name: "trampoline".to_string().to_string(),
            tag: None,
            file_path: None,
            host_mappings: vec![],
            build_args: HashMap::new(),
        }
    }

    #[test]
    fn test_build_format_command() {
        let image = image();
        let command = DockerCommand::default().build(&image, true).unwrap();
        assert_eq!(
            command.command_string.as_ref().unwrap().as_str(),
            "docker image build --rm ./docker -t trampoline:latest"
        );
    }

    #[test]
    fn test_rm_format_command() {
        let image = image_2();
        let command = DockerCommand::default().remove(&image).unwrap();
        assert_eq!(
            command.command_string.as_ref().unwrap().as_str(),
            "docker image rm trampoline"
        );
    }
}
