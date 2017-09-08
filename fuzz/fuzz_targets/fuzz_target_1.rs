#![no_main]
#[macro_use]
extern crate libfuzzer_sys;
extern crate stream_line_reader;

use stream_line_reader::StreamReader;

fuzz_target!(|data: &[u8]| {
    let mut r = StreamReader::new(data);
    loop {
        if r.line().unwrap().is_none() {
            break;
        }
    }
});
