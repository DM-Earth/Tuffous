use std::{
    collections::hash_map::DefaultHasher,
    fs::{self, File},
    hash::{Hash, Hasher},
    io::{Read, Write},
    path::Path,
};

use chrono::{Local, NaiveDate, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};

pub fn init_repo(path: &str) {
    create_path(&format!("{path}/.todo"));
    create_path(&format!("{path}/.todo/todos"));
}

fn create_path(path: &str) {
    if let Err(x) = fs::create_dir(format!("{path}")) {
        if x.to_string().contains("File exists") {
        } else {
            panic!("Error when initializing todo repo: {}", x)
        }
    };
}

#[derive(Serialize, Deserialize)]
pub struct Todo {
    id: u64,
    pub completed: bool,
    creation_date: NaiveDateTime,
    pub deadline: Option<NaiveDateTime>,
    pub time: Option<NaiveDate>,
    pub dependents: Vec<u64>,
    pub tags: Vec<String>,
    pub weight: u32,
    pub metadata: TodoMetaData,
}

#[derive(Serialize, Deserialize)]
pub struct TodoMetaData {
    pub details: String,
    pub name: String,
}

impl Todo {
    pub fn get_id(&self) -> &u64 {
        &self.id
    }

    fn create_id(name: &String, time: &NaiveDateTime) -> u64 {
        calculate_hash(&format!("{}{}", calculate_hash(name), calculate_hash(time)))
    }

    pub fn create(name: String) -> Todo {
        let time = Utc::now().naive_utc();
        Todo {
            id: Self::create_id(&name, &time),
            completed: false,
            creation_date: time,
            deadline: Option::None,
            time: Option::None,
            dependents: Vec::new(),
            tags: Vec::new(),
            weight: 1,
            metadata: TodoMetaData {
                name,
                details: String::new(),
            },
        }
    }

    pub fn get_creation_date(&self) -> &NaiveDateTime {
        &self.creation_date
    }

    pub fn write_to_file(&self, path: &str) {
        let p = format!("{path}/.todo/todos/{}.json", self.get_id());

        let Ok(mut file) = File::create(&p) else { return };
        file.write(serde_json::to_string(self).unwrap().as_bytes())
            .unwrap();
    }

    pub fn read_from_file<P: AsRef<Path>>(path: P) -> Option<Self> {
        let mut file;
        if let Ok(f) = File::open(path) {
            file = f
        } else {
            return Option::None;
        };

        let mut str = String::new();
        if let Err(_) = file.read_to_string(&mut str) {
            return Option::None;
        };

        if let Ok(x) = serde_json::from_str::<Todo>(&str) {
            Option::Some(x)
        } else {
            Option::None
        }
    }
}

impl PartialEq for Todo {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Clone for Todo {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            completed: self.completed.clone(),
            creation_date: self.creation_date.clone(),
            deadline: self.deadline.clone(),
            time: self.time.clone(),
            dependents: self.dependents.clone(),
            tags: self.tags.clone(),
            weight: self.weight.clone(),
            metadata: self.metadata.clone(),
        }
    }
}

impl Clone for TodoMetaData {
    fn clone(&self) -> Self {
        Self {
            details: self.details.clone(),
            name: self.name.clone(),
        }
    }
}

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

pub struct TodoInstance {
    pub todos: Vec<Todo>,
    path: String,
}

impl TodoInstance {
    pub fn create(path: &str) -> TodoInstance {
        TodoInstance {
            todos: Vec::new(),
            path: path.to_string(),
        }
    }

    pub fn read_all(&mut self) {
        for f in fs::read_dir(format!("{}/.todo/todos", self.path)).unwrap() {
            if let Ok(_) = f {
                if let Some(x) = Todo::read_from_file(f.unwrap().path()) {
                    self.todos.push(x)
                } else {
                    continue;
                }
            } else {
                continue;
            }
        }
    }

    pub fn write_all(&self) {
        for todo in &self.todos {
            todo.write_to_file(&self.path);
        }
    }

    pub fn get(&self, id: &u64) -> Option<&Todo> {
        for todo in &self.todos {
            if todo.get_id().eq(id) {
                return Option::Some(todo);
            }
        }
        Option::None
    }

    pub fn get_mut(&mut self, id: &u64) -> Option<&mut Todo> {
        for todo in &mut self.todos {
            if todo.get_id().eq(id) {
                return Option::Some(todo);
            }
        }
        Option::None
    }

    pub fn get_todos(&self) -> Vec<u64> {
        let mut vec = Vec::new();
        for todo in &self.todos {
            vec.push(*todo.get_id());
        }
        vec
    }

    pub fn get_children(&self, id: &u64) -> Vec<u64> {
        let mut vec = Vec::new();
        for todo in &self.todos {
            if self.get_all_deps(&todo.id).contains(id) {
                vec.push(*todo.get_id());
            }
        }
        vec
    }

    pub fn get_children_once(&self, id: &u64) -> Vec<u64> {
        let mut vec = Vec::new();
        for todo in &self.todos {
            if todo.dependents.contains(id) {
                vec.push(*todo.get_id());
            }
        }
        vec
    }

    pub fn child_able(&self, father: &u64, child: &u64) -> bool {
        if father == child {
            return false;
        }
        !(self.get_all_deps(father).contains(child) || self.get_children(father).contains(child))
    }

    pub fn replace(&mut self, replacement: Todo) -> bool {
        if !&self.todos.contains(&replacement) {
            return false;
        }
        for t in self.todos.iter().enumerate() {
            if t.1.eq(&replacement) {
                self.todos.remove(t.0);
                self.todos.push(replacement);
                break;
            }
        }
        true
    }

    pub fn child(&mut self, father: &u64, child: &u64) {
        if !self.child_able(father, child) {
            panic!("Can't child the target child")
        }

        let mut target = self.get(child).unwrap().clone();
        if !target.dependents.contains(father) {
            target.dependents.push(*father);
            self.replace(target);
        }
    }

    pub fn get_all_deps(&self, id: &u64) -> Vec<u64> {
        let mut vec = Vec::new();
        if let Some(target) = self.get(id) {
            for dep in &target.dependents {
                if let Some(todo) = self.get(dep) {
                    vec.push(*todo.get_id());
                    for t in self.get_all_deps(dep) {
                        vec.push(t);
                    }
                }
            }
            vec
        } else {
            vec
        }
    }

    pub fn refresh(&mut self) {
        let todos = self.get_todos();
        for todo_id in &todos {
            if let Some(todo) = self.get_mut(todo_id) {
                // Remove broken deps
                loop {
                    let mut rm = 0;
                    let mut remove = false;
                    for dep in todo.dependents.iter().enumerate() {
                        if !todos.contains(&dep.1) {
                            rm = dep.0;
                            remove = true;
                            break;
                        }
                    }

                    if remove {
                        todo.dependents.remove(rm);
                    } else {
                        break;
                    }
                }

                // Correct time
                if let Some(date) = &todo.time {
                    if date < &Local::now().date_naive() {
                        todo.time = Some(Local::now().date_naive());
                    }
                }
            }
        }
    }

    pub fn remove(&mut self, id: &u64) {
        let mut rm = 0;
        let mut remove = false;

        for todo in self.todos.iter().enumerate() {
            if todo.1.get_id().eq(id) {
                rm = todo.0;
                remove = true;
            }
        }
        if remove {
            self.todos.remove(rm);
        }
        self.refresh();
        fs::remove_file(format!("{}/.todo/todos/{}.json", self.path, id)).unwrap();
    }

    pub fn get_weight(&self, id: &u64, completed: bool) -> u32 {
        let mut base = 0;
        if self.get_children_once(id).is_empty() && (self.get(id).unwrap().completed || !completed)
        {
            base += self.get(id).unwrap().weight
        }

        for child in self.get_children_once(id) {
            if self.get(&child).unwrap().completed || !completed {
                base += self.get_weight(&child, completed);
            }
        }

        base
    }
}
