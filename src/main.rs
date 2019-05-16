#[macro_use]
extern crate clap;

extern crate lpr;

use lpr::LprConnection;

fn main() {
    let matches = clap_app!(lpr =>
        (version: "0.1.4")
        (author: "Gerrit Pape <papeg@crowler.org>")
        (about: "simple LPR client using the classic Line Printer Daemon Protocol - TCP only")
        (@arg verbose: -v -vv --verbose "go verbose")
        (@arg printer: +required "IP-Address of printer to target")
        (@arg file: "path to file to print")
    )
    .get_matches();

    let printer = matches.value_of("printer").unwrap_or("127.0.0.1");

    let mut conn = LprConnection::new(printer, 4200)
        .expect("creating LprConnection");

    conn.verbose(matches.is_present("verbose"));

    if let Some(file) = matches.value_of("file") {
        conn.print_file(file).expect("printing file");
    } else {
        println!(
            "{}",
            match conn.status() {
                Ok(status) => status,
                Err(e) => e.to_string(),
            }
        );
    }
}
