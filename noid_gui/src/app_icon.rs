// SPDX-License-Identifier: Apache-2.0
// Copyright (C) 2026 Paranoid Zero.

use iced::window;

const APPLICATION_ICON: &[u8] = include_bytes!("../assets/app-icons/retro-boomer-512.png");

// Official monochrome cassette assets are embedded as supplied: preserve every color and alpha.
fn logo_pixels(bytes: &[u8]) -> image::RgbaImage {
    image::load_from_memory(bytes)
        .expect("valid Retro Boomer icon")
        .into_rgba8()
}

pub fn logo_handle() -> iced::widget::image::Handle {
    static LOGO: std::sync::OnceLock<iced::widget::image::Handle> = std::sync::OnceLock::new();
    LOGO.get_or_init(|| {
        let pixels = logo_pixels(include_bytes!("../assets/app-icons/retro-boomer-1024.png"));
        let (w, h) = pixels.dimensions();
        iced::widget::image::Handle::from_rgba(w, h, pixels.into_raw())
    })
    .clone()
}

/// Dedicated header artwork; other logo views retain their existing source and size.
pub fn header_logo_handle() -> iced::widget::image::Handle {
    static HEADER_LOGO: std::sync::OnceLock<iced::widget::image::Handle> =
        std::sync::OnceLock::new();
    HEADER_LOGO
        .get_or_init(|| {
            let pixels = logo_pixels(include_bytes!("../assets/app-icons/retro-boomer-64.png"));
            let (width, height) = pixels.dimensions();
            iced::widget::image::Handle::from_rgba(width, height, pixels.into_raw())
        })
        .clone()
}

pub fn icon() -> window::Icon {
    let icon = logo_pixels(APPLICATION_ICON);
    let (width, height) = icon.dimensions();
    window::icon::from_rgba(icon.into_raw(), width, height)
        .expect("embedded application icon dimensions are valid")
}

#[cfg(test)]
mod tests {
    #[test]
    fn in_app_logo_preserves_the_official_cassette_pixels() {
        let iced::widget::image::Handle::Rgba {
            width,
            height,
            pixels,
            ..
        } = super::logo_handle()
        else {
            panic!("logo must use decoded RGBA pixels")
        };
        assert_eq!((width, height), (1024, 1024));
        let p: &[u8] = pixels.as_ref();
        let expected = image::load_from_memory(include_bytes!("../assets/app-icons/retro-boomer-1024.png"))
            .unwrap()
            .into_rgba8();
        assert_eq!(p, expected.as_raw().as_slice());
    }

    #[test]
    fn shipped_retro_boomer_icons_have_correct_sizes_and_transparent_backgrounds() {
        let icons: [(u32, &[u8]); 9] = [
            (16, include_bytes!("../assets/app-icons/retro-boomer-16.png")),
            (24, include_bytes!("../assets/app-icons/retro-boomer-24.png")),
            (32, include_bytes!("../assets/app-icons/retro-boomer-32.png")),
            (48, include_bytes!("../assets/app-icons/retro-boomer-48.png")),
            (64, include_bytes!("../assets/app-icons/retro-boomer-64.png")),
            (128, include_bytes!("../assets/app-icons/retro-boomer-128.png")),
            (256, include_bytes!("../assets/app-icons/retro-boomer-256.png")),
            (512, include_bytes!("../assets/app-icons/retro-boomer-512.png")),
            (1024, include_bytes!("../assets/app-icons/retro-boomer-1024.png")),
        ];
        for (size, bytes) in icons {
            let im = image::load_from_memory(bytes).unwrap().into_rgba8();
            assert_eq!(im.dimensions(), (size, size));
            assert_eq!(im.get_pixel(0, 0).0[3], 0);
            assert!(im.pixels().any(|p| p.0[3] == 255));
            assert!(im.pixels().any(|p| p.0[3] == 0));
            assert!(im.pixels().any(|p| p.0[3] > 0 && p.0[3] < 255));
        }
        let ico = include_bytes!("../assets/app-icons/retro-boomer.ico");
        assert_eq!(&ico[..4], &[0, 0, 1, 0]);
        assert!(u16::from_le_bytes([ico[4], ico[5]]) > 0);
        let _decoded_window_icon = super::icon();
    }
}
