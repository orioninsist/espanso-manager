use std::{fs, path::Path};

fn main() {
    // Tauri's compile-time context expects the default icon path even when
    // application bundling is disabled. Keep a tiny valid RGBA PNG available
    // during development; release artwork will replace it later.
    let icon_dir = Path::new("icons");
    let icon_path = icon_dir.join("icon.png");

    if !icon_path.exists() {
        fs::create_dir_all(icon_dir).expect("failed to create icons directory");

        // 1x1 transparent RGBA PNG.
        const PNG: &[u8] = &[
            137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82,
            0, 0, 0, 1, 0, 0, 0, 1, 8, 6, 0, 0, 0, 31, 21, 196, 137,
            0, 0, 0, 13, 73, 68, 65, 84, 8, 215, 99, 96, 96, 96, 96, 0,
            0, 0, 5, 0, 1, 94, 242, 26, 11, 0, 0, 0, 0, 73, 69, 78, 68,
            174, 66, 96, 130,
        ];

        fs::write(&icon_path, PNG).expect("failed to create development icon");
    }

    tauri_build::build()
}
