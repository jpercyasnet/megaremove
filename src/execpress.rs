use std::path::Path;
use std::io::{BufRead, BufReader};
use std::fs::File;

pub fn execpress (mega_file: String, rows_num: u64) -> (u32, String) {
     let mut errcode: u32 = 0;
     let mut errstring: String = "all good and now process execution".to_string();
     let mut bolok = true;
     if Path::new(&mega_file).exists() {
         if rows_num < 2 {
             errcode = 1;
             errstring = "The number of rows is less than 2".to_string();
         } else {
             let targetfullname = format!("{}__tmp3", mega_file);
             if Path::new(&targetfullname).exists() {
                 println!("output file already exists: {}", targetfullname);
                 errcode = 2;
                 errstring = "output file already exists".to_string();
             } else {
                 let file = File::open(mega_file).unwrap();
                 let mut reader = BufReader::new(file);
                 let mut line = String::new();
                 let mut linenum = 0;
                 let mut mvnum = 0;
                 loop {
                     match reader.read_line(&mut line) {
                          Ok(bytes_read) => {
                             if bytes_read == 0 {
                                 break;
                             }
                             if !bolok {
                                 break;
                             }
                             linenum = linenum + 1;
                             if line.starts_with("mv ") {
                                let fromfile: String;
                                 if line.contains("'") {
                                     let vqfiles: Vec<&str> = line.split("'").collect();
                                     let lenvqfiles = vqfiles.len();
                                     if lenvqfiles < 4 {
                                         println!("quoted need 4 parms ({}): {}", lenvqfiles, line);
                                         errcode = 3;
                                         errstring = "a line in the file quoted less than 4".to_string();
                                         bolok = false;
                                         break;
                                     } else {
                                         fromfile = vqfiles[1].to_string();
                                     }
                                 } else {
                                     let vfiles: Vec<&str> = line.split(" ").collect();
                                     let lenvfiles = vfiles.len();
                                     if lenvfiles < 3 {
                                         println!("spaced need 3 parms ({}): {}", lenvfiles, line);
                                         errcode = 4;
                                         errstring = "a line in the file spaced less than 3".to_string();
                                         bolok = false;
                                         break;
                                     } else {
                                         fromfile = vfiles[1].to_string();
                                     }
                                 }
                                 if bolok {
                                     if !fromfile.contains("/MEGA/") {
                                         println!("needs /MEGA/ in file name: {}", line);
                                         errcode = 5;
                                         errstring = "a line in the file does not have /MEGA/".to_string();
                                         bolok = false;
                                         break;
                                     } else {
                                         mvnum = mvnum + 1;
                                     }
                                 }
                             } 
                             if linenum > rows_num {
                                 break;
                             }
                             line.clear();
                          }
                          Err(_err) => {
                               errstring = "error reading mega file".to_string();
                               errcode = 6;
                               bolok = false;   
                               break;
                          }
                     };
                 }
                 if bolok {
                     if mvnum < 1 {
                         errcode = 6;
                         errstring = "no lines containing mv in file".to_string();
                     } else {
                         errstring = "got fclone dry run and all files are ok".to_string();
                     }
                 }
             }
         }             
     } else {
         errstring = "the fclone dry run file does not exist".to_string();
         errcode = 6;
     }
     (errcode, errstring)
}

