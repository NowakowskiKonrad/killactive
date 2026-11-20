mod cli;

use cli::*;
use core::error;
use serde::Deserialize;
use std::{
    collections::HashMap,
    io::{Error, Read, Write},
    os::unix::net::UnixStream,
    path::{Path, PathBuf},
};

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
struct ActiveWindow<'a> {
    title: Option<&'a str>,
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
    let cap = 1 + 4 + (target.len() * 2) + 4 + (method.len() * 2) + 4;
    let mut vector = Vec::with_capacity(cap);
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

fn qs_cmd(sock_path: &Path, target: &str, method: &str) -> Result<(), Box<dyn error::Error>> {
    let Ok(mut stream) = UnixStream::connect(sock_path) else {
        return Ok(());
    };

    stream.write_all(&encode_qs_call(target, method))?;
    Ok(())
}

fn main() -> Result<(), Box<dyn error::Error>> {
    let (layers, windows) = parse_args();
    let runtime = std::env::var("XDG_RUNTIME_DIR")?;
    let hyprland_signature = std::env::var("HYPRLAND_INSTANCE_SIGNATURE")?;
    let hyprland_ipc = PathBuf::from(format!("{runtime}/hypr/{hyprland_signature}/.socket.sock"));

    if !layers.is_empty() {
        let binding = hypr_cmd(&hyprland_ipc, "j/layers")?;
        if let Ok(Some(pid)) = find_qs_pid(&binding) {
            let qs_ipc = PathBuf::from(format!("{}/quickshell/by-pid/{}/ipc.sock", runtime, pid));
            for entity in layers {
                qs_cmd(&qs_ipc, &entity.target, &entity.method)?;
            }

            return Ok(());
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
                let qs_ipc =
                    PathBuf::from(format!("{}/quickshell/by-pid/{}/ipc.sock", runtime, pid));
                let _ = qs_cmd(&qs_ipc, &entity.target, &entity.method);
            }
        } else {
            hypr_cmd(&hyprland_ipc, "dispatch hl.dsp.window.close()")?;
        }
    }

    Ok(())
}
