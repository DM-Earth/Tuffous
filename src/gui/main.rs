use chrono::{Datelike, Local};
use iced::{
    executor, theme,
    widget::{button, checkbox, column, horizontal_space, row, text, text_input},
    window, Application, Color, Element, Renderer, Settings, Theme,
};

use crate::{
    base::{Todo, TodoInstance},
    util,
};

use super::icons;

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
        self.states.iter().find(|&state| state.id.eq(id))
    }

    pub fn get_state_mut(&mut self, id: &u64) -> Option<&mut TodoState> {
        self.states.iter_mut().find(|state| state.id.eq(id))
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
        self.instance.refresh();
        match message {
            Message::TodoMessage(id, msg) => match msg {
                TodoMessage::ToggleComplete => {
                    let mut todo = self.instance.get_mut(&id).unwrap();
                    todo.completed = !todo.completed;
                }
                TodoMessage::Edit(edit_msg) => match edit_msg {
                    EditMessage::Name(name) => {
                        self.instance.get_mut(&id).unwrap().metadata.name = name
                    }
                    EditMessage::Details(details) => {
                        self.instance.get_mut(&id).unwrap().metadata.details = details
                    }
                    EditMessage::ToggleEdit => {
                        let todo = self.instance.get(&id).unwrap();
                        let time_o = todo.time.clone();
                        let ddl_o = todo.deadline.clone();
                        let mut state = self.get_state_mut(&id).unwrap();
                        state.editing = !state.editing;
                        if state.editing {
                            if let Some(time) = time_o {
                                state.time_cache = time.format("%Y/%m/%d").to_string();
                            }

                            if let Some(ddl) = ddl_o {
                                state.ddl_cache = ddl.format("%Y/%m/%d-%H:%M:%S").to_string();
                            }
                        } else {
                            state.time_cache = String::new();
                            state.ddl_cache = String::new();
                        }
                    }
                    EditMessage::Date(date) => {
                        if let Some(date_r) = util::parse_date(&date) {
                            self.instance.get_mut(&id).unwrap().time = Option::Some(date_r);
                        } else {
                            self.instance.get_mut(&id).unwrap().time = Option::None;
                        }
                        self.get_state_mut(&id).unwrap().time_cache = date;
                    }
                    EditMessage::Deadline(ddl) => {
                        if let Some(ddl_r) = util::parse_date_and_time(&ddl) {
                            self.instance.get_mut(&id).unwrap().deadline = Option::Some(ddl_r);
                        } else {
                            self.instance.get_mut(&id).unwrap().deadline = Option::None;
                        }
                        self.get_state_mut(&id).unwrap().ddl_cache = ddl;
                    }
                },
                TodoMessage::ExpandToggle => {
                    let state = self.get_state_mut(&id).unwrap();
                    state.expanded = !state.expanded;
                }
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

        let todo_views: Element<_> = column({
            let mut vec: Vec<Element<'_, Message, Renderer>> = Vec::new();
            for todo in &self.instance.todos {
                if todo.dependents.is_empty() {
                    for view in &mut self.get_state(todo.get_id()).unwrap().get_view(self) {
                        let mut row_c: Vec<Element<'_, Message, Renderer>> = Vec::new();
                        row_c.push(horizontal_space(view.0).into());
                        row_c.append(&mut view.1);
                        vec.push(row(row_c).into());
                    }
                }
            }
            vec
        })
        .spacing(7.5)
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
    ToggleComplete,
    Edit(EditMessage),
    ExpandToggle,
}

#[derive(Debug, Clone)]
enum EditMessage {
    Name(String),
    Details(String),
    Date(String),
    Deadline(String),
    ToggleEdit,
}

struct TodoState {
    pub id: u64,
    pub editing: bool,
    pub expanded: bool,
    pub time_cache: String,
    pub ddl_cache: String,
}

impl TodoState {
    pub fn new(todo: &Todo) -> Self {
        TodoState {
            id: *todo.get_id(),
            editing: false,
            expanded: true,
            time_cache: String::new(),
            ddl_cache: String::new(),
        }
    }

    pub fn get_view<'a>(
        &'a self,
        app: &'a TodoApplication,
    ) -> Vec<(u16, Vec<Element<'_, Message, Renderer>>)> {
        let todo = app.instance.get(&self.id).unwrap();
        let mut self_vec: Vec<Element<'_, Message, Renderer>> = Vec::new();
        if app.instance.get_children_once(&self.id).is_empty() {
            self_vec.push(horizontal_space(20).into());
        } else {
            self_vec.push(
                button(icons::icon(if self.expanded { '' } else { '' }))
                    .width(20)
                    .style(theme::Button::Text)
                    .on_press(Message::TodoMessage(
                        self.id.to_owned(),
                        TodoMessage::ExpandToggle,
                    ))
                    .into(),
            );
            self_vec.push(horizontal_space(5).into());
        }

        self_vec.push(
            button(
                icons::icon(if todo.completed { '󰄲' } else { '󰄱' })
                    .style(theme::Text::Color(Color::from_rgb(0.0, 0.0, 0.8))),
            )
            .on_press(Message::TodoMessage(
                self.id.to_owned(),
                TodoMessage::ToggleComplete,
            ))
            .style(theme::Button::Text)
            .into(),
        );
        if !self.editing {
            self_vec.push(text(&todo.metadata.name).into());
            if let Some(time) = &todo.time {
                self_vec.push(horizontal_space(7.5).into());
                if time.eq(&Local::now().date_naive()) {
                    self_vec.push(
                        icons::icon('')
                            .style(theme::Text::Color(Color::from_rgb(1.0, 0.84, 0.0)))
                            .into(),
                    );
                } else {
                    self_vec.push(
                        text(format!(
                            "{}{} {}",
                            if time.year() == Local::now().year() {
                                String::from("")
                            } else {
                                format!("{} ", time.year())
                            },
                            util::get_month_str(time.month()),
                            time.day()
                        ))
                        .size(18.5)
                        .style(theme::Text::Color(Color::from_rgb(0.45, 0.45, 0.45)))
                        .into(),
                    );
                }
            }

            if let Some(ddl) = &todo.deadline {
                self_vec.push(horizontal_space(50).into());
                self_vec.push(
                    icons::icon(if ddl > &Local::now().naive_local() {
                        '󰈽'
                    } else {
                        '󰈻'
                    })
                    .style(theme::Text::Color(Color::from_rgb(0.9, 0.0, 0.0)))
                    .into(),
                );
                self_vec.push(
                    text(format!(
                        " {} {}",
                        if ddl.date().eq(&Local::now().date_naive()) {
                            String::from("Today")
                        } else {
                            format!("{} {}", util::get_month_str(ddl.month()), ddl.day())
                        },
                        ddl.time().format("%H:%M")
                    ))
                    .style(theme::Text::Color(Color::from_rgb(0.9, 0.0, 0.0)))
                    .into(),
                );
            }
        } else {
            let mut col_vec: Vec<Element<'_, Message, Renderer>> = Vec::new();

            col_vec.push(
                row!(
                    icons::icon('󰑕'),
                    text_input("Input title here", &todo.metadata.name, |input| {
                        Message::TodoMessage(
                            self.id.to_owned(),
                            TodoMessage::Edit(EditMessage::Name(input)),
                        )
                    })
                )
                .into(),
            );
            col_vec.push(
                row!(
                    icons::icon('󰟃'),
                    text_input("Input details here", &todo.metadata.details, |input| {
                        Message::TodoMessage(
                            self.id.to_owned(),
                            TodoMessage::Edit(EditMessage::Details(input)),
                        )
                    })
                )
                .into(),
            );
            col_vec.push(
                row!(
                    icons::icon('󰃯'),
                    text_input("Input date here", &self.time_cache, |input| {
                        Message::TodoMessage(
                            self.id.to_owned(),
                            TodoMessage::Edit(EditMessage::Date(input)),
                        )
                    })
                )
                .into(),
            );
            col_vec.push(
                row!(
                    icons::icon('󰈼'),
                    text_input("Input deadline here", &self.ddl_cache, |input| {
                        Message::TodoMessage(
                            self.id.to_owned(),
                            TodoMessage::Edit(EditMessage::Deadline(input)),
                        )
                    })
                )
                .into(),
            );

            self_vec.push(column(col_vec).width(350).into());
        }

        self_vec.push(horizontal_space(15).into());
        self_vec.push(
            button(icons::icon('󰏫'))
                .style(theme::Button::Text)
                .on_press(Message::TodoMessage(
                    self.id.to_owned(),
                    TodoMessage::Edit(EditMessage::ToggleEdit),
                ))
                .into(),
        );

        let mut vec: Vec<(u16, Vec<Element<'_, Message, Renderer>>)> = Vec::new();
        vec.push((0, self_vec));
        if self.expanded {
            for todo_id in app.instance.get_children_once(&self.id) {
                for v in app.get_state(&todo_id).unwrap().get_view(app) {
                    vec.push((v.0 + 25, v.1));
                }
            }
        }
        vec
    }
}
