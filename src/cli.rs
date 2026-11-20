use std::{env, process};

pub struct LayerEntity {
    pub target: String,
    pub method: String,
}

pub struct WindowEntity {
    pub title: String,
    pub target: String,
    pub method: String,
}

#[inline]
pub fn usage() -> ! {
    eprintln!("Usage: ml4w-killactive [-l target:method] [-w title:target:method]");
    process::exit(1);
}

#[inline]
pub fn parse_args() -> (Vec<LayerEntity>, Vec<WindowEntity>) {
    let (mut layers, mut windows) = (Vec::new(), Vec::new());
    let mut args = env::args().skip(1);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-l" | "--layer" => {
                let Some(raw_value) = args.next() else {
                    usage();
                };

                let Some((target, method)) = raw_value.split_once(':') else {
                    usage();
                };

                if target.is_empty() || method.is_empty() {
                    usage();
                }

                layers.push(LayerEntity {
                    target: target.to_owned(),
                    method: method.to_owned(),
                });
            }
            "-w" | "--window" => {
                let Some(raw_value) = args.next() else {
                    usage();
                };

                let Some((reuse, method)) = raw_value.rsplit_once(':') else {
                    usage();
                };

                let Some((title, target)) = reuse.rsplit_once(':') else {
                    usage();
                };

                if title.is_empty() || target.is_empty() || method.is_empty() {
                    usage();
                }

                windows.push(WindowEntity {
                    title: title.to_owned(),
                    target: target.to_owned(),
                    method: method.to_owned(),
                });
            }
            _ => usage(),
        }
    }

    (layers, windows)
}
