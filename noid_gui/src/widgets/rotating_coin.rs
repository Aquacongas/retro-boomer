// SPDX-License-Identifier: Apache-2.0
// Copyright (C) 2026 Paranoid Zero.

use iced::widget::canvas;
use iced::{Point, Rectangle, Renderer, Theme, Vector};

#[derive(Clone)]
pub struct RotatingCoin {
    angle: f32,
    diameter: f32,
    depth: f32,
}

impl RotatingCoin {
    pub fn new(angle: f32, diameter: f32, depth: f32) -> Self {
        Self {
            angle,
            diameter,
            depth,
        }
    }
}

impl<Message> canvas::Program<Message> for RotatingCoin {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let center = Point::new(bounds.width * 0.5, bounds.height * 0.5);
        draw_coin(
            &mut frame,
            center,
            self.diameter.min(bounds.width).min(bounds.height),
            self.depth,
            self.angle,
        );
        vec![frame.into_geometry()]
    }
}

fn draw_coin(frame: &mut canvas::Frame, center: Point, diameter: f32, depth: f32, angle: f32) {
    let cosine = angle.cos();
    let sine = angle.sin();
    let face_scale = cosine.abs().max(0.018);

    // Preserve the existing rotation and front-face position, without extrusion.
    let visible_z = if cosine >= 0.0 { 0.5 } else { -0.5 };
    draw_symbol(
        frame,
        Point::new(center.x + visible_z * depth * sine, center.y),
        face_scale,
        diameter,
    );
}

fn draw_symbol(frame: &mut canvas::Frame, center: Point, face_scale: f32, diameter: f32) {
    frame.with_save(|frame| {
        frame.translate(Vector::new(center.x, center.y));
        frame.scale_nonuniform(Vector::new(face_scale, 1.0));
        frame.scale(diameter / 512.0);
        frame.translate(Vector::new(-256.0, -256.0));
        let gold = crate::theme::GOLD;
        frame.stroke(
            &canvas::Path::circle(Point::new(256.0, 256.0), 214.0),
            canvas::Stroke::default().with_width(36.0).with_color(gold),
        );
        // Legacy vector coin prototype with an open counter; not used for the official logo.
        let letter = canvas::Path::new(|p| {
            p.move_to(Point::new(184.0, 146.0));
            p.line_to(Point::new(256.0, 146.0));
            p.bezier_curve_to(
                Point::new(302.0, 146.0),
                Point::new(328.0, 172.0),
                Point::new(328.0, 213.0),
            );
            p.bezier_curve_to(
                Point::new(328.0, 254.0),
                Point::new(302.0, 280.0),
                Point::new(256.0, 280.0),
            );
            p.line_to(Point::new(220.0, 280.0));
            p.line_to(Point::new(220.0, 366.0));
            p.line_to(Point::new(184.0, 366.0));
            p.close();
            p.move_to(Point::new(220.0, 182.0));
            p.line_to(Point::new(220.0, 244.0));
            p.line_to(Point::new(256.0, 244.0));
            p.bezier_curve_to(
                Point::new(279.0, 244.0),
                Point::new(292.0, 233.0),
                Point::new(292.0, 213.0),
            );
            p.bezier_curve_to(
                Point::new(292.0, 193.0),
                Point::new(279.0, 182.0),
                Point::new(256.0, 182.0),
            );
            p.close();
        });
        frame.fill(
            &letter,
            canvas::Fill {
                style: gold.into(),
                rule: canvas::fill::Rule::EvenOdd,
            },
        );
    });
}
