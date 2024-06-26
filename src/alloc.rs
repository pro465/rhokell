const INIT: &[&'static str] = &["EOF", "byte", "input", "output"];
pub const EOF: Id = Id(0);
pub const BYTE: Id = Id(1);
pub const INPUT: Id = Id(2);
pub const OUTPUT: Id = Id(3);

#[derive(Clone, Debug, Default, Hash, PartialEq, Eq)]
pub struct Id(usize);

pub struct Alloc {
    ids: Vec<String>,
}

impl Alloc {
    pub fn new() -> Self {
        Self {
            ids: INIT.into_iter().map(ToString::to_string).collect(),
        }
    }
    pub fn alloc_or_get<'a>(&mut self, s: &'a str) -> Id {
        match self.ids.iter().position(|x| x == s) {
            Some(x) => Id(x),
            None => {
                self.ids.push(s.to_string());
                Id(self.ids.len() - 1)
            }
        }
    }
    pub fn get_string(&self, i: &Id) -> &str {
        &self.ids[i.0]
    }
}

pub trait DisplayWithAlloc {
    fn display(&self, alloc: &Alloc, s: &mut String);
    fn to_string(&self, alloc: &Alloc) -> String {
        let mut s = String::new();
        self.display(alloc, &mut s);
        s
    }
}
