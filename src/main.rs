mod thread_pool;

use env_logger::Env;
use log::{error, debug, info};
use serde::{Serialize, Deserialize};
use std::{process::{Command, self}, io::{Error, BufReader, ErrorKind}, fs::File, sync::{Mutex, Arc}};

use crate::thread_pool::ThreadPool;

#[derive(Serialize, Deserialize)]
struct Config {
    commands: Vec<ConfigCommand>
}

#[derive(Serialize, Deserialize)]
struct ConfigCommand {
    name: String,
    command: String,
}

fn run_command(config_command: &ConfigCommand) -> Result<String, Error>{
    let mut command = Command::new("sh");

    debug!("-> 🔁 Running {}", config_command.name);
    let result_output = command.arg("-c").arg(&config_command.command).output().expect("Failed run command");
    if result_output.status.success() {
        return Ok(format!("Success {}", config_command.name));
    } else {
        debug!("{:?}", String::from_utf8(result_output.stderr).expect("Failed parse result_output"));
        return Err(Error::new(ErrorKind::Interrupted, format!("failed run command {}", config_command.name)));
    }
}

fn read_file_config(filename: &String) -> Result<Config, Error>{
    let file = match File::open(filename) {
        Ok(file_open) => file_open,
        Err(error) => {
            error!("{:?}", error);
            error!("Failed get file {}", filename);
            return Err(error);
        }
    };

    let result = serde_json::from_reader(BufReader::new(&file));
    return match result {
        Ok(res) => Ok(res),
        Err(error) => {
            error!("{:?}", error);
            error!("Failed parse json");
            let error_model = Error::new(std::io::ErrorKind::InvalidData, "Failed parse json");
            return Err(error_model);
        }
    }
}

fn main() {
    let env = Env::default();
    env_logger::init_from_env(env);
    let filename = String::from("config.json");

    let config = match read_file_config(&filename) {
        Ok(data) => data,
        Err(_) => {
            process::exit(0)
        }
    };

    let mut pool = ThreadPool::new(10);
    let success_count = Arc::new(Mutex::new(0));
    let total_length = config.commands.len();
    for command in config.commands {
        let success_count = Arc::clone(&success_count);
        pool.execute(move || {
            match run_command(&command) {
                Ok(res) => {
                    info!("<- 🟢 {}", res);
                    println!("<- 🟢 {} Success", command.name);
                    let mut count = success_count.lock().unwrap();
                    *count += 1;
                },
                Err(error) => {
                    info!("<- 🔴 [{}] {:?}", command.name, error);
                    println!("<- 🔴 {} Failed", command.name);
                }
            }
        })
    }

    pool.drop();

    let total_success = *success_count.lock().unwrap();
    let total_failed = total_length - total_success;
    println!("🏠🟢 Done. Success {}",total_success);
    println!("🏠🔴 Done. Failed {}",total_failed);

    
}
