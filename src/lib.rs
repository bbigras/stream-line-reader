#![recursion_limit = "1024"]

use std::io::Cursor;
use std::io::Read;

#[macro_use]
extern crate error_chain;

pub mod errors {
    error_chain! {
        foreign_links {
            Io(::std::io::Error);
            Utf(::std::string::FromUtf8Error);
        }
    }
}

use errors::*;

// \r 13
// \n 10

fn find_new_line(data: &[u8]) -> Option<usize> {
    for (index, one) in data.iter().peekable().enumerate() {
        if *one == 13 || *one == 10 {
            return Some(index);
        }
    }
    None
}

#[derive(Clone, Debug)]
pub struct StreamReader<T> {
    inner: T,
    pos: u64,
    buffer: Vec<u8>,
}

impl<T: Read> StreamReader<T> {
    pub fn new(inner: T) -> StreamReader<T> {
        StreamReader { pos: 0, inner: inner, buffer: Vec::new() }
    }

    pub fn buf_size(&mut self) -> usize {
        self.buffer.len()
    }

    pub 
    fn line(&mut self) -> Result<Option<String>> {
        {
            let i = find_new_line(&self.buffer);

            if let Some(i2) = i {
                let a2 = {
                    let a1: Vec<u8> = self.buffer.iter().take(i2).map(|b| b.clone()).collect();

                    if self.buffer.get(i2+1) == Some(&b'\n') {
                        self.buffer.drain(0..i2+2);
                    } else {
                        self.buffer.drain(0..i2+1);
                    }

                    String::from_utf8_lossy(&a1).into_owned()
                };

                return Ok(Some(a2));
            }
        }

        let mut buf2 = vec![0; 1024];

        let size = self.inner.read(&mut buf2)?;

        if size > 0 {
            self.buffer.append(&mut buf2[0..size].to_vec());
        }

        let i = find_new_line(&self.buffer);

        if let Some(i2) = i {
            let a2 = {
                let a1: Vec<u8> = self.buffer.iter().take(i2).map(|b| b.clone()).collect();

                if self.buffer.get(i2+1) == Some(&b'\n') {
                    self.buffer.drain(0..i2+2);
                } else {
                    self.buffer.drain(0..i2+1);
                }

                String::from_utf8_lossy(&a1).into_owned()
            };

            return Ok(Some(a2));
        }
        Ok(None)
    }
}

/*
fn toto<T: Read>(t: &T) -> Option<String> {
    None
}
*/

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        use ::StreamReader;
        use std::io::Cursor;
        use std::io::Read;

        {
            let buf = Cursor::new(&b""[..]);
            let mut r = StreamReader::new(buf);
            assert_eq!(r.line().unwrap(), None);
        }

        {
            let buf = Cursor::new(&b"\n"[..]);
            let mut r = StreamReader::new(buf);
            assert_eq!(r.line().unwrap(), Some(String::new()));
        }

        {
            let buf = Cursor::new(&b"12"[..]);
            let mut r = StreamReader::new(buf);
            assert_eq!(r.line().unwrap(), None);
        }

        {
            let buf = Cursor::new(&b"12\r"[..]);

            let mut r = StreamReader::new(buf);

            assert_eq!(r.line().unwrap(), Some("12".to_string()));
            assert_eq!(r.line().unwrap(), None);
        }

        {
            let buf = Cursor::new(&b"12\r1"[..]);

            let mut r = StreamReader::new(buf);

            assert_eq!(r.line().unwrap(), Some("12".to_string()));
            assert_eq!(r.line().unwrap(), None);
        }

        {
            let buf = Cursor::new(&b"12\r13\rtest"[..]);

            let mut r = StreamReader::new(buf);

            assert_eq!(r.line().unwrap(), Some("12".to_string()));
            assert_eq!(r.line().unwrap(), Some("13".to_string()));
            assert_eq!(r.line().unwrap(), None);
        }


        // ---

        {
            let buf = Cursor::new(&b"12"[..]);
            let mut r = StreamReader::new(buf);
            assert_eq!(r.line().unwrap(), None);
        }

        {
            let buf = Cursor::new(&b"12\r\n"[..]);

            let mut r = StreamReader::new(buf);

            assert_eq!(r.line().unwrap(), Some("12".to_string()));
            assert_eq!(r.line().unwrap(), None);
        }

        {
            let buf = Cursor::new(&b"12\r\n1"[..]);

            let mut r = StreamReader::new(buf);

            assert_eq!(r.line().unwrap(), Some("12".to_string()));
            assert_eq!(r.line().unwrap(), None);
        }

        {
            let buf = Cursor::new(&b"12\r\n13\r\ntest"[..]);

            let mut r = StreamReader::new(buf);

            assert_eq!(r.line().unwrap(), Some("12".to_string()));
            assert_eq!(r.line().unwrap(), Some("13".to_string()));
            assert_eq!(r.line().unwrap(), None);
        }

        // -----

        {
            let buf = Cursor::new(&b"12"[..]);
            let mut r = StreamReader::new(buf);
            assert_eq!(r.line().unwrap(), None);
        }

        {
            let buf = Cursor::new(&b"12\n"[..]);

            let mut r = StreamReader::new(buf);

            assert_eq!(r.line().unwrap(), Some("12".to_string()));
            assert_eq!(r.line().unwrap(), None);
        }

        {
            let buf = Cursor::new(&b"12\n1"[..]);

            let mut r = StreamReader::new(buf);

            assert_eq!(r.line().unwrap(), Some("12".to_string()));
            assert_eq!(r.line().unwrap(), None);
        }

        {
            let buf = Cursor::new(&b"12\n13\ntest"[..]);

            let mut r = StreamReader::new(buf);

            assert_eq!(r.line().unwrap(), Some("12".to_string()));
            assert_eq!(r.line().unwrap(), Some("13".to_string()));
            assert_eq!(r.line().unwrap(), None);
        }
    }

    #[test]
    fn it_works2() {
        use ::StreamReader;
        use std::io::Cursor;
        use std::io::Read;
        use std::io::Write;
        use std::io::SeekFrom;
        use std::io::Seek;

        {
            //let mut aaa: Vec<u8> = b"test"[..].to_vec();
            let mut aaa = Vec::new();
            //let mut aaa = Vec::with_capacity(1024);
            {
            let mut buf = Cursor::new(aaa);
            {
                let mut r = StreamReader::new(buf.clone());

                buf.get_mut().append(&mut b"test".to_vec()); //= &mut b"test"[..].to_vec();
                //buf.write(b"test").unwrap();
                //buf.seek(SeekFrom::Start(0)).unwrap();
                assert_eq!(r.line().unwrap(), None);

                buf.get_mut().append(&mut b"\nsomething".to_vec());
                //buf.write(b"\rsome bytes\nttt").unwrap();
                //println!("aaa: {:?}", buf.into_inner());
                //buf.seek(SeekFrom::Start(0)).unwrap();
                //buf.set_position(0);
                //println!("aaa: {:?}", buf.into_inner());

                assert_eq!(r.line().unwrap(), Some("test".to_string()));
                //assert!(false);
            }
            }

            
            
        }
    }
}
