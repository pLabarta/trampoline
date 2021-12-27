use crate::{
    TrampolineResourceType,
    TrampolineResource,
    project::VirtualEnv,
};
use std::marker::PhantomData;
use std::{process::Command};
use std::collections::HashMap;
use std::fmt::Formatter;
use std::io::Write;

use std::path::{PathBuf, Path};

use thiserror::Error;

use std::process::{Stdio};






pub const DOCKER_BIN: &str = "docker";
pub const IMAGE_NAME: &str = "iamm/trampoline-env:latest";
#[derive(Debug, Error)]
pub enum DockerError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Any(#[from] anyhow::Error),
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

pub struct DockerPort {
    pub host: usize,
    pub container: usize,
}


#[derive(Debug, Clone)]
pub struct DockerContainer<'a> {
    name: String,
    port_bindings: Vec<Port>,
    volumes: Vec<Volume<'a>>,
    env_vars: HashMap<String, String>,
    image: &'a DockerImage

}

#[derive(Debug, Clone)]
pub struct DockerImage {
    pub name: String,
    pub tag: Option<String>,
    pub file_path: Option<String>,
    pub host_mappings: Vec<Port>,
    pub build_args: HashMap<String, String>
}


pub struct DockerCommand<C> {
    _docker: PhantomData<C>,
}

impl DockerCommand<DockerImage> {
    pub fn build(image: &DockerImage, rm: bool, detach: bool) -> DockerResult<()> {
        let DockerImage {name,
            tag,
            file_path,
            host_mappings,
            build_args
        } = image;

        Ok(())
    }

    pub fn remove(image: &DockerImage) -> DockerResult<()> {
        Ok(())
    }

    pub fn prune() -> DockerResult<()> {
        Ok(())
    }
}

impl DockerCommand<DockerContainer<'_>> {
    pub fn run(container: &DockerContainer) -> DockerResult<()> {
        Ok(())
    }

    pub fn exec(container: &DockerContainer) -> DockerResult<()> {
        Ok(())
    }

    pub fn cp(container: &DockerContainer) -> DockerResult<()> {

    }

    pub fn start(container: &DockerContainer) -> DockerResult<()> {

    }

    pub fn stop(container: &DockerContainer) -> DockerResult<()> {

    }

    pub fn pause(container: &DockerContainer) -> DockerResult<()> {

    }

    pub fn unpause(container: &DockerContainer) -> DockerResult<()> {

    }

    pub fn restart(container: &DockerContainer) -> DockerResult<()> {

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
        let VirtualEnv {host: _, container_port, local_binding, container_mount, host_port} = env.clone();
        self.port_bindings.push((host_port.into(), container_port.into()));
        let local_binding = local_binding.canonicalize()?;
        self.volumes.insert(local_binding.into(), container_mount.into());
        self.virtuals.push(env.clone());

        Ok(self)
    }

    pub fn build(&self) -> DockerResult<()> {
        let mut cmd = Command::new(DOCKER_BIN);
        cmd.arg("build");
        cmd.arg(".");
        cmd.arg("-t");
        cmd.arg(IMAGE_NAME);
       let output =  cmd
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


    pub fn exec(container_name: &str,  exec_args: Vec<&str>, work_dir: &str) -> DockerResult<()> {
        let mut cmd = Command::new(DOCKER_BIN);
        cmd.args(&[
            "exec",
            "-d",
            container_name,
            "bash",
            "-c"
        ]);


       let args_string = exec_args.join(" ");
       let args_string = format!(r#"{}"#, args_string);
       println!("Args string: {}", args_string);
       cmd.arg(args_string.as_str());

       println!("{:?}", cmd);
       cmd
            .stdout(Stdio::null())
            .stderr(Stdio::inherit())
            .stdin(Stdio::null())
            .spawn()?;
      
        Ok(())
    }

    pub fn restart(&self) -> DockerResult<()> {
        let mut cmd = Command::new(DOCKER_BIN);
        cmd.args(&[
            "restart",
            self.name.as_ref().unwrap().as_str()
        ]);

        let _child =  cmd
            .stdout(Stdio::null())
            .stderr(Stdio::inherit())
            .spawn()?;

        Ok(())
    }
    pub fn run(&self, exec_args: Option<Vec<String>>, name_mod: Option<&str>, additional_ports: Vec<(Port, Port)>) -> DockerResult<()> {
        let mut cmd = Command::new(DOCKER_BIN);
        cmd.args(&[
            "run",
           "--rm",
            "-d",
            "-eCKB_CHAIN:dev"
        ]);
        let mut container_name = None;

        if let Some(name) = self.name.as_ref() {
            if let Some(name_mod) = name_mod {
                let display_name = format!("{}-{}",&name, name_mod);
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

      
       let _child =  cmd
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;

        
        println!("Successfully started network!\n");
        if container_name.is_some() {

            println!("Running in container: {}", container_name.unwrap());
        }
        self.virtuals.iter().for_each(|e| println!("{}", e));
     
        Ok(())

    }
}