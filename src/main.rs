use std::{process, sync::mpsc, thread, time::Duration};

use anyhow::Result;

use replit_xray::server::Server;

use signal_hook::{consts::SIGINT, iterator::Signals};

fn main() -> Result<()> {
    let mut app = Server::new();

    app.run()?;

    let mut signals = Signals::new(&[SIGINT])?;

    let (tx, rx) = mpsc::channel::<i32>();
    let tx1 = tx.clone();

    thread::spawn(move || {
        for sig in signals.forever() {
            println!("Received signal {:?}", sig);
            tx.send(sig).unwrap();
        }
    });

    thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(3));
        tx1.send(1000).unwrap();
    });

    for received in rx {
        if received == SIGINT {
            app.stop();
            process::exit(0)
        } else if received == 1000 {
            app.keep();
        }
    }

    Ok(())
}
