#[macro_use]
extern crate clap;

extern crate lpr;

use lpr::LprConnection;

fn main() {
    let matches = clap_app!(lpr =>
        (version: "0.1.0")
        (author: "Gerrit Pape <papeg@crowler.org>")
        (about: "simple LPR client using the classic Line Printer Daemon Protocol")
        (@arg status: -s --status "Prints the status of the queue")
        (@arg verbose: -v -vv --verbose "Go verbose")
        (@arg printer: +required "Address of Printer to target")
        (@arg file: "File to print")
    )
    .get_matches();

    let printer = matches.value_of("printer").unwrap_or("127.0.0.1");

    let mut conn = LprConnection::new(printer, 4200);

    conn.verbose(matches.is_present("verbose"));

    if matches.is_present("status") {
        println!(
            "{}",
            match conn.status() {
                Ok(status) => status,
                Err(e) => e.to_string(),
            }
        );
    } else if matches.is_present("file") {
        let file = matches.value_of("file").unwrap();
        conn.print_file(file).expect("printing file");
    }
}
