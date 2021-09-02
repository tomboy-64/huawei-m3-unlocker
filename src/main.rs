use rand::prelude::*;
use std::char::from_digit;
use std::env;
use std::fs::File;
use std::io::{Error, ErrorKind, Read, Write};
use std::process::Command;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering::{Acquire, SeqCst};
use std::sync::Arc;
use std::time::Instant;

fn main() {
    let total_time = Instant::now();
    let total_time_ctrlc = Instant::now();

    let base_start: Arc<AtomicU64> = Arc::new(AtomicU64::new(1000000000000000));
    let mut randomizer = Vec::new();

    handle_args(env::args().collect(), base_start.clone(), &mut randomizer);

    let cd_num = base_start.clone();
    let cd_rnd = randomizer.clone();
    ctrlc::set_handler(move || {
        println!("Received Ctrl-C.");
        print!(
            "Saving current code {} ({}) ...",
            cd_num.load(Acquire),
            cd_rnd
                .iter()
                .map(|d| from_digit(*d as u32, 16).unwrap())
                .collect::<String>()
        );
        match saver(cd_num.load(Acquire), &cd_rnd) {
            Ok(_) => println!(" successful."),
            Err(e) => println!(" failed: {:?}", e),
        }

        println!("Exiting.");
        print_total_time(total_time_ctrlc);
        std::process::exit(0)
    })
    .expect("Error setting Ctrl-C handler.");

    let mut stdout: Option<[String; 2]> = None;
    let mut before = Instant::now();

    // main loop
    loop {
        let mut code = vec!['0'; 16];
        for (c, t) in base_start
            .load(Acquire)
            .to_string()
            .chars()
            .zip(randomizer.iter())
        {
            code[*t as usize] = c;
        }
        let code = code.iter().collect::<String>();

        let output = Command::new("/usr/bin/fastboot")
            .args(&["oem", "unlock"])
            .arg(&code)
            .output()
            .expect("failed to execute");

        let o_s = [output.stdout, output.stderr]
            .iter()
            .map(|v| {
                v.iter()
                    .map(|b| *b as char)
                    .collect::<String>()
                    .trim()
                    .to_string()
            })
            .collect::<Vec<String>>();

        let new_instant = Instant::now();
        println!(
            "code: {} ({}), {}, stdout: {}, stderr: {}, elapsed: {}ms",
            code,
            base_start.load(Acquire),
            output.status,
            o_s[0],
            o_s[1],
            new_instant.duration_since(before).as_millis()
        );
        before = new_instant;

        // if output status is success: break
        if output.status.success() {
            println!("Received success exit code. Halting.");
            break;
        }
        // if output strings change: break <- pretty hacky. but in case?
        if let Some(previous) = stdout {
            if &previous != &o_s[..2] {
                println!("Output string changed! Halting.");
                break;
            }
        }

        stdout = Some([o_s[0].clone(), o_s[1].clone()]);

        base_start.fetch_add(1, Acquire);
    }

    print!("Storing last used code {} ... ", base_start.load(Acquire));
    match saver(base_start.load(Acquire), &randomizer) {
        Ok(_) => println!("success."),
        Err(e) => println!("failed: {}", e),
    }
    print_total_time(total_time);
    println!("Current code: {}", base_start.load(Acquire));
}

fn handle_args(args: Vec<String>, base_start: Arc<AtomicU64>, randomizer: &mut Vec<u8>) {
    if args.len() > 1 {
        // check first argument / code to start with
        match args[1].trim_end().parse::<u64>() {
            Ok(prev_start) => base_start.store(prev_start, SeqCst),
            Err(_) => help_text(),
        }
        // check whether our code is exactly 16 digits long
        if base_start.load(Acquire).to_string().len() != 16 {
            println!("Error. Code is not exactly 16 digits long.\n");
            help_text();
        }

        // check second argument / randomizing string
        if args[2].chars().any(|c| !c.is_ascii_hexdigit()) {
            println!(
                "Error. Randomizer does not consist exclusively of hex digits: {}",
                args[2]
            );
            help_text();
        }
        randomizer.extend(
            args[2]
                .chars()
                .map(|c| u8::from_str_radix(&c.to_string(), 16).unwrap()),
        );
        if check_randomizer(&randomizer).is_err() {
            help_text();
        }
        println!(
            "Loaded offset {:?}, randomizer {} successfully. Resuming.",
            args[1], args[2]
        );
    } else {
        if resumer(base_start.clone(), randomizer).is_ok() {
            println!(
                "Loaded offset {:?}, randomizer {} successfully from 'lastcode'. Resuming.",
                base_start,
                randomizer
                    .iter()
                    .map(|b| from_digit(*b as u32, 16).unwrap())
                    .collect::<String>()
            );
        } else {
            base_start.store(1_000_000_000_000_000, SeqCst);
            *randomizer = (1..=0xf).collect::<Vec<_>>();
            // need to randomize
            let mut rng = rand::thread_rng();
            randomizer.shuffle(&mut rng);
            randomizer.insert(0, 0);

            if check_randomizer(&randomizer).is_err() {
                panic!("randomizer initialization is faulty!");
            }
        }
    }

    if check_randomizer(&randomizer).is_err() {
        panic!("randomizer is faulty!");
    }
}

fn check_randomizer(randomizer: &Vec<u8>) -> std::io::Result<()> {
    let mut rand_test = randomizer.clone();
    rand_test.sort_unstable();

    if rand_test.len() != 16 || // not 16 digits
        (0..=0xf).zip(rand_test.iter()).any(|(a,b)| a != *b) || // not all digits present
        (0..=0xf).zip(randomizer.iter()).all(|(a,b)| a == *b)
    // not rnd
    {
        Err(Error::new(ErrorKind::InvalidData, "randomizer is invalid"))
    } else {
        Ok(())
    }
}

fn resumer(base_start: Arc<AtomicU64>, randomizer: &mut Vec<u8>) -> std::io::Result<()> {
    let mut f = File::open("lastcode")?;
    let mut buffer = String::new();

    f.read_to_string(&mut buffer)?;
    base_start.store(
        buffer
            .chars()
            .take(16)
            .collect::<String>()
            .parse::<u64>()
            .map_err(|e| Error::new(ErrorKind::InvalidData, e.to_string()))?,
        SeqCst,
    );

    if buffer
        .chars()
        .skip(16)
        .take(16)
        .any(|c| !c.is_ascii_hexdigit())
    {
        return Err(Error::new(ErrorKind::InvalidData, "not all hexdigits"));
    }
    *randomizer = buffer
        .chars()
        .skip(16)
        .take(16)
        .map(|c| u8::from_str_radix(&c.to_string(), 16).unwrap())
        .collect::<Vec<_>>();
    check_randomizer(&randomizer)?;

    Ok(())
}

fn saver(base_start: u64, randomizer: &Vec<u8>) -> std::io::Result<()> {
    let mut f = File::create("lastcode")?;
    let mut savestring = base_start.to_string();
    while savestring.len() != 16 {
        savestring.insert(0, '0');
    }
    savestring.extend(
        randomizer
            .iter()
            .map(|d| from_digit(*d as u32, 16).unwrap()),
    );
    f.write_all(savestring.as_bytes())?;

    Ok(())
}

fn help_text() {
    println!("This tool is supposed to unlock a Huawei MediaPad M3.");
    println!("Run it with exactly 1 argument to start with an offset other than 1000000000000000.");
    println!("Else it tries to load the previously used offset from 'lastcode' in $PWD.");

    std::process::exit(0);
}

fn print_total_time(start: Instant) {
    let times = [60u64, 60, 24, 7, 1]
        .iter()
        .scan(
            Instant::now().duration_since(start).as_secs(),
            |state, d| {
                let r = *state % *d;
                *state /= *d;
                Some(r)
            },
        )
        .collect::<Vec<_>>();

    let date = {
        match Command::new("date").output() {
            Ok(d) => {
                let mut s = "at ".to_string();
                s.push_str(
                    &d.stdout
                        .iter()
                        .map(|b| *b as char)
                        .collect::<String>()
                        .trim()
                        .to_string(),
                );
                s
            }
            // okay, so we don't have `date` available.
            // let's be snarky about that.
            Err(_) => "a while back.".to_string(),
        }
    };

    println!(
        "I've been running for {}w {}d {}h {}m {}s. This program finished {}.",
        times[4], times[3], times[2], times[1], times[0], date
    );
}
