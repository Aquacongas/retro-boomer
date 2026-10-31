use iced::widget::canvas;
use iced::{Point, Rectangle, Renderer, Size, Theme};

pub struct CopyIcon(pub bool);

impl<Message> canvas::Program<Message> for CopyIcon {
    type State = ();
    fn draw(
        &self,
        _: &(),
        renderer: &Renderer,
        _: &Theme,
        bounds: Rectangle,
        _: iced::mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let stroke = canvas::Stroke::default()
            .with_width(1.5)
            .with_color(crate::theme::ACCENT);
        if self.0 {
            let tick = canvas::Path::new(|p| {
                p.move_to(Point::new(3.0, 9.0));
                p.line_to(Point::new(7.0, 13.0));
                p.line_to(Point::new(14.0, 4.0));
            });
            frame.stroke(&tick, stroke);
        } else {
            frame.stroke(
                &canvas::Path::rectangle(Point::new(3.0, 2.0), Size::new(9.0, 10.0)),
                stroke,
            );
            frame.fill_rectangle(
                Point::new(6.0, 5.0),
                Size::new(9.0, 10.0),
                crate::theme::SURFACE,
            );
            frame.stroke(
                &canvas::Path::rectangle(Point::new(6.0, 5.0), Size::new(9.0, 10.0)),
                stroke,
            );
        }
        vec![frame.into_geometry()]
    }
}
