#![recursion_limit = "1024"]

use std::io::Read;

#[macro_use]
extern crate error_chain;

extern crate memchr;

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
    match memchr::memchr(b'\n', data) {
        Some(i) => if i > 0 && data[i - 1] == b'\r' {
            return Some(i - 1);
        } else {
            return Some(i);
        },
        None => return None,
    };

    return None;
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use std::io::Write;

    #[test]
    fn it_works() {
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
        {
            let mut r = StreamReader::new(Cursor::new(Vec::new()));

            r.inner.write(b"test").unwrap();
            r.inner.set_position(0);
            assert_eq!(r.line().unwrap(), None);

            let last_pos = r.inner.position();

            r.inner.write(b"\r\nsome bytes\nttt").unwrap();
            r.inner.set_position(last_pos);

            assert_eq!(r.line().unwrap(), Some("test".to_string()));
            assert_eq!(r.line().unwrap(), Some("some bytes".to_string()));
            assert_eq!(r.line().unwrap(), None);
        }
    }

    #[test]
    fn line_endings_win() {
        let mut r = StreamReader::new(Cursor::new("line1\r\nline 2\r\nsomething"));
        assert_eq!(r.line().unwrap(), Some("line1".to_string()));
        assert_eq!(r.line().unwrap(), Some("line 2".to_string()));
        assert_eq!(r.line().unwrap(), None);
    }

    #[test]
    fn line_endings_unix() {
        let mut r = StreamReader::new(Cursor::new("line1\nline 2\nsomething"));
        assert_eq!(r.line().unwrap(), Some("line1".to_string()));
        assert_eq!(r.line().unwrap(), Some("line 2".to_string()));
        assert_eq!(r.line().unwrap(), None);
    }

    #[test]
    fn line_endings_both() {
        let mut r = StreamReader::new(Cursor::new("line1\r\nline 2\nsomething"));
        assert_eq!(r.line().unwrap(), Some("line1".to_string()));
        assert_eq!(r.line().unwrap(), Some("line 2".to_string()));
        assert_eq!(r.line().unwrap(), None);
    }

    #[test]
    fn test_find_new_line() {
        {
            let data = "aaaaaaaaaaaaaaaaaaaaa\naaaaaaaaaaaaaaaa";
            assert_eq!(find_new_line(data.as_bytes()).unwrap(), 21);
        }
        {
            let data = "aaaaaaaaaaaaaaaaa\r\naaaaaaaaaaaaaaaa";
            assert_eq!(find_new_line(data.as_bytes()).unwrap(), 17);
        }
        {
            let data = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
            assert_eq!(find_new_line(data.as_bytes()), None);
        }
        {
            let data = "aaaaaaaaaaaaaaaaa\raaaaaaaaaaaaaaaaaaaa";
            assert_eq!(find_new_line(data.as_bytes()), None);
        }
        {
            let data = "";
            assert_eq!(find_new_line(data.as_bytes()), None);
        }
    }
}
