// SPDX-License-Identifier: Apache-2.0
// Copyright (C) 2026 Paranoid Zero.

use iced::border::Radius;
use iced::widget::{
    button as button_widget, container, overlay::menu, pick_list, scrollable as scrollable_widget,
    text_editor as editor_widget, text_input as input_widget, toggler,
};
use iced::{
    font, theme::Palette, Background, Border, Color, Font, Shadow, Theme, Vector,
};

// RETRO BOOMER: neutral monochrome chrome and semantic status colors.
pub const BACKGROUND: Color = Color::from_rgb8(8, 8, 8); // #080808
pub const SURFACE: Color = Color::from_rgb8(18, 18, 18);
pub const SURFACE_ALT: Color = Color::from_rgb8(27, 27, 27);
pub const SURFACE_HIGH: Color = Color::from_rgb8(38, 38, 38);
pub const LINE: Color = Color::from_rgb8(102, 107, 112); // #666B70
pub const TEXT: Color = Color::from_rgb8(242, 242, 238); // #F2F2EE
pub const MUTED: Color = Color::from_rgb8(173, 176, 178);
pub const DIM: Color = MUTED;
pub const SUCCESS: Color = Color::from_rgb8(115, 201, 145); // #73C991
pub const WARNING: Color = Color::from_rgb8(226, 184, 107); // #E2B86B
pub const DANGER: Color = Color::from_rgb8(216, 107, 112); // #D86B70
// Compatibility aliases keep existing component styling without decorative hues.
pub const GOLD: Color = TEXT;
pub const LIGHT_GOLD: Color = TEXT;
pub const BUTTON_GOLD: Color = TEXT;
pub const CYAN: Color = TEXT;
pub const ACCENT: Color = TEXT;
pub const LINE_STRONG: Color = TEXT;
pub const HOVER: Color = Color::from_rgb8(255, 255, 255);
pub const PROOF: Color = TEXT;
pub const ADVISORY: Color = WARNING;
pub const INK: Color = BACKGROUND;
pub const CHROME: Color = SURFACE;
pub fn alpha(color: Color, a: f32) -> Color {
    Color { a, ..color }
}
pub fn mix(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    Color {
        r: a.r + (b.r - a.r) * t,
        g: a.g + (b.g - a.g) * t,
        b: a.b + (b.b - a.b) * t,
        a: a.a + (b.a - a.a) * t,
    }
}

pub const TECH_FONT: Font = Font {
    family: font::Family::Name("Noto Sans Mono"),
    weight: font::Weight::Normal,
    stretch: font::Stretch::Normal,
    style: font::Style::Normal,
};
pub const BRAND_FONT: Font = Font {
    family: font::Family::Name("Noto Sans"),
    weight: font::Weight::Bold,
    stretch: font::Stretch::Normal,
    style: font::Style::Normal,
};
pub const BRAND_REGULAR_FONT: Font = Font {
    family: font::Family::Name("Noto Sans"),
    weight: font::Weight::Normal,
    stretch: font::Stretch::Normal,
    style: font::Style::Normal,
};
pub const SYMBOL_FONT: Font = Font {
    family: font::Family::Name("Noto Sans Symbols"),
    weight: font::Weight::Bold,
    stretch: font::Stretch::Normal,
    style: font::Style::Normal,
};
pub const CJK_FONT: Font = Font {
    family: font::Family::Name("Noto Sans CJK SC"),
    weight: font::Weight::Normal,
    stretch: font::Stretch::Normal,
    style: font::Style::Normal,
};

fn soft_shadow() -> Shadow {
    Shadow {
        color: alpha(BACKGROUND, 0.20),
        offset: Vector::new(0.0, 2.0),
        blur_radius: 6.0,
    }
}

pub fn retro_boomer_theme() -> Theme {
    Theme::custom(
        "Retro Boomer Monochrome",
        Palette {
            background: BACKGROUND,
            text: TEXT,
            primary: ACCENT,
            success: SUCCESS,
            warning: WARNING,
            danger: DANGER,
        },
    )
}

pub fn root(_: &Theme) -> container::Style {
    container::Style::default()
        .background(
            BACKGROUND,
        )
        .color(TEXT)
}

pub fn language_background(_: &Theme) -> container::Style {
    container::Style::default()
        .background(
            BACKGROUND,
        )
        .color(TEXT)
}

pub fn surface(_: &Theme) -> container::Style {
    container::Style {
        text_color: Some(TEXT),
        background: Some(Background::Color(SURFACE)),
        border: Border {
            color: LINE,
            width: 1.0,
            radius: Radius::from(8.0),
        },
        shadow: soft_shadow(),
        snap: true,
    }
}

pub fn surface_alt(_: &Theme) -> container::Style {
    container::Style {
        text_color: Some(TEXT),
        background: Some(Background::Color(SURFACE_ALT)),
        border: Border {
            color: LINE,
            width: 1.0,
            radius: Radius::from(6.0),
        },
        shadow: Shadow::default(),
        snap: true,
    }
}

pub fn node_log_panel(_: &Theme) -> container::Style {
    container::Style {
        text_color: Some(TEXT),
        background: Some(Background::Color(SURFACE)),
        border: Border {
            color: Color { a: 0.42, ..CYAN },
            width: 1.0,
            radius: Radius::from(8.0),
        },
        shadow: soft_shadow(),
        snap: true,
    }
}

pub fn top_bar(_: &Theme) -> container::Style {
    container::Style {
        text_color: Some(TEXT),
        background: Some(Background::Color(SURFACE)),
        border: Border {
            color: alpha(LINE, 0.20),
            width: 1.0,
            radius: Radius::from(10.0),
        },
        shadow: Shadow {
            color: alpha(BACKGROUND, 0.48),
            offset: Vector::new(0.0, 5.0),
            blur_radius: 13.0,
        },
        snap: true,
    }
}

pub fn command_bar(_: &Theme) -> container::Style {
    container::Style {
        text_color: Some(TEXT),
        background: Some(Background::Color(CHROME)),
        border: Border {
            color: LINE_STRONG,
            width: 1.0,
            radius: Radius::from(8.0),
        },
        shadow: soft_shadow(),
        snap: true,
    }
}

pub fn status_panel(_: &Theme) -> container::Style {
    container::Style {
        text_color: Some(TEXT),
        background: Some(Background::Color(CHROME)),
        border: Border {
            color: alpha(CYAN, 0.18),
            width: 1.0,
            radius: Radius::from(8.0),
        },
        shadow: Shadow::default(),
        snap: true,
    }
}

pub fn secret_visual(_: &Theme) -> container::Style {
    container::Style {
        text_color: Some(TEXT),
        background: Some(Background::Color(alpha(SURFACE, 0.88))),
        border: Border {
            color: alpha(PROOF, 0.32),
            width: 1.0,
            radius: Radius::from(8.0),
        },
        shadow: soft_shadow(),
        snap: true,
    }
}

pub fn secret_key_token(_: &Theme) -> container::Style {
    container::Style {
        text_color: Some(TEXT),
        background: Some(Background::Color(SURFACE)),
        border: Border {
            color: alpha(ACCENT, 0.58),
            width: 1.0,
            radius: Radius::from(10.0),
        },
        shadow: Shadow {
            color: alpha(BACKGROUND, 0.48),
            offset: Vector::new(0.0, 5.0),
            blur_radius: 13.0,
        },
        snap: true,
    }
}

pub fn secret_address_token(depth: u8) -> container::Style {
    let lift = 3.0 - f32::from(depth).min(2.0) * 0.7;
    container::Style {
        text_color: Some(TEXT),
        background: Some(Background::Color(mix(
            SURFACE_ALT,
            LINE,
            f32::from(depth.min(2)) * 0.08,
        ))),
        border: Border {
            color: alpha(CYAN, 0.28),
            width: 1.0,
            radius: Radius::from(5.0),
        },
        shadow: Shadow {
            color: alpha(BACKGROUND, 0.42),
            offset: Vector::new(0.0, lift),
            blur_radius: 6.0,
        },
        snap: true,
    }
}

pub fn photo_frame(_: &Theme) -> container::Style {
    container::Style {
        text_color: Some(TEXT),
        background: Some(Background::Color(SURFACE)),
        border: Border {
            color: alpha(CYAN, 0.42),
            width: 1.0,
            radius: Radius::from(6.0),
        },
        shadow: Shadow::default(),
        snap: true,
    }
}

pub fn status_capsule(_: &Theme) -> container::Style {
    container::Style {
        text_color: Some(TEXT),
        background: Some(Background::Color(SURFACE)),
        border: Border {
            color: LINE,
            width: 1.0,
            radius: Radius::from(6.0),
        },
        shadow: Shadow::default(),
        snap: true,
    }
}

pub fn external_data_switch(_: &Theme, status: toggler::Status) -> toggler::Style {
    let (enabled, hovered, disabled) = match status {
        toggler::Status::Active { is_toggled } => (is_toggled, false, false),
        toggler::Status::Hovered { is_toggled } => (is_toggled, true, false),
        toggler::Status::Disabled { is_toggled } => (is_toggled, false, true),
    };
    let (top, _bottom) = if disabled {
        (SURFACE, SURFACE)
    } else if enabled {
        (ACCENT, ACCENT)
    } else {
        (SURFACE, SURFACE)
    };
    let edge = if disabled {
        LINE
    } else if enabled {
        Color { a: 0.8, ..ACCENT }
    } else if hovered {
        Color { a: 0.6, ..CYAN }
    } else {
        alpha(LINE, 0.3)
    };
    let knob_top = if disabled { DIM } else { TEXT };
    let _knob_bottom = if disabled {
        MUTED
    } else if hovered {
        MUTED
    } else {
        MUTED
    };
    toggler::Style {
        background: Background::Color(top),
        background_border_width: 1.0,
        background_border_color: edge,
        foreground: Background::Color(knob_top),
        foreground_border_width: 1.0,
        foreground_border_color: alpha(LINE, 0.42),
        text_color: Some(if disabled { DIM } else { MUTED }),
        border_radius: None,
        padding_ratio: 3.0 / 22.0,
    }
}

pub fn state_scale_tick(active: bool) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style::default().background(if active { ACCENT } else { alpha(LINE, 0.12) })
}

fn title_style(background: Color) -> container::Style {
    container::Style {
        text_color: Some(INK),
        background: Some(Background::Color(background)),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: Radius::from(4.0),
        },
        shadow: Shadow::default(),
        snap: true,
    }
}

pub fn title_bar_cyan(_: &Theme) -> container::Style {
    title_style(CYAN)
}

pub fn title_bar_proof(_: &Theme) -> container::Style {
    title_style(PROOF)
}

pub fn title_bar_accent(_: &Theme) -> container::Style {
    title_style(ACCENT)
}

pub fn table_header(_: &Theme) -> container::Style {
    container::Style {
        border: Border {
            radius: Radius::from(3.0),
            ..Border::default()
        },
        ..title_style(ACCENT)
    }
}

pub fn utxo_table_header(_: &Theme) -> container::Style {
    container::Style {
        text_color: Some(MUTED),
        background: Some(Background::Color(SURFACE_ALT)),
        border: Border {
            color: Color { a: 0.50, ..CYAN },
            width: 1.0,
            radius: Radius::from(3.0),
        },
        shadow: Shadow::default(),
        snap: true,
    }
}

pub fn scope_table_header(_: &Theme) -> container::Style {
    container::Style {
        text_color: Some(MUTED),
        background: Some(Background::Color(SURFACE_ALT)),
        border: Border {
            color: alpha(PROOF, 0.34),
            width: 1.0,
            radius: Radius::from(3.0),
        },
        shadow: Shadow::default(),
        snap: true,
    }
}

pub fn table_row(alternate: bool) -> impl Fn(&Theme) -> container::Style {
    move |_| {
        container::Style::default()
            .background(if alternate { SURFACE_ALT } else { SURFACE })
            .color(TEXT)
    }
}

pub fn transaction_row(alternate: bool, status: button_widget::Status) -> button_widget::Style {
    let hovered = matches!(
        status,
        button_widget::Status::Hovered | button_widget::Status::Pressed
    );
    button_widget::Style {
        background: Some(Background::Color(if hovered {
            SURFACE_HIGH
        } else if alternate {
            SURFACE_ALT
        } else {
            SURFACE
        })),
        text_color: TEXT,
        border: Border {
            color: if hovered {
                LINE_STRONG
            } else {
                Color::TRANSPARENT
            },
            width: if hovered { 1.0 } else { 0.0 },
            radius: Radius::from(2.0),
        },
        shadow: Shadow::default(),
        snap: true,
    }
}

pub fn utxo_row(
    alternate: bool,
    selected: bool,
    status: button_widget::Status,
) -> button_widget::Style {
    let hovered = matches!(
        status,
        button_widget::Status::Hovered | button_widget::Status::Pressed
    );
    let background = if selected {
        mix(SURFACE, CYAN, if hovered { 0.16 } else { 0.10 })
    } else if hovered {
        SURFACE_HIGH
    } else if alternate {
        SURFACE_ALT
    } else {
        SURFACE
    };

    button_widget::Style {
        background: Some(Background::Color(background)),
        text_color: TEXT,
        border: Border {
            color: if selected { CYAN } else { Color::TRANSPARENT },
            width: if selected { 1.0 } else { 0.0 },
            radius: Radius::from(2.0),
        },
        shadow: Shadow::default(),
        snap: true,
    }
}

pub fn divider(_: &Theme) -> container::Style {
    container::Style::default().background(LINE)
}

pub fn market_card(status: button_widget::Status) -> button_widget::Style {
    let hovered = matches!(
        status,
        button_widget::Status::Hovered | button_widget::Status::Pressed
    );
    button_widget::Style {
        background: Some(raised_gradient(if hovered {
            SURFACE_HIGH
        } else {
            SURFACE_ALT
        })),
        text_color: TEXT,
        border: Border {
            color: if hovered {
                Color { a: 0.6, ..CYAN }
            } else {
                LINE
            },
            width: 1.0,
            radius: Radius::from(8.0),
        },
        shadow: neutral_button_shadow(status, hovered),
        snap: true,
    }
}

pub fn scrollable(theme: &Theme, status: scrollable_widget::Status) -> scrollable_widget::Style {
    let mut style = scrollable_widget::default(theme, status);
    let active = matches!(
        status,
        scrollable_widget::Status::Hovered {
            is_vertical_scrollbar_hovered: true,
            ..
        } | scrollable_widget::Status::Dragged {
            is_vertical_scrollbar_dragged: true,
            ..
        }
    );

    style.vertical_rail.background = Some(Background::Color(alpha(
        CYAN,
        if active { 0.10 } else { 0.05 },
    )));
    style.vertical_rail.border = Border {
        color: Color::TRANSPARENT,
        width: 0.0,
        radius: Radius::from(99.0),
    };
    style.vertical_rail.scroller.background = Background::Color(Color {
        a: if active { 0.92 } else { 0.48 },
        ..CYAN
    });
    style.vertical_rail.scroller.border = Border {
        color: Color::TRANSPARENT,
        width: 0.0,
        radius: Radius::from(99.0),
    };
    style
}

pub fn status_dot(color: Color) -> impl Fn(&Theme) -> container::Style {
    move |_| {
        container::Style::default()
            .background(color)
            .border(Border {
                color,
                width: 0.0,
                radius: Radius::from(99.0),
            })
    }
}

pub fn advisory_badge(_: &Theme) -> container::Style {
    container::Style {
        text_color: Some(ADVISORY),
        background: Some(Background::Color(Color {
            a: 0.18,
            ..ADVISORY
        })),
        border: Border {
            color: Color {
                a: 0.76,
                ..ADVISORY
            },
            width: 1.25,
            radius: Radius::from(99.0),
        },
        shadow: Shadow::default(),
        snap: true,
    }
}

pub fn advisory_card(_: &Theme) -> container::Style {
    container::Style {
        text_color: Some(TEXT),
        background: Some(Background::Color(CHROME)),
        border: Border {
            color: Color {
                a: 0.68,
                ..ADVISORY
            },
            width: 1.0,
            radius: Radius::from(8.0),
        },
        shadow: soft_shadow(),
        snap: true,
    }
}

pub fn overlay(_: &Theme) -> container::Style {
    container::Style::default().background(alpha(BACKGROUND, 0.78))
}

pub fn proof_forge_overlay(_: &Theme) -> container::Style {
    container::Style {
        text_color: Some(TEXT),
        background: Some(Background::Color(alpha(SURFACE, 0.97))),
        border: Border {
            color: alpha(PROOF, 0.42),
            width: 1.0,
            radius: Radius::from(8.0),
        },
        shadow: Shadow {
            color: alpha(PROOF, 0.16),
            offset: Vector::new(0.0, 0.0),
            blur_radius: 14.0,
        },
        snap: true,
    }
}

pub fn shutdown_forge_overlay(_: &Theme) -> container::Style {
    container::Style {
        text_color: Some(TEXT),
        background: Some(Background::Color(alpha(BACKGROUND, 0.94))),
        border: Border {
            color: alpha(PROOF, 0.34),
            width: 1.0,
            radius: Radius::from(0.0),
        },
        shadow: Shadow::default(),
        snap: true,
    }
}

pub fn key_cap(active: bool) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        text_color: Some(if active { INK } else { TEXT }),
        background: Some(Background::Color(if active { ACCENT } else { SURFACE_ALT })),
        border: Border {
            color: LINE,
            width: 1.0,
            radius: Radius::from(3.0),
        },
        shadow: Shadow::default(),
        snap: true,
    }
}

pub fn text_input(_: &Theme, status: input_widget::Status) -> input_widget::Style {
    let focused = matches!(status, input_widget::Status::Focused { .. });
    let hovered = matches!(status, input_widget::Status::Hovered);

    input_widget::Style {
        background: Background::Color(SURFACE),
        border: Border {
            color: if focused {
                CYAN
            } else if hovered {
                LINE_STRONG
            } else {
                LINE
            },
            width: 1.0,
            radius: Radius::from(6.0),
        },
        icon: MUTED,
        placeholder: DIM,
        value: TEXT,
        selection: alpha(CYAN, 0.28),
    }
}

pub fn scope_search_input(_: &Theme, status: input_widget::Status) -> input_widget::Style {
    let focused = matches!(status, input_widget::Status::Focused { .. });
    let hovered = matches!(status, input_widget::Status::Hovered);

    input_widget::Style {
        background: Background::Color(SURFACE),
        border: Border {
            color: Color {
                a: if focused {
                    1.0
                } else if hovered {
                    0.82
                } else {
                    0.64
                },
                ..ACCENT
            },
            width: if focused { 2.0 } else { 1.5 },
            radius: Radius::from(6.0),
        },
        icon: ACCENT,
        placeholder: MUTED,
        value: TEXT,
        selection: alpha(ACCENT, 0.28),
    }
}

pub fn text_editor(_: &Theme, status: editor_widget::Status) -> editor_widget::Style {
    let focused = matches!(status, editor_widget::Status::Focused { .. });
    let hovered = matches!(status, editor_widget::Status::Hovered);

    editor_widget::Style {
        background: Background::Color(SURFACE),
        border: Border {
            color: if focused {
                CYAN
            } else if hovered {
                LINE_STRONG
            } else {
                LINE
            },
            width: 1.0,
            radius: Radius::from(6.0),
        },
        placeholder: DIM,
        value: TEXT,
        selection: alpha(CYAN, 0.28),
    }
}

pub fn node_log_editor(_: &Theme, status: editor_widget::Status) -> editor_widget::Style {
    let focused = matches!(status, editor_widget::Status::Focused { .. });
    let hovered = matches!(status, editor_widget::Status::Hovered);

    editor_widget::Style {
        background: Background::Color(SURFACE),
        border: Border {
            color: if focused {
                ACCENT
            } else if hovered {
                Color { a: 0.72, ..CYAN }
            } else {
                Color { a: 0.34, ..CYAN }
            },
            width: if focused { 1.5 } else { 1.0 },
            radius: Radius::from(5.0),
        },
        placeholder: DIM,
        value: MUTED,
        selection: alpha(CYAN, 0.32),
    }
}

pub fn selectable_address(_: &Theme, _: input_widget::Status) -> input_widget::Style {
    input_widget::Style {
        background: Background::Color(Color::TRANSPARENT),
        border: Border::default(),
        icon: MUTED,
        placeholder: DIM,
        value: TEXT,
        selection: alpha(CYAN, 0.34),
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ButtonKind {
    Primary,
    Secondary,
    Ghost,
    Command,
    CommandActive,
}

fn primary_button_shadow(status: button_widget::Status) -> Shadow {
    if matches!(status, button_widget::Status::Pressed) {
        Shadow {
            color: alpha(BACKGROUND, 0.44),
            offset: Vector::new(0.0, 1.5),
            blur_radius: 3.0,
        }
    } else {
        Shadow {
            color: alpha(BACKGROUND, 0.58),
            offset: Vector::new(0.0, 4.0),
            blur_radius: 8.0,
        }
    }
}

pub fn colored_primary(color: Color, status: button_widget::Status) -> button_widget::Style {
    if matches!(status, button_widget::Status::Disabled) {
        return disabled_button_style();
    }

    let hovered = matches!(
        status,
        button_widget::Status::Hovered | button_widget::Status::Pressed
    );
    let pressed = matches!(status, button_widget::Status::Pressed);
    let base = if hovered { HOVER } else { ACCENT };

    button_widget::Style {
        background: Some(raised_gradient(base)),
        text_color: INK,
        border: Border {
            color: Color {
                a: 0.38,
                ..tint(color, 0.68)
            },
            width: if pressed { 1.0 } else { 1.15 },
            radius: Radius::from(6.0),
        },
        shadow: primary_button_shadow(status),
        snap: true,
    }
}

/// The first-run language controls use the ordinary primary-button language,
/// with a firmer contact shadow to match the forged blocks above them.
pub fn language_choice(color: Color, status: button_widget::Status) -> button_widget::Style {
    let mut style = colored_primary(color, status);
    let pressed = matches!(status, button_widget::Status::Pressed);
    style.shadow = Shadow {
        color: alpha(BACKGROUND, if pressed { 0.44 } else { 0.72 }),
        offset: Vector::new(0.0, if pressed { 2.0 } else { 7.0 }),
        blur_radius: if pressed { 3.0 } else { 12.0 },
    };
    style.border.width = if pressed { 1.0 } else { 1.25 };
    style
}

fn raised_gradient(color: Color) -> Background {
    Background::Color(color)
}

fn tint(color: Color, amount: f32) -> Color {
    mix(color, SURFACE, amount)
}
fn shade(color: Color, amount: f32) -> Color {
    mix(color, BACKGROUND, amount)
}

fn disabled_button_style() -> button_widget::Style {
    button_widget::Style {
        background: Some(Background::Color(SURFACE_ALT)),
        text_color: DIM,
        border: Border {
            color: LINE,
            width: 1.0,
            radius: Radius::from(6.0),
        },
        shadow: Shadow {
            color: alpha(BACKGROUND, 0.18),
            offset: Vector::new(0.0, 2.0),
            blur_radius: 4.0,
        },
        snap: true,
    }
}

fn neutral_button_shadow(status: button_widget::Status, visible: bool) -> Shadow {
    let pressed = matches!(status, button_widget::Status::Pressed);
    if !visible && !pressed {
        return Shadow::default();
    }

    Shadow {
        color: alpha(BACKGROUND, if pressed { 0.30 } else { 0.40 }),
        offset: Vector::new(0.0, if pressed { 1.0 } else { 3.0 }),
        blur_radius: if pressed { 2.0 } else { 7.0 },
    }
}

pub fn asset_picker(status: pick_list::Status) -> pick_list::Style {
    let opened = matches!(status, pick_list::Status::Opened { .. });
    let hovered = matches!(status, pick_list::Status::Hovered);
    let base = if opened {
        SURFACE
    } else if hovered {
        SURFACE
    } else {
        SURFACE
    };

    pick_list::Style {
        text_color: TEXT,
        placeholder_color: DIM,
        handle_color: if opened || hovered { CYAN } else { MUTED },
        background: raised_gradient(base),
        border: Border {
            color: if opened {
                Color { a: 0.72, ..CYAN }
            } else if hovered {
                Color { a: 0.46, ..CYAN }
            } else {
                alpha(LINE, 0.28)
            },
            width: 1.0,
            radius: Radius::from(7.0),
        },
    }
}

pub fn asset_picker_frame() -> container::Style {
    container::Style {
        border: Border {
            radius: Radius::from(7.0),
            ..Border::default()
        },
        shadow: Shadow {
            color: alpha(BACKGROUND, 0.46),
            offset: Vector::new(0.0, 3.0),
            blur_radius: 7.0,
        },
        ..container::Style::default()
    }
}

pub fn asset_picker_menu() -> menu::Style {
    menu::Style {
        background: raised_gradient(SURFACE),
        border: Border {
            color: alpha(CYAN, 0.40),
            width: 1.0,
            radius: Radius::from(7.0),
        },
        text_color: MUTED,
        selected_text_color: TEXT,
        selected_background: Background::Color(mix(SURFACE, CYAN, 0.12)),
        shadow: Shadow {
            color: alpha(BACKGROUND, 0.68),
            offset: Vector::new(0.0, 8.0),
            blur_radius: 18.0,
        },
    }
}

pub fn asset_picker_locked() -> container::Style {
    let style = asset_picker(pick_list::Status::Active);
    container::Style {
        text_color: Some(DIM),
        background: Some(style.background),
        border: Border {
            color: alpha(LINE, 0.14),
            ..style.border
        },
        ..asset_picker_frame()
    }
}

pub fn button(kind: ButtonKind, status: button_widget::Status) -> button_widget::Style {
    if matches!(status, button_widget::Status::Disabled) && !matches!(kind, ButtonKind::Ghost) {
        return disabled_button_style();
    }

    let hovered = matches!(
        status,
        button_widget::Status::Hovered | button_widget::Status::Pressed
    );
    let pressed = matches!(status, button_widget::Status::Pressed);

    match kind {
        ButtonKind::Primary => colored_primary(ACCENT, status),
        ButtonKind::Secondary => {
            let base = if pressed {
                shade(SURFACE_ALT, 0.14)
            } else if hovered {
                tint(SURFACE_HIGH, 0.06)
            } else {
                SURFACE_ALT
            };
            button_widget::Style {
                background: Some(raised_gradient(base)),
                text_color: TEXT,
                border: Border {
                    color: if hovered {
                        Color {
                            a: 0.42,
                            ..LINE_STRONG
                        }
                    } else {
                        Color { a: 0.22, ..LINE }
                    },
                    width: 1.0,
                    radius: Radius::from(6.0),
                },
                shadow: neutral_button_shadow(status, true),
                snap: true,
            }
        }
        ButtonKind::Ghost => {
            let disabled = matches!(status, button_widget::Status::Disabled);
            // Keep text readable on both dark panels and light dialog title bars.
            let base = if pressed {
                mix(BACKGROUND, LINE, 0.20)
            } else if hovered {
                mix(BACKGROUND, LINE, 0.10)
            } else {
                BACKGROUND
            };
            button_widget::Style {
                background: Some(raised_gradient(base)),
                text_color: if disabled { mix(TEXT, LINE, 0.25) } else { TEXT },
                border: Border {
                    color: if hovered { TEXT } else { LINE },
                    width: 1.0,
                    radius: Radius::from(5.0),
                },
                shadow: neutral_button_shadow(status, hovered),
                snap: true,
            }
        }
        ButtonKind::Command => {
            let base = if pressed {
                alpha(CYAN, 0.34)
            } else if hovered {
                alpha(CYAN, 0.30)
            } else {
                alpha(SURFACE, 0.18)
            };
            button_widget::Style {
                background: Some(raised_gradient(base)),
                text_color: TEXT,
                border: Border {
                    color: if hovered {
                        alpha(CYAN, 0.22)
                    } else {
                        alpha(LINE, 0.05)
                    },
                    width: 1.0,
                    radius: Radius::from(5.0),
                },
                shadow: neutral_button_shadow(status, hovered),
                snap: true,
            }
        }
        ButtonKind::CommandActive => {
            let base = if hovered { HOVER } else { ACCENT };
            button_widget::Style {
                background: Some(raised_gradient(base)),
                text_color: INK,
                border: Border {
                    color: alpha(CYAN, 0.38),
                    width: 1.0,
                    radius: Radius::from(5.0),
                },
                shadow: neutral_button_shadow(status, true),
                snap: true,
            }
        }
    }
}

pub fn consolidation_button(status: button_widget::Status) -> button_widget::Style {
    if matches!(status, button_widget::Status::Disabled) {
        return button(ButtonKind::Secondary, status);
    }

    let hovered = matches!(
        status,
        button_widget::Status::Hovered | button_widget::Status::Pressed
    );
    let pressed = matches!(status, button_widget::Status::Pressed);

    let base = if pressed {
        Color {
            a: 0.36,
            ..ADVISORY
        }
    } else if hovered {
        Color {
            a: 0.29,
            ..ADVISORY
        }
    } else {
        Color {
            a: 0.20,
            ..ADVISORY
        }
    };

    button_widget::Style {
        background: Some(raised_gradient(base)),
        text_color: ADVISORY,
        border: Border {
            color: Color {
                a: if hovered { 0.95 } else { 0.72 },
                ..ADVISORY
            },
            width: if hovered { 1.5 } else { 1.25 },
            radius: Radius::from(6.0),
        },
        shadow: neutral_button_shadow(status, true),
        snap: true,
    }
}

#[cfg(test)]
mod palette_tests {
    use super::*;
    fn luminance(c: Color) -> f32 {
        let f = |x: f32| {
            if x <= 0.04045 {
                x / 12.92
            } else {
                ((x + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * f(c.r) + 0.7152 * f(c.g) + 0.0722 * f(c.b)
    }
    fn ratio(a: Color, b: Color) -> f32 {
        let (a, b) = (luminance(a), luminance(b));
        (a.max(b) + 0.05) / (a.min(b) + 0.05)
    }
    fn check_text(fg: Color, bg: Background) {
        let colors = match bg {
            Background::Color(c) => vec![c],
            Background::Gradient(iced::Gradient::Linear(g)) => {
                g.stops.into_iter().flatten().map(|x| x.color).collect()
            }
        };
        for c in colors {
            let c = mix(SURFACE, Color { a: 1.0, ..c }, c.a);
            assert!(
                ratio(fg, c) >= 4.5,
                "contrast {:.2}:1 {:?} {:?}",
                ratio(fg, c),
                fg,
                c
            );
        }
    }
    #[test]
    fn small_text_is_readable_on_retro_boomer_surfaces() {
        for bg in [BACKGROUND, SURFACE, SURFACE_ALT] {
            for fg in [TEXT, MUTED, DIM, ACCENT, CYAN, PROOF, WARNING, DANGER] {
                check_text(fg, bg.into());
            }
        }
    }
    #[test]
    fn button_states_keep_readable_text() {
        for status in [
            button_widget::Status::Active,
            button_widget::Status::Hovered,
            button_widget::Status::Pressed,
            button_widget::Status::Disabled,
        ] {
            for kind in [
                ButtonKind::Primary,
                ButtonKind::Secondary,
                ButtonKind::Ghost,
                ButtonKind::Command,
                ButtonKind::CommandActive,
            ] {
                let s = button(kind, status);
                check_text(s.text_color, s.background.unwrap());
            }
        }
    }
}
