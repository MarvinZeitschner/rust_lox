#[derive(Debug, Clone)]
pub struct LoxClass<'a> {
    pub name: &'a str,
}

impl<'a> LoxClass<'a> {
    pub fn new(name: &'a str) -> Self {
        Self { name }
    }

    pub fn to_string(&self) -> String {
        self.name.to_string()
    }
}
