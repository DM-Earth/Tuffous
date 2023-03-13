use chrono::{Datelike, Local};
use iced::{
    alignment, executor, theme,
    widget::{button, column, container, horizontal_space, row, scrollable, text, text_input},
    window, Application, Color, Element, Length, Renderer, Settings, Theme,
};

use crate::{
    base::{Todo, TodoInstance},
    util,
};

use super::appearance;

struct TodoApplication {
    pub instance: TodoInstance,
    pub states: Vec<TodoState>,
    pub dep_selection: Option<(u64, Vec<u64>)>,
}

pub fn run() -> iced::Result {
    TodoApplication::run(Settings {
        window: window::Settings {
            size: (650, 800),
            min_size: Option::Some((500, 650)),
            ..window::Settings::default()
        },
        default_font: Some(appearance::NOTO_SANS),
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
            dep_selection: Option::None,
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
                        {
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
                                self.dep_selection = Option::None;
                            }
                        }
                        if self.get_state(&id).unwrap().editing {
                            for state in &mut self.states {
                                if !state.id.eq(&id) {
                                    state.editing = false;
                                }
                            }
                        }

                        {
                            if !self.get_state(&id).unwrap().editing {
                                let mut todo = self.instance.get_mut(&id).unwrap();
                                if todo.metadata.name.is_empty() {
                                    todo.metadata.name = String::from("untitled todo");
                                }
                            }
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
                    EditMessage::Tags(tags) => {
                        let todo = self.instance.get_mut(&id).unwrap();
                        todo.tags.clear();
                        for tag in tags.split_whitespace() {
                            todo.tags.push(tag.to_string());
                        }
                    }
                    EditMessage::ToggleSelectChildren => {
                        if self.dep_selection.is_some() {
                            self.dep_selection = Option::None;
                        } else {
                            self.dep_selection =
                                Option::Some((id, self.instance.get_children_once(&id)));
                        }
                    }
                },
                TodoMessage::ExpandToggle => {
                    let state = self.get_state_mut(&id).unwrap();
                    state.expanded = !state.expanded;
                }
                TodoMessage::Delete => {
                    self.instance.remove(&id);
                    self.refresh_states();
                }
                TodoMessage::ToggleChild => {
                    if let Some((father_id, child_vec)) = &mut self.dep_selection {
                        if child_vec.contains(&id) {
                            util::remove_from_vec(
                                &mut self.instance.get_mut(&id).unwrap().dependents,
                                father_id,
                            );
                            util::remove_from_vec(child_vec, &id);
                        } else {
                            self.instance.child(father_id, &id);
                            child_vec.push(id);
                        }
                    }
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

        let todo_views: Element<_> = scrollable(
            column({
                let mut vec: Vec<Element<'_, Message, Renderer>> = Vec::new();
                for todo in &self.instance.todos {
                    if todo.dependents.is_empty() {
                        for view in &mut self.get_state(todo.get_id()).unwrap().get_view(self) {
                            let mut row_c: Vec<Element<'_, Message, Renderer>> = Vec::new();
                            row_c.push(horizontal_space(view.0).into());
                            row_c.append(&mut view.1);
                            vec.push(
                                container(container(row(row_c)).max_width(750))
                                    .align_x(alignment::Horizontal::Center)
                                    .width(Length::Fill)
                                    .into(),
                            );
                        }
                    }
                }
                vec
            })
            .spacing(7.5),
        )
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
    Delete,
    ToggleChild,
}

#[derive(Debug, Clone)]
enum EditMessage {
    Name(String),
    Details(String),
    Date(String),
    Deadline(String),
    Tags(String),
    ToggleEdit,
    ToggleSelectChildren,
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
        let height = 27.5;

        let todo = app.instance.get(&self.id).unwrap();
        let mut self_vec: Vec<Element<'_, Message, Renderer>> = Vec::new();
        if app.instance.get_children_once(&self.id).is_empty() {
            self_vec.push(horizontal_space(25).into());
        } else {
            self_vec.push(
                container(
                    button(
                        appearance::icon(if self.expanded { '' } else { '' })
                            .style(theme::Text::Color(Color::from_rgb(0.5, 0.5, 0.5))),
                    )
                    .width(20)
                    .style(theme::Button::Text)
                    .on_press(Message::TodoMessage(
                        self.id.to_owned(),
                        TodoMessage::ExpandToggle,
                    )),
                )
                .height(height)
                .center_y()
                .into(),
            );
            self_vec.push(horizontal_space(5).into());
        }

        self_vec.push(
            container(
                button(
                    appearance::icon(if todo.completed { '󰄲' } else { '󰄱' })
                        .size(17.5)
                        .style(theme::Text::Color(Color::from_rgb(0.0, 0.0, 0.8))),
                )
                .on_press(Message::TodoMessage(
                    self.id.to_owned(),
                    TodoMessage::ToggleComplete,
                ))
                .style(theme::Button::Text),
            )
            .height(height)
            .center_y()
            .into(),
        );

        let mut left_vec: Vec<Element<'_, Message, Renderer>> = Vec::new();
        let mut right_vec: Vec<Element<'_, Message, Renderer>> = Vec::new();

        if !self.editing {
            // Todo information
            if let Some(time) = &todo.time {
                left_vec.push(
                    container(if time.eq(&Local::now().date_naive()) {
                        appearance::icon('')
                            .style(theme::Text::Color(Color::from_rgb(1.0, 0.84, 0.0)))
                    } else {
                        text(format!(
                            " {}{} {} ",
                            if time.year() == Local::now().year() {
                                String::from("")
                            } else {
                                format!("{} ", time.year())
                            },
                            util::get_month_str(time.month()),
                            time.day()
                        ))
                        .size(18.5)
                    })
                    .style(if time.eq(&Local::now().date_naive()) {
                        theme::Container::Transparent
                    } else {
                        theme::Container::Box
                    })
                    .height(height)
                    .center_y()
                    .into(),
                );
                left_vec.push(horizontal_space(3.5).into());
            }

            left_vec.push(
                container(
                    button(text(&todo.metadata.name))
                        .style(theme::Button::Text)
                        .on_press(Message::TodoMessage(
                            self.id.to_owned(),
                            TodoMessage::Edit(EditMessage::ToggleEdit),
                        ))
                        .height(Length::Fill),
                )
                .height(height)
                .center_y()
                .into(),
            );

            if !todo.tags.is_empty() {
                left_vec.push(horizontal_space(3.5).into());
                for tag in &todo.tags {
                    left_vec.push(horizontal_space(5).into());
                    left_vec.push(
                        container(
                            container(text(format!("  {}  ", tag)).size(17.5))
                                .style(theme::Container::Custom(Box::new(appearance::TagStyle {})))
                                .height(height - 4.0)
                                .center_y(),
                        )
                        .center_y()
                        .height(height)
                        .into(),
                    );
                }
            }

            if let Some(ddl) = &todo.deadline {
                right_vec.push(
                    container(
                        text(format!(
                            "{} {} ",
                            if ddl.date().eq(&Local::now().date_naive()) {
                                String::from("Today")
                            } else {
                                format!(
                                    "{}{} {}",
                                    if ddl.year().eq(&Local::now().year()) {
                                        String::new()
                                    } else {
                                        format!("{} ", ddl.year())
                                    },
                                    util::get_month_str(ddl.month()),
                                    ddl.day()
                                )
                            },
                            ddl.time().format("%H:%M")
                        ))
                        .style(theme::Text::Color(Color::from_rgb(0.9, 0.0, 0.0))),
                    )
                    .height(height)
                    .center_y()
                    .into(),
                );
                right_vec.push(
                    container(
                        appearance::icon(if ddl > &Local::now().naive_local() {
                            '󰈻'
                        } else {
                            '󰮛'
                        })
                        .style(theme::Text::Color(Color::from_rgb(0.9, 0.0, 0.0))),
                    )
                    .height(height)
                    .center_y()
                    .into(),
                );
            }

            if let Some((father_id, child_vec)) = &app.dep_selection {
                if app.instance.child_able(father_id, &self.id) || child_vec.contains(&self.id) {
                    right_vec.push(horizontal_space(7.5).into());
                    right_vec.push(
                        container(
                            button(appearance::icon(if !child_vec.contains(&self.id) {
                                '󰝦'
                            } else {
                                '󰻃'
                            }))
                            .style(theme::Button::Text)
                            .on_press(Message::TodoMessage(self.id, TodoMessage::ToggleChild)),
                        )
                        .height(height)
                        .into(),
                    );
                }
            }

            right_vec.push(horizontal_space(12.5).into());
        } else {
            let mut col_vec: Vec<Element<'_, Message, Renderer>> = Vec::new();

            // Todo Editing
            col_vec.push(
                row!(
                    container(appearance::icon('󰑕')).height(height).center_y(),
                    container(
                        text_input("Input title here", &todo.metadata.name, |input| {
                            Message::TodoMessage(
                                self.id.to_owned(),
                                TodoMessage::Edit(EditMessage::Name(input)),
                            )
                        })
                        .width(350)
                    )
                    .height(height)
                    .center_y()
                )
                .into(),
            );
            col_vec.push(
                row!(
                    container(appearance::icon('󰟃')).height(height).center_y(),
                    container(
                        text_input("Input details here", &todo.metadata.details, |input| {
                            Message::TodoMessage(
                                self.id.to_owned(),
                                TodoMessage::Edit(EditMessage::Details(input)),
                            )
                        })
                        .width(350)
                    )
                    .height(height)
                    .center_y()
                )
                .into(),
            );
            col_vec.push(
                row!(
                    container(appearance::icon('󰃯')).height(height).center_y(),
                    container(
                        text_input("Input date here", &self.time_cache, |input| {
                            Message::TodoMessage(
                                self.id.to_owned(),
                                TodoMessage::Edit(EditMessage::Date(input)),
                            )
                        })
                        .width(350)
                    )
                    .height(height)
                    .center_y()
                )
                .into(),
            );
            col_vec.push(
                row!(
                    container(appearance::icon('󰈼')),
                    container(
                        text_input("Input deadline here", &self.ddl_cache, |input| {
                            Message::TodoMessage(
                                self.id.to_owned(),
                                TodoMessage::Edit(EditMessage::Deadline(input)),
                            )
                        })
                        .width(350)
                    )
                    .height(height)
                    .center_y()
                )
                .into(),
            );
            col_vec.push(
                row!(
                    container(appearance::icon('󰓻')).height(height).center_y(),
                    container(
                        text_input(
                            "Separate tags by space",
                            &format!("{} ", util::join_str_with(&todo.tags, " ")),
                            |input| {
                                Message::TodoMessage(
                                    self.id.to_owned(),
                                    TodoMessage::Edit(EditMessage::Tags(input)),
                                )
                            }
                        )
                        .width(350)
                    )
                    .height(height)
                    .center_y()
                )
                .into(),
            );

            self_vec.push(column(col_vec).into());

            right_vec.push(horizontal_space(8.5).into());

            // Right side controls for editing
            let mut controls_vec: Vec<Element<'_, Message, Renderer>> = Vec::new();
            controls_vec.push(
                container(
                    button(
                        appearance::icon('󰸞')
                            .style(theme::Text::Color(Color::from_rgb(0.65, 0.65, 0.65))),
                    )
                    .style(theme::Button::Text)
                    .on_press(Message::TodoMessage(
                        self.id.to_owned(),
                        TodoMessage::Edit(EditMessage::ToggleEdit),
                    )),
                )
                .height(height)
                .center_y()
                .into(),
            );

            controls_vec.push(
                container(
                    button(
                        appearance::icon(if app.dep_selection.is_some() {
                            '󱏒'
                        } else {
                            '󰙅'
                        })
                        .style(theme::Text::Color(Color::from_rgb(0.65, 0.65, 0.65))),
                    )
                    .style(theme::Button::Text)
                    .on_press(Message::TodoMessage(
                        self.id.to_owned(),
                        TodoMessage::Edit(EditMessage::ToggleSelectChildren),
                    )),
                )
                .height(height)
                .center_y()
                .into(),
            );
            controls_vec.push(
                container(
                    button(
                        appearance::icon('󰩹')
                            .style(theme::Text::Color(Color::from_rgb(0.65, 0.65, 0.65))),
                    )
                    .style(theme::Button::Text)
                    .on_press(Message::TodoMessage(
                        self.id.to_owned(),
                        TodoMessage::Delete,
                    )),
                )
                .height(height)
                .center_y()
                .into(),
            );

            right_vec.push(column(controls_vec).into());
        }

        right_vec.push(horizontal_space(12).into());

        self_vec.push(
            container(row(left_vec))
                .align_x(alignment::Horizontal::Left)
                .width(Length::Fill)
                .into(),
        );
        self_vec.push(
            container(row(right_vec))
                .align_x(alignment::Horizontal::Right)
                .width(Length::Fill)
                .into(),
        );

        let mut vec: Vec<(u16, Vec<Element<'_, Message, Renderer>>)> = Vec::new();
        vec.push((
            12,
            if self.editing {
                self_vec
            } else {
                vec![column!(
                    row(self_vec),
                    row![
                        horizontal_space(57.5),
                        text(todo.metadata.details.clone())
                            .style(theme::Text::Color(Color::from_rgb(0.35, 0.35, 0.35)))
                    ]
                )
                .into()]
            },
        ));
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