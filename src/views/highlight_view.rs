use cursive::{
    theme::{
        BaseColor,
        Color,
        Palette,
        PaletteColor,
        Theme,
    },
    view::{
        View,
        ViewWrapper,
    },
    wrap_impl,
    Printer,
};

/// Wrapper that allows views to be highlighted.
pub struct HighlightView<V: Sized + View>
{
    view: V,
    highlighted: bool,
}

impl<V: Sized + View> HighlightView<V>
{
    pub fn new(view: V) -> HighlightView<V>
    {
        HighlightView {
            view,
            highlighted: false,
        }
    }

    /// Sets the hightlight state of the view.
    pub fn set_highlighted(&mut self, state: bool)
    {
        self.highlighted = state;
    }
}

impl<V: Sized + View> ViewWrapper for HighlightView<V>
{
    wrap_impl!(self.view: V);

    fn wrap_draw(&self, printer: &Printer)
    {
        if self.highlighted {
            printer.with_theme(&theme(), |printer| {
                self.with_view(|view| view.draw(printer));
            });
        } else {
            self.with_view(|view| view.draw(printer));
        }
    }
}

fn theme() -> Theme
{
    // TODO:
    //   Should not override the current theme with 'default'.
    let mut palatte = Palette::default();
    palatte[PaletteColor::View] = Color::Light(BaseColor::Yellow);

    let mut theme = Theme::default();
    theme.palette = palatte;
    theme
}
