use iced::widget::{container, row, Column, Container, Row};
use iced::{Element, Length};

/// This helper trait makes it easier to handle container styling in Iced
pub trait ContainerExt<'a, Message> {
    fn styled_box(self) -> Container<'a, Message>;
}

impl<'a, Message> ContainerExt<'a, Message> for Container<'a, Message> {
    fn styled_box(self) -> Container<'a, Message> {
        self.style(iced::theme::Container::Box)
    }
}

/// Extension trait for Row to check if it's empty
pub trait RowExt {
    fn is_empty(&self) -> bool;
    fn len(&self) -> usize;
}

impl<'a, Message> RowExt for Row<'a, Message> {
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn len(&self) -> usize {
        // In Iced 0.9, we don't have direct access to children count
        // This is a workaround that will always work,
        // though not super efficient
        let mut count = 0;
        for _ in self.iter() {
            count += 1;
        }
        count
    }
}

/// Helper function to create an empty element
pub fn empty<Message>() -> Element<'static, Message> {
    container(iced::widget::text("")).into()
}

/// Helper function to extend Element with styling
pub trait ElementExt<'a, Message> {
    fn error_styled(self) -> Element<'a, Message>;
    fn success_styled(self) -> Element<'a, Message>;
    fn centered(self) -> Element<'a, Message>;
}

impl<'a, Message> ElementExt<'a, Message> for Element<'a, Message> {
    fn error_styled(self) -> Element<'a, Message> {
        container(self)
            .padding(10)
            .style(iced::theme::Container::Box)
            .into()
    }

    fn success_styled(self) -> Element<'a, Message> {
        container(self)
            .padding(10)
            .style(iced::theme::Container::Box)
            .into()
    }

    fn centered(self) -> Element<'a, Message> {
        container(self)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}

/// Extension trait for Column to make common patterns easier
pub trait ColumnExt<'a, Message> {
    fn card(self) -> Element<'a, Message>;
}

impl<'a, Message> ColumnExt<'a, Message> for Column<'a, Message> {
    fn card(self) -> Element<'a, Message> {
        container(self)
            .padding(15)
            .style(iced::theme::Container::Box)
            .into()
    }
}
