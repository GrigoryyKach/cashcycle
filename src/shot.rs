//! `--screenshot <file.png>`: renders the window, saves it as a PNG and exits. Used for the README images,
//! so no synthetic input and no foreground window are needed.

use eframe::egui::{self, ColorImage};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct Shot {
    path: PathBuf,
    frames: u32,
    asked: bool,
    done: Arc<AtomicBool>,
}

impl Shot {
    pub fn from_args(args: &[String]) -> Option<Shot> {
        let i = args.iter().position(|a| a == "--screenshot")?;
        Some(Shot { path: PathBuf::from(args.get(i + 1)?), frames: 0, asked: false, done: Arc::new(AtomicBool::new(false)) })
    }

    /// Call every frame. Asks for a screenshot after the animations have settled and closes the window after saving.
    pub fn tick(&mut self, ctx: &egui::Context) {
        self.frames += 1;
        ctx.request_repaint();
        if self.frames == 40 && !self.asked {
            self.asked = true;
            let (path, done, c) = (self.path.clone(), self.done.clone(), ctx.clone());
            ctx.request_screenshot(move |img| {
                let _ = std::fs::write(&path, png(&img));
                done.store(true, Ordering::SeqCst);
                c.request_repaint();
            });
        }
        if self.done.load(Ordering::SeqCst) || self.frames > 600 {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
}

fn crc32(data: &[u8]) -> u32 {
    let mut table = [0u32; 256];
    for (n, slot) in table.iter_mut().enumerate() {
        let mut c = n as u32;
        for _ in 0..8 {
            c = if c & 1 != 0 { 0xedb8_8320 ^ (c >> 1) } else { c >> 1 };
        }
        *slot = c;
    }
    let mut c = 0xffff_ffffu32;
    for b in data {
        c = table[((c ^ *b as u32) & 0xff) as usize] ^ (c >> 8);
    }
    c ^ 0xffff_ffff
}

fn adler32(data: &[u8]) -> u32 {
    let (mut a, mut b) = (1u32, 0u32);
    for d in data {
        a = (a + *d as u32) % 65521;
        b = (b + a) % 65521;
    }
    (b << 16) | a
}

fn chunk(out: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    let mut body = kind.to_vec();
    body.extend_from_slice(data);
    out.extend_from_slice(&body);
    out.extend_from_slice(&crc32(&body).to_be_bytes());
}

/// An RGBA PNG with stored (uncompressed) deflate blocks: big, but it needs no extra crate.
pub fn png(img: &ColorImage) -> Vec<u8> {
    let (w, h) = (img.size[0], img.size[1]);
    let mut raw = Vec::with_capacity((w * 4 + 1) * h);
    for y in 0..h {
        raw.push(0);
        for px in &img.pixels[y * w..(y + 1) * w] {
            raw.extend_from_slice(&[px.r(), px.g(), px.b(), 255]);
        }
    }
    let mut z = vec![0x78, 0x01];
    let mut chunks = raw.chunks(65535).peekable();
    while let Some(c) = chunks.next() {
        z.push(if chunks.peek().is_none() { 1 } else { 0 });
        z.extend_from_slice(&(c.len() as u16).to_le_bytes());
        z.extend_from_slice(&(!(c.len() as u16)).to_le_bytes());
        z.extend_from_slice(c);
    }
    z.extend_from_slice(&adler32(&raw).to_be_bytes());
    let mut out = vec![0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];
    let mut ihdr = Vec::new();
    ihdr.extend_from_slice(&(w as u32).to_be_bytes());
    ihdr.extend_from_slice(&(h as u32).to_be_bytes());
    ihdr.extend_from_slice(&[8, 6, 0, 0, 0]);
    chunk(&mut out, b"IHDR", &ihdr);
    chunk(&mut out, b"IDAT", &z);
    chunk(&mut out, b"IEND", &[]);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn png_has_signature_size_and_valid_checksums() {
        let img = ColorImage::filled([3, 2], egui::Color32::from_rgb(10, 20, 30));
        let bytes = png(&img);
        assert_eq!(&bytes[..8], &[0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a]);
        assert_eq!(&bytes[12..16], b"IHDR");
        assert_eq!(u32::from_be_bytes(bytes[16..20].try_into().unwrap()), 3);
        assert_eq!(u32::from_be_bytes(bytes[20..24].try_into().unwrap()), 2);
        // The IHDR checksum matches (known-answer for the CRC routine).
        let crc = u32::from_be_bytes(bytes[29..33].try_into().unwrap());
        assert_eq!(crc, crc32(&bytes[12..29]));
        assert_eq!(crc32(b"123456789"), 0xcbf4_3926);
        assert_eq!(adler32(b"Wikipedia"), 0x11e6_0398);
        assert!(bytes.ends_with(&[0x49, 0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82]));
    }
}
