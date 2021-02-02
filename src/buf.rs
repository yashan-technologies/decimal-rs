// Copyright 2021 CoD Technologies Corp.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::mem::MaybeUninit;

pub struct Buf {
    buf: MaybeUninit<[u8; 256]>,
    len: usize,
}

impl Buf {
    #[inline]
    pub const fn new() -> Buf {
        Buf {
            buf: MaybeUninit::uninit(),
            len: 0,
        }
    }

    #[inline]
    fn as_mut(&mut self) -> &mut [u8; 256] {
        unsafe { &mut *self.buf.as_mut_ptr() }
    }

    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        let s = unsafe { &*self.buf.as_ptr() };
        &s[0..self.len]
    }

    #[inline]
    pub fn write_u8(&mut self, value: u8) {
        let i = self.len;
        self.as_mut()[i] = value;
        self.len += 1;
    }

    #[inline]
    pub fn write_slice(&mut self, slice: &[u8]) {
        let i = self.len;
        let len = slice.len();
        self.as_mut()[i..i + len].copy_from_slice(slice);
        self.len += len;
    }

    #[inline]
    pub fn write_bytes(&mut self, val: u8, count: usize) {
        let i = self.len;
        let s = self.as_mut()[i..i + count].as_mut_ptr();
        unsafe {
            s.write_bytes(val, count);
        }
        self.len += count;
    }

    #[inline]
    pub fn truncate(&mut self, len: usize) {
        if len < self.len {
            self.len = len;
        }
    }
}

impl std::io::Write for Buf {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.write_slice(buf);
        Ok(buf.len())
    }

    #[inline]
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
