// SPDX-License-Identifier: Apache-2.0
// Original Mining layout from Parano1d 1051452100db39a3d44899fec16bc8c838ddbad5.
use super::copy_value_button;
use crate::app::{App, BackendState, Message};
use crate::i18n::{address_label, text};
use crate::model::MinedBlockSnapshot;
use crate::theme::{self, ButtonKind};
use iced::widget::{button, column, container, row, scrollable};
use iced::{Alignment, Element, Length, Padding};

pub fn view(app: &App, compact: bool) -> Element<'_, Message> {
    let controls: Element<'_, Message> = if compact {
        column![miner_status(app), miner_controls(app)]
            .spacing(10)
            .into()
    } else {
        row![
            miner_status(app)
                .width(Length::FillPortion(7))
                .height(Length::Fill),
            miner_controls(app)
                .width(Length::FillPortion(5))
                .height(Length::Fill),
        ]
        .spacing(10)
        .height(Length::Fixed(160.0))
        .into()
    };

    let mut page = column![controls].spacing(10);
    if let Some(error) = &app.mining_error {
        page = page.push(
            container(
                column![
                    text("NODE ERROR").size(13).color(theme::DANGER),
                    text(error).size(13).color(theme::MUTED),
                ]
                .spacing(5),
            )
            .padding([10, 12])
            .width(Length::Fill)
            .style(theme::surface),
        );
    }
    page = page.push(mined_blocks(app)).height(Length::Fill);

    let content = container(page).padding(Padding::ZERO.right(10));
    let content: Element<'_, Message> = if compact {
        scrollable(content).style(theme::scrollable).into()
    } else {
        content.into()
    };
    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(12)
        .into()
}

fn miner_status(app: &App) -> iced::widget::Container<'_, Message> {
    let address = app
        .mining
        .payout_address
        .as_deref()
        .and_then(|payout| {
            app.snapshot
                .addresses
                .iter()
                .find(|address| address.address == payout)
        })
        .unwrap_or_else(|| app.snapshot.active_address());
    let (status, status_color) = if app.mining_action_in_flight {
        ("UPDATING MINER", theme::WARNING)
    } else if app.mining.node_mining {
        ("NODE MINER RUNNING", theme::WARNING)
    } else if app.mining.enabled && app.mining.isolated {
        ("ISOLATED", theme::WARNING)
    } else if app.mining.enabled && app.mining.ready {
        ("WORKER RUNNING", theme::WARNING)
    } else if app.mining.enabled {
        ("WAITING FOR PEERS", theme::WARNING)
    } else {
        ("STOPPED", theme::DIM)
    };
    let connection = match app.backend_state {
        BackendState::Online => "LOCAL NODE ONLINE",
        BackendState::Starting => "LOCAL NODE STARTING",
        BackendState::Offline => "LOCAL NODE OFFLINE",
        BackendState::Mock => "DESIGN PREVIEW",
    };

    container(
        column![
            container(
                row![
                    text("MINER STATUS").size(13).color(theme::PROOF),
                    iced::widget::Space::new().width(Length::Fill),
                    text(format!("[{status}]")).size(13).color(status_color),
                ]
                .align_y(Alignment::Center),
            )
            .padding([5, 9])
            .style(theme::surface_alt),
            container(
                column![
                    detail(
                        "PAYOUT",
                        format!("[{}] {}", address.key_index, address_label(&address.label))
                    ),
                    row![
                        text(&address.address)
                            .size(13)
                            .color(theme::TEXT)
                            .wrapping(iced::widget::text::Wrapping::WordOrGlyph),
                        copy_value_button(
                            &address.address,
                            app.copied_value.as_deref() == Some(address.address.as_str()),
                        ),
                        iced::widget::Space::new().width(Length::Fill),
                    ]
                    .spacing(6)
                    .align_y(Alignment::Center),
                    divider(),
                    row![
                        detail("BACKEND", app.snapshot.network.backend.clone()),
                        iced::widget::Space::new().width(Length::Fill),
                        detail(
                            "THREADS",
                            format!(
                                "{}/{}",
                                app.mining.selected_threads, app.mining.available_threads
                            ),
                        ),
                    ]
                    .spacing(12),
                    row![
                        detail(
                            "PEERS",
                            format!(
                                "{}/{}",
                                app.mining.confirmed_peers, app.mining.required_peers
                            )
                        ),
                        iced::widget::Space::new().width(Length::Fill),
                        text("·").size(12).color(theme::DIM),
                        text(connection).size(12).color(theme::DIM),
                    ]
                    .spacing(6)
                    .align_y(Alignment::Center),
                ]
                .spacing(7),
            )
            .padding([9, 12]),
        ]
        .spacing(0),
    )
    .width(Length::Fill)
    .style(theme::surface)
}

fn miner_controls(app: &App) -> iced::widget::Container<'_, Message> {
    let can_edit_threads = !app.mining.enabled && !app.mining_action_in_flight;
    let mut decrement = button(text("−").size(17))
        .padding([5, 10])
        .style(|_, status| theme::button(ButtonKind::Secondary, status));
    let mut increment = button(text("+").size(17))
        .padding([5, 10])
        .style(|_, status| theme::button(ButtonKind::Secondary, status));
    if can_edit_threads && app.mining.selected_threads > 1 {
        decrement = decrement.on_press(Message::AdjustMiningThreads(-1));
    }
    if can_edit_threads && app.mining.selected_threads < app.mining.available_threads {
        increment = increment.on_press(Message::AdjustMiningThreads(1));
    }

    let thread_control = row![
        decrement,
        container(
            column![
                text(app.mining.selected_threads.to_string())
                    .size(22)
                    .color(if can_edit_threads {
                        theme::TEXT
                    } else {
                        theme::DIM
                    }),
                text("CPU THREADS").size(12).color(theme::DIM),
            ]
            .spacing(1)
            .align_x(Alignment::Center),
        )
        .width(Length::Fill)
        .align_x(Alignment::Center),
        increment,
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    let genesis_control: Element<'_, Message> = row![
        text("NETWORK READINESS").size(12).color(theme::DIM),
        text("API: automatic").size(13).color(theme::MUTED),
    ]
    .spacing(5)
    .align_y(Alignment::Center)
    .into();
    let mining_enabled = app.mining.enabled;
    let label = if app.mining_action_in_flight {
        "UPDATING…"
    } else if mining_enabled {
        "STOP MINING"
    } else {
        "START MINING"
    };
    let mut toggle = button(
        text(label)
            .size(13)
            .color(
                if app.mining_action_in_flight
                    || (!mining_enabled
                        && (app.backend_state != BackendState::Online || app.mining.node_mining))
                {
                    theme::TEXT
                } else {
                    theme::INK
                },
            )
            .width(Length::Fill)
            .align_x(Alignment::Center),
    )
    .width(Length::Fill)
    .height(32)
    .padding([7, 12])
    .style(|_, status| mining_toggle_style(status));
    if !app.mining_action_in_flight
        && (mining_enabled
            || (app.backend_state == BackendState::Online && !app.mining.node_mining))
    {
        toggle = toggle.on_press(Message::SetMining(!mining_enabled));
    }

    container(
        column![
            container(text("MINER CONTROL").size(13).color(theme::CYAN)).padding([5, 9]),
            container(column![thread_control, divider(), genesis_control, toggle].spacing(6),)
                .padding([7, 12]),
        ]
        .spacing(0),
    )
    .width(Length::Fill)
    .style(theme::surface)
}

fn mined_blocks(app: &App) -> iced::widget::Container<'_, Message> {
    let history = &app.mining.history;
    let page_count = history.total_pages.max(1);
    let title = row![
        text("MINED BLOCKS").size(13).color(theme::CYAN),
        text(format!("[{}]", history.total))
            .size(13)
            .color(theme::MUTED),
        iced::widget::Space::new().width(Length::Fill),
        legend("FULL BLOCK", theme::ACCENT),
        legend("HEADER", theme::DIM),
    ]
    .spacing(9)
    .align_y(Alignment::Center);

    let header = container(
        row![
            table_cell("HEIGHT".into(), 2, theme::INK),
            table_cell("FOUND".into(), 3, theme::INK),
            table_cell("REWARD".into(), 4, theme::INK),
            table_cell("PAYOUT".into(), 3, theme::INK),
            table_cell("BLOCK".into(), 5, theme::INK),
            table_cell("AVAILABLE".into(), 3, theme::INK),
            text("OPEN")
                .size(13)
                .color(theme::INK)
                .width(Length::Fixed(92.0)),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    )
    .padding([7, 9])
    .style(theme::table_header);

    let rows: Element<'_, Message> = if history.blocks.is_empty() {
        container(
            column![
                text("NO LOCALLY RECORDED MINED BLOCKS")
                    .size(13)
                    .color(theme::MUTED),
                text("Start the miner; accepted coinbase blocks will appear here.")
                    .size(13)
                    .color(theme::DIM),
            ]
            .spacing(5)
            .align_x(Alignment::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Alignment::Center)
        .align_y(Alignment::Center)
        .into()
    } else {
        let mut rows = column![].spacing(0);
        for (index, block) in history.blocks.iter().enumerate() {
            rows = rows.push(mined_block_row(app, block, index % 2 == 1));
        }
        scrollable(rows)
            .height(Length::Fill)
            .style(theme::scrollable)
            .into()
    };

    let mut previous = button(text("← PREV").size(13))
        .padding([7, 11])
        .style(|_, status| theme::button(ButtonKind::Secondary, status));
    if app.mining_page > 1 {
        previous = previous.on_press(Message::PreviousMiningPage);
    }
    let mut next = button(text("NEXT →").size(13))
        .padding([7, 11])
        .style(|_, status| theme::button(ButtonKind::Secondary, status));
    if app.mining_page < history.total_pages {
        next = next.on_press(Message::NextMiningPage);
    }
    let footer = row![
        previous,
        text(format!("PAGE {} / {page_count}", app.mining_page))
            .size(13)
            .color(theme::MUTED),
        next,
        iced::widget::Space::new().width(Length::Fill),
        text("AVAILABILITY REPORTED BY THE NODE")
            .size(12)
            .color(theme::DIM),
    ]
    .spacing(10)
    .align_y(Alignment::Center);

    container(
        column![
            container(title).padding([7, 10]),
            header,
            rows,
            container(footer).padding([8, 10]),
        ]
        .spacing(0),
    )
    .height(Length::Fill)
    .width(Length::Fill)
    .style(theme::surface)
}

fn mined_block_row<'a>(
    app: &App,
    block: &'a MinedBlockSnapshot,
    alternate: bool,
) -> Element<'a, Message> {
    let available = if !block.canonical {
        ("ORPHANED".into(), theme::WARNING)
    } else if block.full_block_available {
        (
            format!("FULL · {} conf", block.confirmations),
            theme::ACCENT,
        )
    } else {
        (format!("HEADER · {} conf", block.confirmations), theme::DIM)
    };
    let payout_label = app
        .snapshot
        .addresses
        .iter()
        .find(|address| address.key_index == block.payout_key_index)
        .map(|address| address_label(&address.label))
        .unwrap_or_else(|| "Address".into());
    let mut open = button(text(if block.full_block_available {
        "DETAILS"
    } else {
        "HEADER"
    }))
    .width(Length::Fixed(92.0))
    .padding([6, 8])
    .style(|_, status| theme::button(ButtonKind::Ghost, status));
    if !app.block_details_loading {
        open = open.on_press(Message::OpenBlockDetails(block.height));
    }

    container(
        row![
            table_cell(block.height.to_string(), 2, theme::CYAN),
            table_cell(format_age(block.timestamp), 3, theme::MUTED),
            table_cell(format!("{} TAPE", block.reward()), 4, theme::TEXT),
            table_cell(
                format!("[{}] {payout_label}", block.payout_key_index),
                3,
                theme::PROOF,
            ),
            table_cell(block.short_hash(), 5, theme::MUTED),
            table_cell(available.0, 3, available.1),
            open,
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    )
    .padding([6, 9])
    .style(theme::table_row(alternate))
    .into()
}

fn table_cell(value: String, portion: u16, color: iced::Color) -> Element<'static, Message> {
    text(value)
        .size(13)
        .color(color)
        .wrapping(iced::widget::text::Wrapping::None)
        .width(Length::FillPortion(portion))
        .into()
}

fn legend(label: &'static str, color: iced::Color) -> Element<'static, Message> {
    row![
        text("■").size(12).color(color),
        text(label).size(12).color(theme::MUTED),
    ]
    .spacing(5)
    .align_y(Alignment::Center)
    .into()
}

fn detail(label: &'static str, value: String) -> Element<'static, Message> {
    row![
        text(label).size(12).color(theme::DIM),
        text(format!("[{value}]")).size(13).color(theme::CYAN),
    ]
    .spacing(6)
    .align_y(Alignment::Center)
    .into()
}

fn divider() -> Element<'static, Message> {
    container(iced::widget::Space::new())
        .width(Length::Fill)
        .height(1)
        .style(theme::divider)
        .into()
}

fn format_age(timestamp: u64) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let seconds = now.saturating_sub(timestamp);
    match seconds {
        0..=59 => format!("{seconds}s ago"),
        60..=3_599 => format!("{}m ago", seconds / 60),
        3_600..=86_399 => format!("{}h ago", seconds / 3_600),
        _ => format!("{}d ago", seconds / 86_400),
    }
}

// Local to the mining toggle: do not alter primary/disabled buttons elsewhere.
fn mining_toggle_style(status: iced::widget::button::Status) -> iced::widget::button::Style {
    let mut style = theme::button(ButtonKind::Primary, status);
    style.background = Some(iced::Background::Color(
        if status == iced::widget::button::Status::Disabled {
            theme::SURFACE_ALT
        } else {
            theme::ACCENT
        },
    ));
    style.text_color = if status == iced::widget::button::Status::Disabled {
        theme::TEXT
    } else {
        theme::INK
    };
    style.border.color = theme::LINE;
    style.border.width = 1.0;
    style
}
