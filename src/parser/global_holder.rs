use super::types::*;
use std::rc::Rc;
use std::string::String;
use std::vec::Vec;

pub struct GlobalHolder<'a> {
    vars: Vec<Rc<Variable<'a>>>,
    label_id: u32,
}
impl<'a> GlobalHolder<'a> {
    pub fn new() -> GlobalHolder<'a> {
        return GlobalHolder {
            vars: Vec::new(),
            label_id: 0,
        };
    }

    fn gen_label_id(&mut self) -> String {
        self.label_id += 1;
        return format!(".L.data.{}", self.label_id);
    }

    pub fn find_var(&self, name: &str) -> Option<Rc<Variable<'a>>> {
        return self
            .vars
            .iter()
            .find(|&var| var.name == name)
            .map(|var| var.clone());
    }

    pub fn push(&mut self, var: Variable<'a>) {
        self.vars.push(Rc::new(var));
    }

    pub fn string_literal(&mut self, s: &'a str) -> Rc<Variable<'a>> {
        let mut array_size = s.len() as u32;
        // NOTE: For string termination: '\0'
        array_size += 1;
        let var = Rc::new(Variable {
            ty: Type::Array(Box::new(Type::Char), array_size),
            name: self.gen_label_id(),
            offset: 0,
            is_local: false,
            content: Some(s),
        });
        self.vars.push(var.clone());
        return var.clone();
    }

    pub fn dump_to_vec(self) -> Vec<Rc<Variable<'a>>> {
        return self.vars;
    }
}
