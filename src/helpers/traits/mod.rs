pub mod select_parser;

pub trait StringUtil {
    fn copy_string(&self) -> String;
}

impl StringUtil for String {
    fn copy_string(&self) -> String {
        self.as_str().into()
    }
}
