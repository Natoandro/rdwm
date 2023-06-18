use chrono::Local;
use log::info;
use simplelog::{Config, LevelFilter, WriteLogger};
use std::{
    ffi::{c_char, CStr},
    fs::{create_dir_all, File},
    io::Write,
};
use subprocess::Exec;

#[derive(Debug)]
struct CommandParams<'a> {
    command: &'a str,
    args: Vec<&'a str>,
}

impl<'a> CommandParams<'a> {
    unsafe fn new(args: *const *const c_char) -> Self {
        let command = CStr::from_ptr(*args);
        let mut i = 0;
        let argc = loop {
            let ptr = *args.offset(i + 1);
            if ptr.is_null() {
                break i;
            } else {
                i += 1;
            }
        };
        let args = std::slice::from_raw_parts(args.offset(1), argc as usize);
        let args = args
            .iter()
            .map(|&ptr| CStr::from_ptr(ptr).to_str().unwrap())
            .collect::<Vec<_>>();
        Self {
            command: command.to_str().unwrap(),
            args,
        }
    }
}

#[no_mangle]
pub extern "C" fn spawn_process(args: *const *const c_char) {
    let params = unsafe { CommandParams::new(args) };
    info!(
        "spawn process: {:?} {:?}",
        params.command,
        params.args.join(", ")
    );
    let _p = Exec::cmd(params.command)
        .args(&params.args)
        .detached()
        .popen()
        .unwrap();
}

#[no_mangle]

pub extern "C" fn init_lib() {
    println!("init lib");
    std::io::stdout().flush().unwrap();
    let dt = Local::now().format("%Y-%m-%d-%H%M%S").to_string();
    let data_dir = dirs::data_dir()
        .expect("could not find data dir")
        .join("rdwm");
    create_dir_all(&data_dir).expect("could not create data dir");
    let logfile =
        File::create(data_dir.join(format!("{}.log", dt))).expect("could not open log file");

    WriteLogger::init(LevelFilter::Info, Config::default(), logfile).expect("could not set logger");
    info!("initialized rust library");
    info!("env: PATH={:?}", std::env::var("PATH").unwrap());
}
