use std::{
    collections::HashMap,
    io::{Error, Read, Write},
    os::unix::net::UnixStream,
    path::{Path, PathBuf},
};

use serde::Deserialize;

#[derive(Deserialize)]
struct MonitorLayers<'a> {
    #[serde(borrow)]
    levels: HashMap<&'a str, Vec<LayerSurface<'a>>>,
}

#[derive(Deserialize)]
struct LayerSurface<'a> {
    namespace: &'a str,
    pid: i32,
}

#[derive(serde::Deserialize)]
#[allow(dead_code)]
struct ActiveWindow<'a> {
    title: Option<&'a str>,
    class: Option<&'a str>,
    pid: Option<i32>,
}

fn find_qs_pid(json: &str) -> serde_json::Result<Option<i32>> {
    let monitors: HashMap<&str, MonitorLayers> = serde_json::from_str(json)?;

    Ok(monitors
        .values()
        .flat_map(|mon| mon.levels.values())
        .flatten()
        .find(|layer| layer.namespace.contains("quickshell"))
        .map(|layer| layer.pid))
}

fn write_qstring(buf: &mut Vec<u8>, text: &str) {
    let text: Vec<u16> = text.encode_utf16().collect();
    let bytes_len = (text.len() * 2) as u32;
    buf.extend_from_slice(&bytes_len.to_be_bytes());

    for el in text {
        buf.extend_from_slice(&el.to_be_bytes());
    }
}

fn encode_qs_call(target: &str, method: &str) -> Vec<u8> {
    let mut vector = vec![];
    vector.extend_from_slice(&[0x03]);
    write_qstring(&mut vector, target);
    write_qstring(&mut vector, method);
    for i in 0u32.to_be_bytes() {
        vector.extend_from_slice(&[i]);
    }

    vector
}

fn hypr_cmd(sock_path: &Path, cmd: &str) -> Result<String, Error> {
    let mut stream = UnixStream::connect(sock_path)?;

    stream.write_all(cmd.as_bytes())?;
    let mut resp = String::new();
    stream.read_to_string(&mut resp)?;

    Ok(resp)
}

fn qs_cmd(sock_path: &Path, target: &str, method: &str) -> Option<()> {
    let Ok(mut stream) = UnixStream::connect(sock_path) else {
        return None;
    };

    stream.write_all(&encode_qs_call(target, method)).unwrap();
    Some(())
}

fn main() -> Result<(), Error> {
    let runtime = std::env::var("XDG_RUNTIME_DIR").unwrap();
    let hyprland_signature = std::env::var("HYPRLAND_INSTANCE_SIGNATURE").unwrap();
    let path = PathBuf::from(format!("{runtime}/hypr/{hyprland_signature}/.socket.sock"));

    let binding = hypr_cmd(&path, "j/layers")?;
    if let Ok(Some(resp)) = find_qs_pid(&binding) {
        let qs_ipc = PathBuf::from(runtime)
            .join("quickshell/by-pid")
            .join(resp.to_string())
            .join("ipc.sock");
        for target in ["sidebar", "wallpaper", "calendar"] {
            qs_cmd(&qs_ipc, target, "close").unwrap();
        }

        return Ok(());
    }

    let binding = hypr_cmd(&path, "j/activewindow")?;
    let resp: ActiveWindow = serde_json::from_str(&binding)?;

    if let Some(title) = resp.title {
        if title.is_empty() || title == "Empty" {
            return Ok(());
        }

        match title {
            "ML4W Welcome" => {
                let qs_ipc = PathBuf::from(runtime)
                    .join("quickshell/by-pid")
                    .join(resp.pid.unwrap().to_string())
                    .join("ipc.sock");
                qs_cmd(&qs_ipc, "welcome", "toggle").unwrap();
            }
            "ML4W Dotfiles Settings" => {
                let qs_ipc = PathBuf::from(runtime)
                    .join("quickshell/by-pid")
                    .join(resp.pid.unwrap().to_string())
                    .join("ipc.sock");
                qs_cmd(&qs_ipc, "settings", "toggle").unwrap();
            }
            _ => {
                hypr_cmd(&path, "dispatch hl.dsp.window.close()")?;
                return Ok(());
            }
        }
    }

    Ok(())
}
