use crate::backend::{self, Backend};
use crate::{Primitive, Vector};
use iced_native::layout;
use iced_native::renderer;
use iced_native::{Element, Font, Point, Rectangle, Size};

pub use iced_native::renderer::Style;

/// A backend-agnostic renderer that supports all the built-in widgets.
#[derive(Debug)]
pub struct Renderer<B: Backend> {
    backend: B,
    primitives: Vec<Primitive>,
}

impl<B: Backend> Renderer<B> {
    /// Creates a new [`Renderer`] from the given [`Backend`].
    pub fn new(backend: B) -> Self {
        Self {
            backend,
            primitives: Vec::new(),
        }
    }

    pub fn backend(&self) -> &B {
        &self.backend
    }

    pub fn present(&mut self, f: impl FnOnce(&mut B, &[Primitive])) {
        f(&mut self.backend, &self.primitives);
    }
}

impl<B> iced_native::Renderer for Renderer<B>
where
    B: Backend,
{
    fn layout<'a, Message>(
        &mut self,
        element: &Element<'a, Message, Self>,
        limits: &layout::Limits,
    ) -> layout::Node {
        let layout = element.layout(self, limits);

        self.backend.trim_measurements();

        layout
    }

    fn with_layer(
        &mut self,
        bounds: Rectangle,
        offset: Vector<u32>,
        f: impl FnOnce(&mut Self),
    ) {
        let current_primitives =
            std::mem::replace(&mut self.primitives, Vec::new());

        f(self);

        let layer_primitives =
            std::mem::replace(&mut self.primitives, current_primitives);

        self.primitives.push(Primitive::Clip {
            bounds,
            offset,
            content: Box::new(Primitive::Group {
                primitives: layer_primitives,
            }),
        });
    }

    fn fill_rectangle(&mut self, quad: renderer::Quad) {
        self.primitives.push(Primitive::Quad {
            bounds: quad.bounds,
            background: quad.background,
            border_radius: quad.border_radius,
            border_width: quad.border_width,
            border_color: quad.border_color,
        });
    }

    fn clear(&mut self) {
        self.primitives.clear();
    }
}

impl<B> renderer::Text for Renderer<B>
where
    B: Backend + backend::Text,
{
    type Font = Font;

    fn default_size(&self) -> u16 {
        self.backend().default_size()
    }

    fn measure(
        &self,
        content: &str,
        size: u16,
        font: Font,
        bounds: Size,
    ) -> (f32, f32) {
        self.backend()
            .measure(content, f32::from(size), font, bounds)
    }

    fn hit_test(
        &self,
        content: &str,
        size: f32,
        font: Font,
        bounds: Size,
        point: Point,
        nearest_only: bool,
    ) -> Option<renderer::text::Hit> {
        self.backend().hit_test(
            content,
            size,
            font,
            bounds,
            point,
            nearest_only,
        )
    }

    fn fill_text(&mut self, text: renderer::text::Section<'_, Self::Font>) {
        self.primitives.push(Primitive::Text {
            content: text.content.to_string(),
            bounds: text.bounds,
            size: text.size,
            color: text.color,
            font: text.font,
            horizontal_alignment: text.horizontal_alignment,
            vertical_alignment: text.vertical_alignment,
        });
    }
}
