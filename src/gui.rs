use iced::{
    executor,
    widget::{checkbox, column, row},
    window, Application, Element, Settings, Theme,
};

use crate::{
    base::{Todo, TodoInstance},
    util,
};

struct TodoApplication {
    pub instance: TodoInstance,
    pub states: Vec<TodoState>,
}

pub fn run() -> iced::Result {
    TodoApplication::run(Settings {
        window: window::Settings {
            size: (500, 800),
            ..window::Settings::default()
        },
        ..Settings::default()
    })
}

impl TodoApplication {
    pub fn get_state(&self, id: &u64) -> Option<&TodoState> {
        for state in &self.states {
            if state.id.eq(id) {
                return Option::Some(state);
            }
        }
        Option::None
    }

    pub fn get_state_mut(&mut self, id: &u64) -> Option<&mut TodoState> {
        for state in &mut self.states {
            if state.id.eq(id) {
                return Option::Some(state);
            }
        }
        Option::None
    }

    pub fn refresh_states(&mut self) {
        util::remove_from_vec_if(&mut self.states, &|state| {
            !self.instance.get_todos().contains(&state.id)
        });

        for todo in &self.instance.todos {
            if util::vec_none_match(&self.states, &|state| state.id == *todo.get_id()) {
                self.states.push(TodoState::new(todo));
            }
        }
    }
}

impl Application for TodoApplication {
    type Executor = executor::Default;

    type Message = Message;

    type Theme = Theme;

    type Flags = Flags;

    fn new(flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        let mut app = TodoApplication {
            instance: TodoInstance::create(&flags.path),
            states: Vec::new(),
        };
        app.instance.read_all();
        app.instance.refresh();
        app.refresh_states();
        (app, iced::Command::none())
    }

    fn title(&self) -> String {
        format!("Tuffous ({})", {
            let mut todos = 0;
            for todo_id in self.instance.get_todos() {
                if !self.instance.get(&todo_id).unwrap().completed {
                    todos += 1;
                }
            }
            todos
        })
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        match message {
            Message::TodoMessage(id, msg) => match msg {
                TodoMessage::Complete(c) => self.instance.get_mut(&id).unwrap().completed = c,
                TodoMessage::Edit(_) => todo!(),
            },
        };
        self.instance.write_all();
        self.refresh_states();
        iced::Command::none()
    }

    fn view(&self) -> iced::Element<Self::Message> {
        let mut todos = Vec::new();
        for todo_id in self.instance.get_todos() {
            todos.push((
                self.instance.get(&todo_id).unwrap(),
                self.get_state(&todo_id).unwrap(),
            ));
        }

        let todo_views: Element<_> = column(
            todos
                .iter()
                .enumerate()
                .map(|(i, state)| state.1.get_view(state.0))
                .collect(),
        )
        .spacing(15)
        .into();

        todo_views
    }
}

struct Flags {
    pub path: String,
}

impl Default for Flags {
    fn default() -> Self {
        Self {
            path: String::from("."),
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    TodoMessage(u64, TodoMessage),
}

#[derive(Debug, Clone)]
enum TodoMessage {
    Complete(bool),
    Edit(EditMessage),
}

#[derive(Debug, Clone)]
enum EditMessage {
    Edit,
    EndEdit,
}

struct TodoState {
    pub id: u64,
    pub editing: bool,
}

impl TodoState {
    pub fn new(todo: &Todo) -> Self {
        TodoState {
            id: *todo.get_id(),
            editing: false,
        }
    }

    pub fn get_view(&self, todo: &Todo) -> Element<Message> {
        row![checkbox(&todo.metadata.name, todo.completed, |x| {
            Message::TodoMessage(self.id.to_owned(), TodoMessage::Complete(x))
        })]
        .spacing(15)
        .into()
    }
}
