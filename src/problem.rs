#[derive(Clone, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Problem {
    link: String,
    legend: String,
    name: String,
}

impl Problem {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn to_sheet_row(&self) -> Vec<&str> {
        vec![&self.name, &self.link, &self.legend]
    }

    pub fn from_message(link: String, message: String) -> Result<Self, String> {
        let (name, legend) = message.split_once('\n').ok_or("Empty legend, problem not loaded")?;
        let name = name.trim().to_string();
        let legend = legend.trim().to_string();
        Ok(Self { link, legend, name })
    }
}