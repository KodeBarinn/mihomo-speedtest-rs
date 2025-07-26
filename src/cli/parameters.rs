use comfy_table::{Cell, Color, ContentArrangement, Table, presets::UTF8_FULL};
use std::time::Duration;

/// Represents a single parameter with its default and user values
#[derive(Debug, Clone)]
pub struct ParameterInfo {
    pub name: String,
    pub default_value: String,
    pub user_value: String,
    pub is_customized: bool,
    pub description: String,
}

impl ParameterInfo {
    pub fn new(name: &str, default_value: &str, user_value: &str, description: &str) -> Self {
        let is_customized = default_value != user_value;
        Self {
            name: name.to_string(),
            default_value: default_value.to_string(),
            user_value: user_value.to_string(),
            is_customized,
            description: description.to_string(),
        }
    }
}

/// Collects and manages parameter information for display
pub struct ParameterTable {
    parameters: Vec<ParameterInfo>,
}

impl ParameterTable {
    pub fn new() -> Self {
        Self {
            parameters: Vec::new(),
        }
    }

    /// Add a parameter to the table
    pub fn add_parameter(&mut self, info: ParameterInfo) {
        self.parameters.push(info);
    }

    /// Add a string parameter
    pub fn add_string_param(&mut self, name: &str, default: &str, user: &str, description: &str) {
        self.add_parameter(ParameterInfo::new(name, default, user, description));
    }

    /// Add a boolean parameter
    pub fn add_bool_param(&mut self, name: &str, default: bool, user: bool, description: &str) {
        self.add_parameter(ParameterInfo::new(
            name,
            &default.to_string(),
            &user.to_string(),
            description,
        ));
    }

    /// Add a numeric parameter
    pub fn add_numeric_param<T: ToString>(
        &mut self,
        name: &str,
        default: T,
        user: T,
        description: &str,
    ) {
        self.add_parameter(ParameterInfo::new(
            name,
            &default.to_string(),
            &user.to_string(),
            description,
        ));
    }

    /// Add a duration parameter
    pub fn add_duration_param(
        &mut self,
        name: &str,
        default: Duration,
        user: Duration,
        description: &str,
    ) {
        let format_duration = |d: Duration| {
            if d.as_millis() % 1000 == 0 {
                format!("{}s", d.as_secs())
            } else {
                format!("{}ms", d.as_millis())
            }
        };

        self.add_parameter(ParameterInfo::new(
            name,
            &format_duration(default),
            &format_duration(user),
            description,
        ));
    }

    /// Add an optional string parameter
    pub fn add_optional_string_param(
        &mut self,
        name: &str,
        default: Option<&str>,
        user: &Option<String>,
        description: &str,
    ) {
        let format_option = |opt: Option<&str>| match opt {
            Some(val) => val.to_string(),
            None => "None".to_string(),
        };

        let user_value = user.as_ref().map(|s| s.as_str());

        self.add_parameter(ParameterInfo::new(
            name,
            &format_option(default),
            &format_option(user_value),
            description,
        ));
    }

    /// Add an optional duration parameter
    pub fn add_optional_duration_param(
        &mut self,
        name: &str,
        default: Option<Duration>,
        user: Option<Duration>,
        description: &str,
    ) {
        let format_duration = |d: Duration| {
            if d.as_millis() % 1000 == 0 {
                format!("{}s", d.as_secs())
            } else {
                format!("{}ms", d.as_millis())
            }
        };

        let format_option = |opt: Option<Duration>| match opt {
            Some(val) => format_duration(val),
            None => "None".to_string(),
        };

        self.add_parameter(ParameterInfo::new(
            name,
            &format_option(default),
            &format_option(user),
            description,
        ));
    }

    /// Format the parameter table for display
    pub fn format_table(&self) -> String {
        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL)
            .set_content_arrangement(ContentArrangement::Dynamic);

        // Add header
        table.set_header(vec![
            Cell::new("Parameter").fg(Color::Blue),
            Cell::new("Default Value").fg(Color::Blue),
            Cell::new("Current Value").fg(Color::Blue),
            Cell::new("Customized").fg(Color::Blue),
            Cell::new("Description").fg(Color::Blue),
        ]);

        // Add parameter rows
        for param in &self.parameters {
            let customized_display = if param.is_customized { "Yes" } else { "No" };
            let customized_cell = if param.is_customized {
                Cell::new(customized_display).fg(Color::Green)
            } else {
                Cell::new(customized_display).fg(Color::DarkGrey)
            };

            let current_value_cell = if param.is_customized {
                Cell::new(&param.user_value).fg(Color::Yellow)
            } else {
                Cell::new(&param.user_value)
            };

            table.add_row(vec![
                Cell::new(&param.name),
                Cell::new(&param.default_value).fg(Color::DarkGrey),
                current_value_cell,
                customized_cell,
                Cell::new(&param.description).fg(Color::DarkGrey),
            ]);
        }

        table.to_string()
    }

    /// Get count of customized parameters
    pub fn customized_count(&self) -> usize {
        self.parameters.iter().filter(|p| p.is_customized).count()
    }

    /// Get total parameter count
    pub fn total_count(&self) -> usize {
        self.parameters.len()
    }
}

impl Default for ParameterTable {
    fn default() -> Self {
        Self::new()
    }
}
