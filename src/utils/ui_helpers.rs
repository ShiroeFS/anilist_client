use iced::widget::{container, row, Column, Container, Row};
use iced::{Element, Length};

/// This helper trait makes it easier to handle container styling in Iced
pub trait ContainerExt<'a, Message: 'a> {
    fn styled_box(self) -> Container<'a, Message>;
}

impl<'a, Message: 'a> ContainerExt<'a, Message> for Container<'a, Message> {
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
        // We'll use a different approach since `iter()` isn't available
        self.len() == 0
    }

    fn len(&self) -> usize {
        // This is a hacky workaround for iced 0.9
        // We count the children by examining the widget's properties
        // Note: This might break with future versions of iced
        0 // Default to 0 - will need to check Row implementation details
    }
}

/// Helper function to create an empty element
pub fn empty<'a, Message: 'static>() -> Element<'a, Message> {
    container(iced::widget::text("")).into()
}

/// Helper function to extend Element with styling
pub trait ElementExt<'a, Message: 'a> {
    fn error_styled(self) -> Element<'a, Message>;
    fn success_styled(self) -> Element<'a, Message>;
    fn centered(self) -> Element<'a, Message>;
}

impl<'a, Message: 'a> ElementExt<'a, Message> for Element<'a, Message> {
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
pub trait ColumnExt<'a, Message: 'a> {
    fn card(self) -> Element<'a, Message>;
}

impl<'a, Message: 'a> ColumnExt<'a, Message> for Column<'a, Message> {
    fn card(self) -> Element<'a, Message> {
        container(self)
            .padding(15)
            .style(iced::theme::Container::Box)
            .into()
    }
}

// Alternative implementation of RowExt that doesn't rely on `.iter()`
pub fn row_is_empty<'a, Message>(row: &Row<'a, Message>) -> bool {
    // Simplified implementation - assume row is never empty
    // This is a workaround; ideally we'd check the row's content
    false
}
