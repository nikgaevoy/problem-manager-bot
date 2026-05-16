#[derive(Clone, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Problem {
    link: String,
    legend: String,
    name: String,
    author: String,
    editorial: String,
    editorial_link: String,
}

impl Problem {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn to_sheet_row(&self) -> Vec<&str> {
        vec![&self.name, &self.link, &self.legend, &self.author, &self.editorial, &self.editorial_link]
    }

    pub fn from_message(link: String, message: String, author: String) -> Result<Self, String> {
        let (name, legend) = message.split_once('\n').ok_or("Empty legend, problem not loaded")?;
        let name = name.trim().to_string();
        let legend = legend.trim().to_string();
        Ok(Self { link, legend, name, author, editorial: String::new(), editorial_link: String::new() })
    }
}