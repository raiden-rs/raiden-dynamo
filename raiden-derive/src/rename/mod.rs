use std::str::FromStr;

// "lowercase", "UPPERCASE", "PascalCase", "camelCase", "snake_case", "SCREAMING_SNAKE_CASE", "kebab-case", "SCREAMING-KEBAB-CASE".

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum RenameAllType {
    LowerCase,
    CamelCase,
    PascalCase,
    SnakeCase,
    ScreamingSnakeCase,
    KebabCase,
    None,
}

impl FromStr for RenameAllType {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, ()> {
        match s {
            "lowercase" => Ok(RenameAllType::LowerCase),
            "camelCase" => Ok(RenameAllType::CamelCase),
            "PascalCase" => Ok(RenameAllType::PascalCase),
            "snake_case" => Ok(RenameAllType::SnakeCase),
            "SCREAMING_SNAKE_CASE" => Ok(RenameAllType::ScreamingSnakeCase),
            "kebab-case" => Ok(RenameAllType::KebabCase),
            _ => panic!("{} is not support type.", s),
        }
    }
}

pub fn rename(t: RenameAllType, base: String) -> String {
    match t {
        crate::rename::RenameAllType::LowerCase => {
            ident_case::RenameRule::LowerCase.apply_to_field(base)
        }
        crate::rename::RenameAllType::CamelCase => {
            ident_case::RenameRule::CamelCase.apply_to_field(base)
        }
        crate::rename::RenameAllType::PascalCase => {
            ident_case::RenameRule::PascalCase.apply_to_field(base)
        }
        crate::rename::RenameAllType::SnakeCase => {
            ident_case::RenameRule::SnakeCase.apply_to_field(base)
        }
        crate::rename::RenameAllType::ScreamingSnakeCase => {
            ident_case::RenameRule::ScreamingSnakeCase.apply_to_field(base)
        }
        crate::rename::RenameAllType::KebabCase => {
            ident_case::RenameRule::KebabCase.apply_to_field(base)
        }
        _ => panic!("{} is not supported rename type"),
    }
}

pub fn create_renamed(
    basename: String,
    renamed: Option<String>,
    rename_all_type: RenameAllType,
) -> String {
    if renamed.is_none() {
        if rename_all_type != RenameAllType::None {
            format!("{}", rename(rename_all_type, basename))
        } else {
            basename
        }
    } else {
        renamed.unwrap()
    }
}
