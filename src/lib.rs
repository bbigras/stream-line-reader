extern crate failure;
extern crate memchr;

use failure::Error;

use std::io::BufRead;

// \r 13
// \n 10

fn find_new_line(data: &[u8]) -> Option<(usize, usize)> {
    match memchr::memchr(b'\n', data) {
        Some(i) => {
            if i > 0 && data[i - 1] == b'\r' {
                Some((i - 1, 2))
            } else {
                Some((i, 1))
            }
        }
        None => None,
    }
}

#[derive(Clone, Debug)]
pub struct StreamReader<T> {
    inner: T,
    buffer: Vec<u8>,
    clear_next: bool,
}

impl<T: BufRead> StreamReader<T> {
    pub fn new(inner: T) -> StreamReader<T> {
        StreamReader {
            inner,
            buffer: Vec::new(),
            clear_next: false,
        }
    }

    pub fn line(&mut self) -> Result<(bool, Option<&[u8]>), Error> {
        if self.clear_next {
            self.buffer.clear();
        }
        self.clear_next = false;

        let (line, length) = {
            let buffer = self.inner.fill_buf().unwrap();
            if let Some((pos, size)) = find_new_line(buffer) {
                if pos == 0 {
                    if self.buffer.len() > 0 && self.buffer[self.buffer.len() - 1] == b'\r' {
                        self.buffer.pop();
                    }
                } else {
                    self.buffer.extend_from_slice(&buffer[..pos]);
                }

                (true, pos + size)
            } else {
                self.buffer.extend_from_slice(&buffer);
                (false, buffer.len())
            }
        };

        if length == 0 {
            return Ok((true, None));
        }

        self.inner.consume(length);

        if line {
            self.clear_next = true;
            Ok((false, Some(&self.buffer)))
        } else {
            Ok((false, None))
        }
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
            assert_eq!(r.line().unwrap(), (true, None));
        }

        {
            let buf = Cursor::new(&b"\n"[..]);
            let mut r = StreamReader::new(buf);
            assert_eq!(r.line().unwrap(), (false, Some(&b""[..])));
        }

        {
            let buf = Cursor::new(&b"12"[..]);
            let mut r = StreamReader::new(buf);
            assert_eq!(r.line().unwrap(), (false, None));
        }

        // ---

        {
            let buf = Cursor::new(&b"12"[..]);
            let mut r = StreamReader::new(buf);
            assert_eq!(r.line().unwrap(), (false, None));
        }

        {
            let buf = Cursor::new(&b"12\r\n"[..]);

            let mut r = StreamReader::new(buf);

            assert_eq!(r.line().unwrap(), (false, Some(&b"12"[..])));
            assert_eq!(r.line().unwrap(), (true, None));
        }

        {
            let buf = Cursor::new(&b"12\r\n1"[..]);

            let mut r = StreamReader::new(buf);

            assert_eq!(r.line().unwrap(), (false, Some(&b"12"[..])));
            assert_eq!(r.line().unwrap(), (false, None));
        }

        {
            let buf = Cursor::new(&b"12\r\n13\r\ntest"[..]);

            let mut r = StreamReader::new(buf);

            assert_eq!(r.line().unwrap(), (false, Some(&b"12"[..])));
            assert_eq!(r.line().unwrap(), (false, Some(&b"13"[..])));
            assert_eq!(r.line().unwrap(), (false, None));
        }

        // -----

        {
            let buf = Cursor::new(&b"12"[..]);
            let mut r = StreamReader::new(buf);
            assert_eq!(r.line().unwrap(), (false, None));
        }

        {
            let buf = Cursor::new(&b"12\n"[..]);

            let mut r = StreamReader::new(buf);

            assert_eq!(r.line().unwrap(), (false, Some(&b"12"[..])));
            assert_eq!(r.line().unwrap(), (true, None));
        }

        {
            let buf = Cursor::new(&b"12\n1"[..]);

            let mut r = StreamReader::new(buf);

            assert_eq!(r.line().unwrap(), (false, Some(&b"12"[..])));
            assert_eq!(r.line().unwrap(), (false, None));
        }

        {
            let buf = Cursor::new(&b"12\n13\ntest"[..]);

            let mut r = StreamReader::new(buf);

            assert_eq!(r.line().unwrap(), (false, Some(&b"12"[..])));
            assert_eq!(r.line().unwrap(), (false, Some(&b"13"[..])));
            assert_eq!(r.line().unwrap(), (false, None));
        }
    }

    #[test]
    fn it_works2() {
        {
            let mut r = StreamReader::new(Cursor::new(Vec::new()));

            r.inner.write(b"test").unwrap();
            r.inner.set_position(0);
            assert_eq!(r.line().unwrap(), (false, None));

            let last_pos = r.inner.position();

            r.inner.write(b"\r\nsome bytes\nttt").unwrap();
            r.inner.set_position(last_pos);

            assert_eq!(r.line().unwrap(), (false, Some(&b"test"[..])));
            assert_eq!(r.line().unwrap(), (false, Some(&b"some bytes"[..])));
            assert_eq!(r.line().unwrap(), (false, None));
        }
    }

    #[test]
    fn line_endings_win() {
        let mut r = StreamReader::new(Cursor::new("line1\r\nline 2\r\nsomething"));
        assert_eq!(r.line().unwrap(), (false, Some(&b"line1"[..])));
        assert_eq!(r.line().unwrap(), (false, Some(&b"line 2"[..])));
        assert_eq!(r.line().unwrap(), (false, None));
    }

    #[test]
    fn line_endings_unix() {
        let mut r = StreamReader::new(Cursor::new("line1\nline 2\nsomething"));
        assert_eq!(r.line().unwrap(), (false, Some(&b"line1"[..])));
        assert_eq!(r.line().unwrap(), (false, Some(&b"line 2"[..])));
        assert_eq!(r.line().unwrap(), (false, None));
    }

    #[test]
    fn test_multiples_newlines_unix() {
        let mut r = StreamReader::new(Cursor::new("\n\ntest\nthing"));
        assert_eq!(r.line().unwrap(), (false, Some(&b""[..])));
        assert_eq!(r.line().unwrap(), (false, Some(&b""[..])));
        assert_eq!(r.line().unwrap(), (false, Some(&b"test"[..])));
        assert_eq!(r.line().unwrap(), (false, None));
    }

    #[test]
    fn test_multiples_newlines_win() {
        let mut r = StreamReader::new(Cursor::new("\r\n\r\ntest\r\nthing"));
        assert_eq!(r.line().unwrap(), (false, Some(&b""[..])));
        assert_eq!(r.line().unwrap(), (false, Some(&b""[..])));
        assert_eq!(r.line().unwrap(), (false, Some(&b"test"[..])));
        assert_eq!(r.line().unwrap(), (false, None));
    }

    #[test]
    fn line_endings_both() {
        let mut r = StreamReader::new(Cursor::new("line1\r\nline 2\nsomething"));
        assert_eq!(r.line().unwrap(), (false, Some(&b"line1"[..])));
        assert_eq!(r.line().unwrap(), (false, Some(&b"line 2"[..])));
        assert_eq!(r.line().unwrap(), (false, None));
    }

    #[test]
    fn bug_middle_cr_lf() {
        let mut r = StreamReader::new(Cursor::new(Vec::new()));

        r.inner.write(b"[2018.10.02-03.10.58:467][952]LogSquadTrace: [DedicatedServer]ASQPlayerController::ChangeState(): PC=TotenKopf OldState=Inactive NewState=Playing\r").unwrap();
        r.inner.set_position(0);
        assert_eq!(r.line().unwrap(), (false, None));

        let last_pos = r.inner.position();

        r.inner.write(b"\n[2018.10.02-03.10.58:467]").unwrap();
        r.inner.set_position(last_pos);

        assert_eq!(r.line().unwrap(), (false, Some(&b"[2018.10.02-03.10.58:467][952]LogSquadTrace: [DedicatedServer]ASQPlayerController::ChangeState(): PC=TotenKopf OldState=Inactive NewState=Playing"[..])));
    }

    #[test]
    fn test_find_new_line() {
        {
            let data = "aaaaaaaaaaaaaaaaaaaaa\naaaaaaaaaaaaaaaa";
            assert_eq!(find_new_line(data.as_bytes()).unwrap(), (21, 1));
        }
        {
            let data = "aaaaaaaaaaaaaaaaa\r\naaaaaaaaaaaaaaaa";
            assert_eq!(find_new_line(data.as_bytes()).unwrap(), (17, 2));
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
