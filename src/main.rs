use std::env;
use std::fs::File;
use std::io::{Error, Read, Write};
use std::process::Command;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering::{Acquire, SeqCst};
use std::sync::Arc;

fn main() {
    let base_start: Arc<AtomicU64> = Arc::new(AtomicU64::new(1000000000000000));

    handle_args(env::args().collect(), base_start.clone());

    let cd_num = base_start.clone();
    ctrlc::set_handler(move || {
        println!("Received Ctrl-C.");
        print!("Saving current code {} ...", cd_num.load(Acquire));
        match saver(cd_num.clone()) {
            Ok(_) => println!(" successful."),
            Err(e) => println!(" failed: {:?}", e),
        }

        println!("Exiting.");
        std::process::exit(0)
    })
    .expect("Error setting Ctrl-C handler.");

    let mut status = false;

    while !status {
        let code = base_start.load(Acquire).to_string();
        let output = Command::new("/usr/bin/fastboot")
            .args(&["oem", "unlock"])
            .arg(&code)
            .output()
            .expect("failed to execute");

        println!("code: {}, {}", code, output.status);
        status = output.status.success();

        base_start.fetch_add(1, Acquire);
    }

    println!(
        "Just received a success exit code for code: {}",
        base_start.load(Acquire)
    );
}

fn handle_args(args: Vec<String>, base_start: Arc<AtomicU64>) {
    if args.len() > 1 {
        match args[1].trim_end().parse::<u64>() {
            Ok(prev_start) => base_start.store(prev_start, SeqCst),
            Err(_) => help_text(),
        }
        if base_start.load(Acquire).to_string().len() != 16 {
            println!("Error. Code is not exactly 16 digits long.\n");
            help_text();
        }
        println!("Loaded offset {:?} successfully. Resuming.", base_start);
    } else {
        if resumer(base_start.clone()).is_ok() {
            println!(
                "Loaded offset {:?} successfully from 'lastcode'. Resuming.",
                base_start
            );
        }
    }
}

fn resumer(base_start: Arc<AtomicU64>) -> std::io::Result<()> {
    let mut f = File::open("lastcode")?;
    let mut buffer = String::new();

    f.read_to_string(&mut buffer)?;
    base_start.store(
        buffer
            .trim_end()
            .parse::<u64>()
            .map_err(|e| Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?,
        SeqCst,
    );

    Ok(())
}

fn saver(base_start: Arc<AtomicU64>) -> std::io::Result<()> {
    let mut f = File::create("lastcode")?;
    f.write_all(base_start.load(Acquire).to_string().as_bytes())?;

    Ok(())
}

fn help_text() {
    println!("This tool is supposed to unlock a Huawei MediaPad M3.");
    println!("Run it with exactly 1 argument to start with an offset other than 1000000000000000.");
    println!("Else it tries to load the previously used offset from 'lastcode' in $PWD.");
    println!("");
    println!("Fuck Huawei for leaving us out in the rain.");

    std::process::exit(0);
}
