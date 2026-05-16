#[derive(Clone, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Problem {
    link: String,
    legend: String,
    name: String,
    author: String,
    editorial: String,
    editorial_link: String,
    difficulty: String,
    tags: String,
}

impl Problem {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn link(&self) -> &str {
        &self.link
    }

    pub fn author(&self) -> &str {
        &self.author
    }

    pub fn set_difficulty(&mut self, v: String) {
        self.difficulty = v;
    }

    pub fn set_tags(&mut self, v: String) {
        self.tags = v;
    }

    pub fn to_sheet_row(&self) -> Vec<&str> {
        vec![&self.name, &self.link, &self.legend, &self.author, &self.editorial, &self.editorial_link, &self.difficulty, &self.tags]
    }

    pub fn from_message(link: String, message: String, author: String) -> Result<Self, String> {
        let (name, legend) = message.split_once('\n').ok_or("Empty legend, problem not loaded")?;
        let name = name.trim().to_string();
        let legend = legend.trim().to_string();
        Ok(Self { link, legend, name, author, editorial: String::new(), editorial_link: String::new(), difficulty: String::new(), tags: String::new() })
    }
}