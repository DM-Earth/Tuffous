use iced::{executor, Application, Theme};

use crate::base::TodoInstance;

pub struct TodoApplication {
    pub instance: TodoInstance,
}

impl Application for TodoApplication {
    type Executor = executor::Default;

    type Message = Message;

    type Theme = Theme;

    type Flags = Flags;

    fn new(flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        todo!()
    }

    fn title(&self) -> String {
        String::from("Tuffous")
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        todo!()
    }

    fn view(&self) -> iced::Element<'_, Self::Message, iced::Renderer<Self::Theme>> {
        todo!()
    }
}

pub struct Flags {
    pub path: String,
}

#[derive(Debug)]
pub struct Message {}
