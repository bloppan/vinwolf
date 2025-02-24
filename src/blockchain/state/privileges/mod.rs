use crate::types::Privileges;


impl Default for Privileges {
    fn default() -> Self {
        Privileges {
            bless: 0,
            assign: 0,
            designate: 0,
            always_acc: Vec::new(),
        }
    }
}