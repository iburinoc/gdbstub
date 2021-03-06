use alloc::format;

use crate::Connection;

#[derive(Debug)]
pub struct Error<C>(C);

/// A wrapper around [`Connection`] that computes the single-byte checksum of
/// incoming / outgoing data.
pub struct ResponseWriter<'a, C: 'a> {
    inner: &'a mut C,
    started: bool,
    checksum: u8,
    #[cfg(feature = "std")]
    msg: String,
}

impl<'a, C: Connection + 'a> ResponseWriter<'a, C> {
    /// Creates a new ResponseWriter
    pub fn new(inner: &'a mut C) -> Self {
        Self {
            inner,
            started: false,
            checksum: 0,
            #[cfg(feature = "std")]
            msg: String::new(),
        }
    }

    /// Consumes self, writing out the final '#' and checksum
    pub fn flush(mut self) -> Result<(), Error<C::Error>> {
        // don't include '#' in checksum calculation
        let checksum = self.checksum;

        #[cfg(feature = "std")]
        log::trace!("--> ${}#{:02x?}", self.msg, checksum);

        self.write(b'#')?;
        self.write_hex(checksum)?;

        Ok(())
    }

    /// Write a single byte.
    pub fn write(&mut self, byte: u8) -> Result<(), Error<C::Error>> {
        #[cfg(feature = "std")]
        self.msg.push(byte as char);

        if !self.started {
            self.started = true;
            self.inner.write(b'$').map_err(Error)?;
        }

        self.checksum = self.checksum.wrapping_add(byte);
        self.inner.write(byte).map_err(Error)
    }

    /// Write an entire buffer over the connection.
    pub fn write_all(&mut self, data: &[u8]) -> Result<(), Error<C::Error>> {
        data.iter().try_for_each(|b| self.write(*b))
    }

    /// Write an entire string over the connection.
    pub fn write_str(&mut self, s: &str) -> Result<(), Error<C::Error>> {
        self.write_all(&s.as_bytes())
    }

    /// Write a single byte as a hex string (two ascii chars)
    pub fn write_hex(&mut self, byte: u8) -> Result<(), Error<C::Error>> {
        let hex_str = format!("{:02x}", byte);
        self.write(hex_str.as_bytes()[0])?;
        self.write(hex_str.as_bytes()[1])?;
        Ok(())
    }

    /// Write an entire buffer as a hex string (two ascii chars / byte).
    pub fn write_hex_buf(&mut self, data: &[u8]) -> Result<(), Error<C::Error>> {
        data.iter().try_for_each(|b| self.write_hex(*b))
    }

    /// Write data using the binary protocol (i.e: escaping any bytes that are
    /// not 7-bit clean)
    pub fn write_binary(&mut self, data: &[u8]) -> Result<(), Error<C::Error>> {
        data.iter().try_for_each(|b| match b {
            b'#' | b'$' | b'}' | b'*' => {
                self.write(0x7d)?;
                self.write(*b ^ 0x20)
            }
            b if b & 0x80 != 0 => {
                self.write(0x7d)?;
                self.write(*b ^ 0x20)
            }
            _ => self.write(*b),
        })
    }
}
