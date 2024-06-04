use std::fmt::Debug;

macro_rules! impl_save_load {
    ($struct_name:ident, $path:expr) => {
        impl $struct_name {
            pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
                let mut file = std::fs::File::create($path)?;
                let ron_string =
                    ron::ser::to_string_pretty(&self, ron::ser::PrettyConfig::default())?;
                std::io::Write::write_all(&mut file, ron_string.as_bytes())?;
                Ok(())
            }

            pub fn load() -> Option<Self> {
                let mut state_file = match std::fs::File::open($path) {
                    Ok(file) => file,
                    Err(_) => return None,
                };
                let mut buf: String = String::new();
                if let Err(e) = std::io::Read::read_to_string(&mut state_file, &mut buf) {
                    println!("Error reading file: {:?}", e);
                    return None;
                }
                match ron::from_str::<Self>(&buf) {
                    Ok(state) => Some(state),
                    Err(e) => {
                        println!("Error deserializing RON: {:?}", e);
                        None
                    }
                }
            }
        }
    };
}

pub fn pretty_print(title: &str, description: Option<&dyn Debug>) {
    let mut output = format!("[{}]", title);
    if let Some(description) = description {
        output = format!("{}: {:?}", output, description);
    }
    println!("{}", output);
}
