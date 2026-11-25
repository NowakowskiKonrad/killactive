mod cli;
mod ipc;

use cli::parse_args;
use core::error;
use std::path::PathBuf;

use crate::ipc::*;

fn main() -> Result<(), Box<dyn error::Error>> {
    let (layers, windows, faster) = parse_args();
    let runtime = std::env::var("XDG_RUNTIME_DIR")?;
    let hyprland_signature = std::env::var("HYPRLAND_INSTANCE_SIGNATURE")?;
    let hyprland_ipc = PathBuf::from(format!("{runtime}/hypr/{hyprland_signature}/.socket.sock"));

    if !layers.is_empty() {
        let binding = hypr_cmd(&hyprland_ipc, "j/layers")?;
        if let Ok(mut pids) = find_qs_pids(&binding)
            && !pids.is_empty()
        {
            pids.sort_unstable();
            pids.dedup();
            if !faster {
                for pid in pids {
                    let qs_ipc =
                        PathBuf::from(format!("{runtime}/quickshell/by-pid/{pid}/ipc.sock"));
                    for entity in &layers {
                        if qs_is_open(&qs_ipc, &entity.target) {
                            qs_cmd(&qs_ipc, &entity.target, &entity.method);
                            return Ok(());
                        }
                    }
                }
            } else {
                for pid in pids {
                    let qs_ipc =
                        PathBuf::from(format!("{runtime}/quickshell/by-pid/{pid}/ipc.sock"));
                    for entity in &layers {
                        qs_cmd(&qs_ipc, &entity.target, &entity.method);
                    }
                }

                return Ok(());
            }
        }
    }

    let binding = hypr_cmd(&hyprland_ipc, "j/activewindow")?;
    let resp: ActiveWindow = serde_json::from_str(&binding)?;

    if let Some(title) = resp.title {
        if title.is_empty() || title == "Empty" {
            return Ok(());
        }
        if let Some(entity) = windows.iter().find(|w| w.title == title) {
            if let Some(pid) = resp.pid {
                let qs_ipc = PathBuf::from(format!("{runtime}/quickshell/by-pid/{pid}/ipc.sock"));
                qs_cmd(&qs_ipc, &entity.target, &entity.method);
            }
        } else {
            hypr_cmd(&hyprland_ipc, "dispatch hl.dsp.window.close()")?;
        }
    }

    Ok(())
}
