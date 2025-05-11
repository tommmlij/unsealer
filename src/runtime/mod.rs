use tokio::process::Command;
use tokio_util::sync::CancellationToken;

use std::collections::HashMap;
use serde_json::Value;

fn json_to_uppercase_hashmap(json_str: Option<String>) -> HashMap<String, String> {
    let v: Value = serde_json::from_str(json_str.as_deref().unwrap()).unwrap();
    let obj = v.as_object().unwrap();

    obj.iter()
        .map(|(k, v)| (k.to_uppercase(), v.to_string().trim_matches('"').to_string()))
        .collect()
}

pub async fn run(config: Option<String>, command: String) -> anyhow::Result<()>  {

    let env_map = json_to_uppercase_hashmap(config);
    

    #[cfg(target_family = "windows")]
    const DETACH_FLAGS: u32 = 0x00000008 | 0x00000200;

    let cancel = CancellationToken::new();
    let cloned_cancel = cancel.clone();

    let task1_handle = tokio::spawn(async move {
        tokio::select! {
        _ = cloned_cancel.cancelled() => {
        }
        _ = async {
            // loop {
            let mut count = 0;
            while count < 2 {
                count += 1;

                #[cfg(target_family = "windows")]
                let mut child = Command::new("cmd")
                    .envs(env_map.clone())
                    .arg("/C")
                    .arg(command.clone())
                    .creation_flags(DETACH_FLAGS)
                    .spawn()?;

                #[cfg(unix)]
                let mut child = Command::new("sh")
                    .envs(env_map.clone())
                    .arg("-c")
                    .arg(command.clone())
                    .spawn()?;

                let status = child.wait().await?;
                if !status.success() {
                    eprintln!("Process B exited with status: {}", status);
                }

             }

            #[allow(unreachable_code)]
            Ok::<(), anyhow::Error>(())
        } => {},
        }
    });

    task1_handle.await?;

    Ok(())
}