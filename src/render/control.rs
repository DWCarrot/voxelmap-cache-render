pub struct Control {
    pub version: u32,
}

impl Default for Control {

    fn default() -> Self {
        Control {
            version: 1
        }
    }
}

impl Control {

    pub fn modify_by(&mut self, line: &str) -> Result<(), usize> {
        let mut sp = line.splitn(2, ':');
        let tuple = (sp.next().ok_or_else(|| line.len())?, sp.next().ok_or_else(|| line.len())?);
        match tuple.0 {
            "version" => {
                self.version = tuple.1.parse().map_err(|e| "version".len())?;
            },
            _ => {

            }
        }
        Ok(())
    }
}