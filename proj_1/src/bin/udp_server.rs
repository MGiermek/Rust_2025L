use std::net::UdpSocket;
use clap::{ArgGroup, Parser};
use proj_1::models::{commands::AnyCommand, db_structure::{AnyDatabase, Database}};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None,
group(
        ArgGroup::new("mode")
            .required(true)    // must have one of them
            .args(["int", "string"]),
    ))]
struct Args {
    /// Use database with integer key
    #[arg(short, long = "Int", default_value_t=false)]
    int: bool,
    /// Use database with string key
    #[arg(short, long = "String", default_value_t=false)]
    string: bool,
}
fn main() {
    let args = Args::parse();
    let mut context_db: AnyDatabase;
    if args.int {
        println!("Using integer key database");
        context_db = AnyDatabase::IntDatabase(Database::<i64>::new());
    } else if args.string {
        println!("Using string key database");
        context_db = AnyDatabase::StringDatabase(Database::<String>::new());
    } else {
        println!("No database type specified");
        return;
    }

    let Ok(socket) = UdpSocket::bind("0.0.0.0:8888") else {
        println!("Failed to bind to socket");
        return;
    };
    println!("Server listening on 0.0.0.0:8888");
    let mut executed_commands = Vec::<String>::new();

    let mut buf = [0u8; 1024];
    loop {
        let Ok((len, src)) = socket.recv_from(&mut buf) else {
            println!("Failed to receive data");
            continue;
        };
        let msg = String::from_utf8_lossy(&buf[..len]);

        let mut response_buf = String::new();

        match AnyCommand::create_and_execute(msg.as_ref(), &mut context_db, &mut executed_commands, &mut response_buf) {
            Ok(_) => response_buf.push_str("Command executed successfully\n\n"),
            Err(e) => response_buf.push_str(&format!("{}\n", e)),
        }

        let Ok(_) = socket.send_to(response_buf.as_bytes(), src) else {
            println!("Failed to send response to {}", src);
            continue;
        };
    }
}