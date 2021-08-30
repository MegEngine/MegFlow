/**
 * \file flow-plugins/src/utils/multipart.rs
 * MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
 *
 * Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 */
use mime::Mime;
use rand::distributions::{Alphanumeric, Distribution};
use std::str::FromStr;

pub struct MultipartWriter<W> {
    inner: W,
    boundary: String,
    data_written: bool,
}

impl<W> MultipartWriter<W> {
    pub fn new(inner: W) -> Self {
        let mut boundary = String::with_capacity(32);
        boundary.extend(
            Alphanumeric
                .sample_iter(rand::thread_rng())
                .take(32)
                .map(|c| c as char),
        );
        MultipartWriter {
            inner,
            boundary,
            data_written: false,
        }
    }

    fn get_field_header(&self, name: &str, content_type: &Mime) -> String {
        format!(
            "--{}\r\nContent-Disposition: attachment; name=\"{}\"\r\nContent-Type: {}\r\n\r\n",
            self.boundary, name, content_type
        )
    }

    pub fn into_inner(self) -> W {
        self.inner
    }

    pub fn header(&self) -> Mime {
        Mime::from_str(format!("multipart/form-data; boundary={}", self.boundary).as_str()).unwrap()
    }
}

impl<W: std::io::Write> MultipartWriter<W> {
    fn write_field_header(&mut self, name: &str, content_type: &Mime) -> std::io::Result<()> {
        let mut header = std::io::Cursor::new(self.get_field_header(name, content_type));
        std::io::copy(&mut header, &mut self.inner)?;
        self.data_written = true;
        Ok(())
    }

    pub fn write_field_with<F>(
        &mut self,
        name: &str,
        content_type: &Mime,
        f: F,
    ) -> std::io::Result<&mut Self>
    where
        F: Fn(&mut W) -> std::io::Result<()>,
    {
        self.write_field_header(name, content_type)?;
        f(&mut self.inner)?;
        self.inner.write_all(b"\r\n")?;
        Ok(self)
    }

    pub fn write_field<R: std::io::Read>(
        &mut self,
        name: &str,
        content_type: &Mime,
        mut contents: R,
    ) -> std::io::Result<&mut Self> {
        self.write_field_header(name, content_type)?;
        std::io::copy(&mut contents, &mut self.inner)?;
        self.inner.write_all(b"\r\n")?;
        Ok(self)
    }

    pub fn write_text(&mut self, name: &str, text: &str) -> std::io::Result<&mut Self> {
        self.write_field(name, &mime::TEXT_PLAIN, text.as_bytes())
    }

    pub fn finish(&mut self) -> std::io::Result<()> {
        if self.data_written {
            self.inner.write_all(b"--")?;
            self.inner.write_all(self.boundary.as_bytes())?;
            self.inner.write_all(b"--\r\n")?;
        }
        self.inner.flush()?;
        Ok(())
    }
}
