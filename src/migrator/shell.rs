use anyhow::Result;
use serde_json::Value;
use std::process::Command;

#[derive(Clone)]
pub struct ShellConfig {
    pub host: String,
    pub port: usize,
}

impl Default for ShellConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 27017,
        }
    }
}

#[derive(Default, Clone)]
pub struct Shell {
    pub config: ShellConfig,
}

impl Shell {
    pub fn execute<S: AsRef<str> + std::fmt::Debug>(&self, db_name: S, query: S) -> Result<Value> {
        let out = Command::new("mongosh")
            .arg("--host")
            .arg(&self.config.host)
            .arg("--port")
            .arg(self.config.port.to_string())
            .arg("--eval")
            .arg(query.as_ref())
            .arg(db_name.as_ref())
            .stdout(std::process::Stdio::piped())
            .spawn()?
            .wait_with_output();

        let out = std::str::from_utf8(&out.expect("mongo shell finished").stdout)
            .expect("u8 to string")
            .to_string();
        let out_as_json = self.out_to_json(&out).unwrap_or(Value::String(out));

        Ok(out_as_json)
    }

    fn out_to_json(&self, shell_out: &String) -> Result<Value> {
        Ok(serde_json::from_str(&shell_out)?)
    }
}
