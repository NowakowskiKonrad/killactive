use serde::Deserialize;
use std::{
    borrow::Cow,
    collections::HashMap,
    io::{Error, Read, Write},
    os::unix::net::UnixStream,
    path::Path,
    time::Duration,
};

#[derive(Deserialize)]
pub struct MonitorLayers<'a> {
    #[serde(borrow)]
    pub levels: HashMap<&'a str, Vec<LayerSurface<'a>>>,
}

#[derive(Deserialize)]
pub struct LayerSurface<'a> {
    pub namespace: &'a str,
    pub pid: i32,
}

#[derive(serde::Deserialize)]
pub struct ActiveWindow<'a> {
    pub title: Option<Cow<'a, str>>,
    pub pid: Option<i32>,
}

pub fn find_qs_pids(json: &str) -> serde_json::Result<Vec<i32>> {
    let monitors: HashMap<&str, MonitorLayers> = serde_json::from_str(json)?;

    Ok(monitors
        .values()
        .flat_map(|mon| mon.levels.values())
        .flatten()
        .filter(|layer| layer.namespace.contains("quickshell"))
        .map(|layer| layer.pid)
        .collect())
}

/// Modifies an existing buffer, adding a `QString` in IPC format (4-byte BE size + UTF-16BE).
///
/// # How it works
/// Extends the buffer by 4 placeholder bytes to reserve space for the length,
/// encodes the string into big-endian UTF-16 directly onto the buffer, and then
/// backfills those first 4 bytes once the final byte size is known.
#[allow(clippy::cast_possible_truncation)]
pub fn write_qstring(buf: &mut Vec<u8>, text: &str) {
    let len_at = buf.len();
    buf.extend_from_slice(&[0; 4]); // length placeholder
    buf.extend(text.encode_utf16().flat_map(u16::to_be_bytes));
    let len = (buf.len() - len_at - 4) as u32;
    buf[len_at..len_at + 4].copy_from_slice(&len.to_be_bytes());
}

/// Encodes a zero-argument Quickshell IPC call into a binary packet.
///
/// Packet layout:
/// - `1` byte: Call opcode (`0x03`)
/// - `N` bytes: Target name encoded as a `QString`
/// - `M` bytes: Method name encoded as a `QString`
/// - `4` bytes: Argument count (`0` as big-endian `u32`)
///
/// # How it works
/// Pre-calculates the worst-case byte size upfront (`1 + 4 + target*2 + 4 + method*2 + 4`)
/// so the `Vec` allocates once and never resizes while writing. It pushes the `0x03`
/// header (qs::io::ipc::comm::StringCallCommand), serializes the target and method strings
///  via [`write_qstring`], and finishes with four zero bytes to signal an empty argument list.
pub fn encode_qs_call(target: &str, method: &str) -> Vec<u8> {
    let cap = 1 + 4 + (target.len() * 2) + 4 + (method.len() * 2) + 4;
    let mut vector = Vec::with_capacity(cap);
    vector.extend_from_slice(&[0x03]);
    write_qstring(&mut vector, target);
    write_qstring(&mut vector, method);
    vector.extend_from_slice(&0u32.to_be_bytes());

    vector
}

const QS_TRUE_REPLY: [u8; 14] = [
    0x05, 0x00, 0x00, 0x00, 0x00, 0x08, 0x00, b't', 0x00, b'r', 0x00, b'u', 0x00, b'e',
];

pub fn qs_is_open(sock_path: &Path, target: &str) -> bool {
    let Ok(mut stream) = UnixStream::connect(sock_path) else {
        return false;
    };

    let _ = stream.set_read_timeout(Some(Duration::from_millis(50)));
    if stream.write_all(&encode_qs_call(target, "isOpen")).is_err() {
        return false;
    }

    let mut buf = [0u8; 32];
    match stream.read(&mut buf) {
        Ok(n) => buf[..n] == QS_TRUE_REPLY,
        Err(_) => false,
    }
}

pub fn hypr_cmd(sock_path: &Path, cmd: &str) -> Result<String, Error> {
    let mut stream = UnixStream::connect(sock_path)?;

    stream.write_all(cmd.as_bytes())?;
    stream.set_read_timeout(Some(Duration::from_millis(100)))?;
    let mut resp = String::new();
    stream.read_to_string(&mut resp)?;

    Ok(resp)
}

pub fn qs_cmd(sock_path: &Path, target: &str, method: &str) {
    let Ok(mut stream) = UnixStream::connect(sock_path) else {
        return;
    };

    let _ = stream.write_all(&encode_qs_call(target, method));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn call(target: &str, method: &str) -> Vec<u8> {
        encode_qs_call(target, method)
    }

    #[test]
    fn encodes_sidebar_close() {
        assert_eq!(
            call("sidebar", "close"),
            vec![
                0x03, // QString "sidebar": 7 UTF-16 code units = 14 bytes
                0x00, 0x00, 0x00, 0x0e, 0x00, b's', 0x00, b'i', 0x00, b'd', 0x00, b'e', 0x00, b'b',
                0x00, b'a', 0x00, b'r',
                // QString "close": 5 UTF-16 code units = 10 bytes
                0x00, 0x00, 0x00, 0x0a, 0x00, b'c', 0x00, b'l', 0x00, b'o', 0x00, b's', 0x00, b'e',
                // argc = 0
                0x00, 0x00, 0x00, 0x00,
            ]
        );
    }

    #[test]
    fn encodes_sidebar_is_open() {
        assert_eq!(
            call("sidebar", "isOpen"),
            vec![
                0x03, // QString "sidebar"
                0x00, 0x00, 0x00, 0x0e, 0x00, b's', 0x00, b'i', 0x00, b'd', 0x00, b'e', 0x00, b'b',
                0x00, b'a', 0x00, b'r',
                // QString "isOpen": 6 UTF-16 code units = 12 bytes
                0x00, 0x00, 0x00, 0x0c, 0x00, b'i', 0x00, b's', 0x00, b'O', 0x00, b'p', 0x00, b'e',
                0x00, b'n', // argc = 0
                0x00, 0x00, 0x00, 0x00,
            ]
        );
    }

    #[test]
    fn qstring_length_is_utf16_byte_length_not_char_count() {
        let mut buf = Vec::new();

        // 'é' is one UTF-16 code unit: 0x00E9.
        // '🙂' is two UTF-16 code units: 0xD83D 0xDE42.
        // Total UTF-16BE byte length: 2 + 4 = 6.
        write_qstring(&mut buf, "é🙂");

        assert_eq!(&buf[0..4], &6u32.to_be_bytes());
        assert_eq!(&buf[4..], &[0x00, 0xe9, 0xd8, 0x3d, 0xde, 0x42,]);
    }

    #[test]
    fn qstring_empty_has_zero_length() {
        let mut buf = Vec::new();
        write_qstring(&mut buf, "");
        assert_eq!(buf, vec![0x00, 0x00, 0x00, 0x00]);
    }

    #[test]
    fn finds_only_quickshell_layer_pids() {
        let json = r#"{
            "DP-1": {
                "levels": {
                    "0": [
                        { "namespace": "quickshell", "pid": 10 },
                        { "namespace": "waybar", "pid": 20 }
                    ],
                    "2": [
                        { "namespace": "quickshell", "pid": 30 }
                    ]
                }
            },
            "HDMI-A-1": {
                "levels": {
                    "0": [
                        { "namespace": "qs", "pid": 40 }
                    ]
                }
            }
        }"#;

        let mut pids = find_qs_pids(json).unwrap();
        pids.sort_unstable();

        assert_eq!(pids, vec![10, 30]);
    }

    #[test]
    fn active_window_missing_title_is_none() {
        let window: ActiveWindow<'_> = serde_json::from_str("{}").unwrap();
        assert!(window.title.is_none());
        assert!(window.pid.is_none());
    }

    #[test]
    fn active_window_empty_title_is_preserved() {
        let window: ActiveWindow<'_> =
            serde_json::from_str(r#"{ "title": "", "pid": 123 }"#).unwrap();

        assert_eq!(window.title, Some(Cow::Borrowed("")));
        assert_eq!(window.pid, Some(123));
    }

    #[test]
    fn active_window_title_and_pid_are_deserialized() {
        let window: ActiveWindow<'_> =
            serde_json::from_str(r#"{ "title": "Firefox", "pid": 123 }"#).unwrap();

        assert_eq!(window.title, Some(Cow::Borrowed("Firefox")));
        assert_eq!(window.pid, Some(123));
    }
}
