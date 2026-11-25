use std::{env, process};

pub struct LayerAction {
    pub target: String,
    pub method: String,
}

pub struct WindowAction {
    pub title: String,
    pub target: String,
    pub method: String,
}

pub fn usage(exit_code: i32) -> ! {
    eprintln!("Usage: killactive [-f] [-h] [-l target:method] [-w title:target:method]");
    eprintln!(
        "-f skips checking if quickshell is open -> if anything quickshell exists it tries to close it."
    );
    process::exit(exit_code);
}

pub fn parse_args() -> (Vec<LayerAction>, Vec<WindowAction>, bool) {
    let (mut layers, mut windows) = (Vec::new(), Vec::new());
    let mut faster = false;
    let mut args = env::args().skip(1);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-f" | "--faster" => faster = true,
            "-h" | "--help" => usage(0),
            "-l" | "--layer" => {
                let Some(raw_value) = args.next() else {
                    usage(1);
                };

                let Some((target, method)) = raw_value.split_once(':') else {
                    usage(1);
                };

                if target.is_empty() || method.is_empty() {
                    usage(1);
                }

                layers.push(LayerAction {
                    target: target.to_owned(),
                    method: method.to_owned(),
                });
            }
            "-w" | "--window" => {
                let Some(raw_value) = args.next() else {
                    usage(1);
                };

                let Some((reuse, method)) = raw_value.rsplit_once(':') else {
                    usage(1);
                };

                let Some((title, target)) = reuse.rsplit_once(':') else {
                    usage(1);
                };

                if title.is_empty() || target.is_empty() || method.is_empty() {
                    usage(1);
                }

                windows.push(WindowAction {
                    title: title.to_owned(),
                    target: target.to_owned(),
                    method: method.to_owned(),
                });
            }
            _ => usage(1),
        }
    }

    (layers, windows, faster)
}
