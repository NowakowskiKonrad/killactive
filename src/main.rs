use std::{
    fs::read_dir,
    io::{Read, Write},
    os::unix::net::UnixStream,
    path::PathBuf,
    vec,
};

#[derive(serde::Deserialize)]
struct ActiveWindow {
    title: Option<String>,
}

fn write_qstring(buf: &mut Vec<u8>, text: &str) {
    let text: Vec<u16> = text.encode_utf16().collect();
    let bytes_len = (text.len() * 2) as u32;
    buf.extend_from_slice(&bytes_len.to_be_bytes());

    for el in text {
        for byte in el.to_be_bytes() {
            buf.extend_from_slice(&[byte]);
        }
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

fn read_qs_bool(stream: &mut UnixStream) -> bool {
    const TRUE_STR: &[u8] = &[
        0x05, 0x00, 0x00, 0x00, 0x00, 0x08, 0x00, b't', 0x00, b'r', 0x00, b'u', 0x00, b'e',
    ];
    const TRUE_BOOL: &[u8] = &[0x02, 0x02, 0x01, 0x01];

    let mut buf = [0u8; 64];
    let mut offset = 0;

    while offset < buf.len() {
        match stream.read(&mut buf[offset..]) {
            Ok(0) => break,
            Ok(n) => offset += n,
            Err(_) => return false,
        }
    }
    println!("{:?}", buf);

    matches!(&buf[..offset], TRUE_STR | TRUE_BOOL)
}

fn hypr_cmd(sock_path: &PathBuf, cmd: &str) -> String {
    let mut stream = UnixStream::connect(sock_path).unwrap();

    stream.write_all(cmd.as_bytes()).unwrap();
    let mut resp = String::new();
    stream.read_to_string(&mut resp).unwrap();

    return resp;
}

fn qs_cmd(sock_path: &PathBuf, target: &str, method: &str) -> Option<UnixStream> {
    let Ok(mut stream) = UnixStream::connect(sock_path) else {
        return None;
    };
    stream
        .set_read_timeout(Some(std::time::Duration::from_millis(50)))
        .unwrap();
    stream.write_all(&encode_qs_call(target, method)).unwrap();

    return Some(stream);
}

fn main() {
    let runtime = std::env::var("XDG_RUNTIME_DIR").unwrap();
    let hyprland_signature = std::env::var("HYPRLAND_INSTANCE_SIGNATURE").unwrap();

    let qs_ipcs = PathBuf::from(runtime.clone()).join("quickshell/by-id");
    let qs_ipcs: Vec<PathBuf> = read_dir(qs_ipcs)
        .unwrap()
        .map(|p| p.unwrap().path().join("ipc.sock"))
        .collect();
    for sock_path in &qs_ipcs {
        for target in ["sidebar", "wallpaper", "calendar"] {
            let mut stream = qs_cmd(&sock_path, target, "isOpen").unwrap();

            if read_qs_bool(&mut stream) {
                qs_cmd(&sock_path, target, "close");
                return;
            }
        }
    }

    let path = PathBuf::from(format!("{runtime}/hypr/{hyprland_signature}/.socket.sock"));

    let resp: ActiveWindow = serde_json::from_str(&hypr_cmd(&path, "j/activewindow")).unwrap();
    if let Some(title) = resp.title {
        if title.is_empty() || title == "Empty" {
            return;
        }

        match title.as_str() {
            "ML4W Welcome" => {
                for sock_path in &qs_ipcs {
                    qs_cmd(&sock_path, "welcome", "toggle");
                }
            }
            "ML4W Dotfiles Settings" => {
                for sock_path in &qs_ipcs {
                    qs_cmd(&sock_path, "settings", "toggle");
                }
            }
            _ => {
                hypr_cmd(&path, "dispatch hl.dsp.window.close()");
                return;
            }
        }
    }
}
