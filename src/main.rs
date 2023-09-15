use env_logger::Env;
use log::{error, info, debug};
use serde::{Serialize, Deserialize};
use std::{process::{Command, self}, io::{Error, BufReader, ErrorKind}, fs::File};

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

    debug!("Running {}", config_command.name);
    let result_output = command.arg("-c").arg(&config_command.command).output().expect("Failed run command");
    if result_output.status.success() {
        return Ok(format!("Success {}", config_command.name));
    } else {
        debug!("{:?}", String::from_utf8(result_output.stderr).expect("Failed parse result_output"));
        return Err(Error::new(ErrorKind::Interrupted, "failed run command"));
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

    let mut count_success:u8 = 0;
    let mut count_failed:u8 = 0;
    
    for config_command in config.commands {
        match run_command(&config_command) {
            Ok(res) => {
                count_success = count_success + 1;
                info!("{}", res);
            }
            Err(error) => {
                count_failed = count_failed + 1;
                error!("{:?}", error);
            }
        }
    }

    info!("Success total {}", count_success);
    info!("Failed total {}", count_failed);
}
