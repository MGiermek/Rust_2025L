use std::collections::BTreeSet;
use std::fs;
use std::hint::black_box;
use std::net::{TcpListener, TcpStream};
use std::num::NonZero;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Instant;
use std::io::{Read, Result, Write};

fn main() {
    println!("Hello, world!");
    let v = vec![1,2,3,4,5];
    assert_sorted(&v);
    benchmark_divisors();

    let listener = TcpListener::bind("127.0.0.1:8888").unwrap();
    for stream in listener.incoming() {
        if let Ok(mut stream) = stream && let Err(error) = handle_client(&mut stream) {
            println!("{:?}", error);
        }
        
    }
}

fn divisors(n_nonzero: NonZero<u32>) -> BTreeSet<NonZero<u32>> {
    let mut i = 2;
    let n = n_nonzero.get();
    let mut ret_set = BTreeSet::<NonZero<u32>>::new();
    while i*i < n {
        if n.is_multiple_of(i) {
            ret_set.insert(NonZero::new(i).unwrap());
            ret_set.insert(NonZero::new(n / i).unwrap());
        }
        i += 1;
    }
    if i*i == n {
        ret_set.insert(NonZero::new(i).unwrap());
    }
    ret_set
}

fn assert_sorted(vec: &Vec<i32>) {
    let iter = vec.windows(2);
    for pair in iter {
        if pair[0] > pair[1] {
            panic!("{} is bigger than {}!", pair[0], pair[1]);
        }
    }
    println!("Vector was sorted!");
}

fn benchmark_divisors() {
    let now = Instant::now();
    let rep: u32 = 100;
    for i in 1..=rep {
        black_box(divisors(NonZero::new(i).unwrap()));
    }
    let elapsed_time = now.elapsed();
    println!("Divisors took on average {} ms", (elapsed_time.as_micros() as f64) / f64::from(1000*rep));
}

fn bulk_write(stream: &mut TcpStream, buf: &[u8]) -> Result<()> {
    let mut remaining = buf;
    let mut n = buf.len();
    while n > 0 {
        let written = stream.write(remaining)?;
        remaining = &remaining[written..];
        n -= written;
    }
    Ok(())
}

fn bulk_read(stream: &mut TcpStream, size: usize) -> Result<Vec<u8>> {
    let mut byte_count = 0;
    let mut ret_vec = Vec::<u8>::new();
    let mut buf: [u8; 1000] = [0; 1000];
    while byte_count < size {
        let bytes_read =stream.read(&mut buf)?;
        if bytes_read == 0 {
            break;
        }
        let to_add = &buf[..bytes_read];
        ret_vec.extend_from_slice(to_add);
        byte_count += bytes_read;
    }
    Ok(ret_vec)
}

fn handle_client(stream: &mut TcpStream) -> Result<()> {
    let message = bulk_read(stream, 1000)?;
    let Ok(message_str) = str::from_utf8(message.as_slice().trim_ascii()) else {
        bulk_write(stream, str::as_bytes("Bad path\n"))?;
        return Ok(());
    };

    let path_buf = PathBuf::from_str(message_str).unwrap(); // this conversion is infallible

    let dir = fs::read_dir(path_buf);
    match dir {
        Ok(ok_dir) => {
            let mut to_send = String::new();
            for dir_entry in ok_dir {
                if let Ok(dir_entry) = dir_entry && let Some(dir_name) = dir_entry.file_name().to_str() {
                    to_send.push_str(dir_name);
                    to_send.push('\n');
                }
            }
            bulk_write(stream, to_send.as_bytes())?;
            println!("{}", to_send);
        }
        Err(error) => {
            println!("{:?}", error);
            bulk_write(stream, str::as_bytes("Bad dir\n"))?;
            return Ok(());
        }
    }
    Ok(())
}