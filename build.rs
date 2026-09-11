use std::{env, fs, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR"));
    let icon_path = out_dir.join("distronomicon.ico");
    fs::write(&icon_path, make_icon(64)).expect("write generated icon");

    let mut res = winres::WindowsResource::new();
    res.set_icon(icon_path.to_str().expect("icon path"));
    res.set("ProductName", "Distronomicon Desktop");
    res.set("FileDescription", "Distronomicon Desktop Portable");
    res.set("LegalCopyright", "MIT License");
    res.compile().expect("compile Windows resources");
}

fn make_icon(size: u32) -> Vec<u8> {
    let xor_bytes = (size * size * 4) as usize;
    let mask_stride = size.div_ceil(32) * 4;
    let mask_bytes = (mask_stride * size) as usize;
    let image_bytes = 40 + xor_bytes + mask_bytes;
    let file_bytes = 6 + 16 + image_bytes;

    let mut out = Vec::with_capacity(file_bytes);
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.push(if size >= 256 { 0 } else { size as u8 });
    out.push(if size >= 256 { 0 } else { size as u8 });
    out.push(0);
    out.push(0);
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&32u16.to_le_bytes());
    out.extend_from_slice(&(image_bytes as u32).to_le_bytes());
    out.extend_from_slice(&22u32.to_le_bytes());

    out.extend_from_slice(&40u32.to_le_bytes());
    out.extend_from_slice(&(size as i32).to_le_bytes());
    out.extend_from_slice(&((size * 2) as i32).to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&32u16.to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(&(xor_bytes as u32).to_le_bytes());
    out.extend_from_slice(&0i32.to_le_bytes());
    out.extend_from_slice(&0i32.to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());

    let s = size as i32;
    let cx = s / 2;
    let cy = s / 2;
    for y in (0..s).rev() {
        for x in 0..s {
            let dx = x - cx;
            let dy = y - cy;
            let r2 = dx * dx + dy * dy;
            let outer = (s * 45 / 100).pow(2);
            let inner = (s * 30 / 100).pow(2);
            let gold = r2 <= outer && r2 >= inner;
            let stem = x >= s * 27 / 100 && x <= s * 38 / 100 && y >= s * 24 / 100 && y <= s * 76 / 100;
            let open_d = x > cx && r2 < inner;
            let (r, g, b, a) = if gold || stem {
                (214u8, 166u8, 84u8, 255u8)
            } else if open_d {
                (24u8, 27u8, 33u8, 255u8)
            } else {
                (17u8, 19u8, 24u8, 255u8)
            };
            out.extend_from_slice(&[b, g, r, a]);
        }
    }
    out.resize(out.len() + mask_bytes, 0);
    out
}
