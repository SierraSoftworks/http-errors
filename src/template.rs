use std::collections::HashMap;

pub fn template_replace<V: AsRef<str>>(template: &str, vars: HashMap<&'static str, V>) -> String {
    let mut result = String::new();

    for token in template_iter(template) {
        match token {
            TemplateToken::Text(text) => result.push_str(text),
            TemplateToken::Var(var) => {
                if let Some(value) = vars.get(var) {
                    result.push_str(value.as_ref());
                } else {
                    result.push_str(&format!("{{{{{}}}}}", var));
                }
            }
        }
    }

    result
}

fn template_iter(template: &str) -> TemplateIter<'_> {
    TemplateIter { template, pos: 0 }
}

#[derive(Debug, PartialEq)]
enum TemplateToken<'a> {
    Text(&'a str),
    Var(&'a str),
}

struct TemplateIter<'a> {
    template: &'a str,
    pos: usize,
}

impl<'a> Iterator for TemplateIter<'a> {
    type Item = TemplateToken<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.template.len() {
            return None;
        }

        let start = self.pos;
        match self.template[start..].find("{{") {
            Some(next) if next > 0 => {
                self.pos = start + next;
                Some(TemplateToken::Text(&self.template[start..self.pos]))
            }
            Some(next) => {
                let var_start = start + next + 2;
                if let Some(var_end) = self.template[var_start..].find("}}") {
                    self.pos = var_start + var_end + 2;
                    Some(TemplateToken::Var(
                        &self.template[var_start..var_start + var_end],
                    ))
                } else {
                    self.pos = start + next + 2;
                    Some(TemplateToken::Text(&self.template[start..self.pos]))
                }
            }
            None => {
                self.pos = self.template.len();
                Some(TemplateToken::Text(&self.template[start..]))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_iter() {
        let template = "Hello, {{name1}} and {{name2}}!";
        let mut iter = template_iter(template);
        assert_eq!(iter.next(), Some(TemplateToken::Text("Hello, ")));
        assert_eq!(iter.next(), Some(TemplateToken::Var("name1")));
        assert_eq!(iter.next(), Some(TemplateToken::Text(" and ")));
        assert_eq!(iter.next(), Some(TemplateToken::Var("name2")));
        assert_eq!(iter.next(), Some(TemplateToken::Text("!")));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_template_replace() {
        let template = "Hello, {{name}}!";
        let mut vars = HashMap::new();
        vars.insert("name", "world");
        assert_eq!(template_replace(template, vars), "Hello, world!");

        let template = "Hello, {{name}}! This is a test of the {{system}} system.";
        let mut vars = HashMap::new();
        vars.insert("name", "world");
        vars.insert("system", "emergency broadcast");
        assert_eq!(
            template_replace(template, vars),
            "Hello, world! This is a test of the emergency broadcast system."
        );
    }
}
