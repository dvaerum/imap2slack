//use imap::error::Result;
//use std::vec::Vec;

//pub trait Folder {
//    fn list_folders(&mut self, dir: &str, filter_sub_folders: &str) -> Result<Vec<String>>;
//    fn list_root_folders(&mut self) -> Result<Vec<String>>;
//    fn list_sub_folders(&mut self, dir: &str, ) -> Result<Vec<String>>;
//}

//impl<T: Read + Write> Folder for Client<T> {
//    fn list_folders(&mut self, dir: &str, filter_sub_folders: &str) -> Result<Vec<String>> {
//        match self.run_command_and_read_response(&format!("LIST \"{}\" \"{}\"", dir, filter_sub_folders)) {
//            Ok(lines) => {
//                return Ok(lines);
//            }
//            Err(e) => return Err(e),
//        }
//    }
//
//    #[allow(dead_code)]
//    fn list_root_folders(&mut self) -> Result<Vec<String>> {
//        self.list_folders("", "%")
//    }
//
//    #[allow(dead_code)]
//    fn list_sub_folders(&mut self, dir: &str, ) -> Result<Vec<String>> {
//        self.list_folders(dir, "%")
//    }
//}