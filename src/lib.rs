extern crate hostname;
extern crate username;

use hostname::get_hostname;
use username::get_user_name;

use std::{
    fs::File,
    io::{self, Error, ErrorKind, Read, Write},
    net::TcpStream,
    process,
    time::Duration,
};

pub struct LprConnection {
    stream: TcpStream,
    verbose: bool,
}

impl LprConnection {
    pub fn new(ip_str: &str, verbose: bool) -> LprConnection {
        let target = format!("{}:515", ip_str);
        let stream = TcpStream::connect(&target)
            .unwrap_or_else(|err| panic!("connecting to {}: {}", target, err));
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .expect("setting read timeout");
        LprConnection { stream, verbose }
    }

    pub fn status(mut self) -> io::Result<String> {
        let bytes_written = self.stream.write(&[4, b'\n'])?;
        match bytes_written {
            2 => {
                let mut buf = [0; 128];
                self.stream.read_exact(&mut buf)?;
                let buf_str = String::from_utf8_lossy(&buf).to_string();
                let split: Vec<&str> = buf_str.split("\n\n").collect();
                Ok(split[0].to_string())
            }
            _ => Err(Error::new(
                ErrorKind::Interrupted,
                "not all bytes have been written",
            )),
        }
    }

    fn send_and_wait_for_ack(&mut self, data: &[u8], description: &str) -> io::Result<()> {
        if self.verbose {
            print!("Sending {}.. ", description);
        }
        self.stream.write_all(data)?;

        let mut buf = [0; 1];
        self.stream.read_exact(&mut buf)?;
        if self.verbose {
            println!("acknowledged");
        }
        if buf[0] != 0 {
            panic!("received invalid acknowledge: {:x}", buf[0]);
        }
        Ok(())
    }

    fn generate_control_file_and_name(&self) -> (String, String) {
        let host = match get_hostname() {
            Some(name) => name,
            None => "lpr-host".to_string(),
        };
        let user = match get_user_name() {
            Ok(name) => name,
            Err(_) => "lpr-user".to_string(),
        };
        let name = format!("fA{}{}", process::id() % 1000, host);
        (format!("H{}\nP{}\nld{}\n", host, user, name), name)
    }

    pub fn print_file(&mut self, path_to_file: &str) -> io::Result<()> {
        if self.verbose {
            print!("Priting {}.. ", &path_to_file)
        }
        let mut file = File::open(path_to_file)?;
        let mut buf: Vec<u8> = Vec::with_capacity(8096);
        let file_size = file.read_to_end(&mut buf)?;
        if self.verbose {
            println!("File Size: {:?}", file_size);
        }
        self.print(&buf)?;
        Ok(())
    }

    pub fn print_file_with_pjl_header(
        &mut self,
        path_to_file: &str,
        mut header_data: Vec<u8>,
    ) -> io::Result<()> {
        let mut buf: Vec<u8> = Vec::with_capacity(8096);
        let mut file = File::open(path_to_file)?;
        let mut file_buf: Vec<u8> = Vec::with_capacity(8096);
        let _file_size = file.read_to_end(&mut file_buf)?;
        buf.append(&mut header_data);
        buf.append(&mut file_buf);
        buf.append(&mut b"\x1b%-12345X@PJL EOJ\r\n\x1b%-12345X".to_vec());
        self.print(&buf)?;
        Ok(())
    }

    pub fn print(&mut self, data: &[u8]) -> io::Result<()> {
        let (controlfile, job_name) = self.generate_control_file_and_name();
        if self.verbose {
            println!("generated controlfile:\n{}", controlfile)
        }
        self.send_and_wait_for_ack(b"\x02lp\n", "receive job command")?;

        self.send_and_wait_for_ack(
            &format!("\x02{} c{}\n", controlfile.len(), job_name).as_bytes(),
            "receive controlfile subcommand",
        )?;

        self.send_and_wait_for_ack(&format!("{}\x00", controlfile).as_bytes(), "control file")?;

        self.send_and_wait_for_ack(
            &format!("\x03{} d{}\n", data.len(), job_name).as_bytes(),
            "receive datafile subcommand",
        )?;

        self.stream.write_all(data)?;
        self.send_and_wait_for_ack(&[0], "data file and ack")?;
        Ok(())
    }
}
