#[derive(Clone, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Problem {
    link: String,
    legend: String,
    name: String,
    author: String,
    #[serde(default)]
    date: chrono::DateTime<chrono::Utc>,
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

    pub fn set_editorial(&mut self, v: String) {
        self.editorial = v;
    }

    pub fn set_editorial_link(&mut self, v: String) {
        self.editorial_link = v;
    }

    pub fn to_sheet_row(&self) -> Vec<String> {
        vec![
            self.name.clone(),
            self.author.clone(),
            self.difficulty.clone(),
            self.date.format("%Y-%m-%d %H:%M:%S").to_string(),
            self.legend.clone(),
            self.editorial.clone(),
            self.tags.clone(),
            self.link.clone(),
            self.editorial_link.clone(),
        ]
    }

    pub fn from_message(
        link: String,
        message: String,
        author: String,
        date: chrono::DateTime<chrono::Utc>,
    ) -> Result<Self, String> {
        let (name, legend) = match message.split_once('\n') {
            Some((n, l)) => (n.trim().to_string(), l.trim().to_string()),
            None => (String::new(), message.trim().to_string()),
        };
        Ok(Self {
            link,
            legend,
            name,
            author,
            date,
            editorial: String::new(),
            editorial_link: String::new(),
            difficulty: String::new(),
            tags: String::new(),
        })
    }
}
