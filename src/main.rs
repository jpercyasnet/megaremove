use iced::widget::{button, column, row, text, horizontal_space, progress_bar};
use iced::{Alignment, Element, Command, Application, Settings, Color};
use iced::theme::{self, Theme};
use iced::executor;
use iced::window;
use iced_futures::futures;
use futures::channel::mpsc;
use std::path::{Path};
use std::io::{Write, BufRead, BufReader};
use std::fs::File;
use std::time::Duration as timeDuration;
use std::thread::sleep;

mod get_winsize;
mod inputpress;
mod execpress;
use get_winsize::get_winsize;
use inputpress::inputpress;
use execpress::execpress;

pub fn main() -> iced::Result {

     let mut widthxx: u32 = 1350;
     let mut heightxx: u32 = 750;
     let (errcode, errstring, widtho, heighto) = get_winsize();
     if errcode == 0 {
         widthxx = widtho - 20;
         heightxx = heighto - 75;
         println!("{}", errstring);
     } else {
         println!("**ERROR {} get_winsize: {}", errcode, errstring);
     }

     Megaremove::run(Settings {
        window: window::Settings {
            size: (widthxx, heightxx),
            ..window::Settings::default()
        },
        ..Settings::default()
     })
}

struct Megaremove {
    mega_file: String,
    mess_color: Color,
    msg_value: String,
    rows_num: u64,
    do_progress: bool,
    progval: f32,
    tx_send: mpsc::UnboundedSender<String>,
    rx_receive: mpsc::UnboundedReceiver<String>,
}

#[derive(Debug, Clone)]
enum Message {
    MegaPressed,
    ExecPressed,
    ExecxFound(Result<Execx, Error>),
    ProgressPressed,
    ProgRtn(Result<Progstart, Error>),
}

impl Application for Megaremove {
    type Message = Message;
    type Theme = Theme;
    type Flags = ();
    type Executor = executor::Default;
    fn new(_flags: Self::Flags) -> (Megaremove, iced::Command<Message>) {
        let (tx_send, rx_receive) = mpsc::unbounded();
//        let mut heightxx: f32 = 190.0;
//        let (errcode, errstring, _widtho, heighto) = get_winsize();
//        if errcode == 0 {
//            heightxx = 190.0 + ((heighto as f32 - 768.0) / 2.0);
//            println!("{}", errstring);
//        } else {
//         println!("**ERROR {} get_winsize: {}", errcode, errstring);
//        }
        ( Self { mega_file: "--".to_string(), msg_value: "no message".to_string(),
               rows_num: 0, mess_color: Color::from([0.0, 0.0, 0.0]), 
               do_progress: false, progval: 0.0, tx_send, rx_receive,
 
          },
          Command::none()
        )
    }

    fn title(&self) -> String {
        String::from("create mega-rm file list from fclone dry run move -- iced")
    }

    fn update(&mut self, message: Message) -> Command<Message>  {
        match message {
            Message::MegaPressed => {
               let inputstr: String = self.mega_file.clone();
               let (errcode, errstr, newinput) = inputpress(inputstr);
               self.msg_value = errstr.to_string();
               if errcode == 0 {
                   if Path::new(&newinput).exists() {
                       self.mess_color = Color::from([0.0, 1.0, 0.0]);
                       self.mega_file = newinput.to_string();
                       self.rows_num = 0;
                       let mut bolok = true;
                       let file = File::open(newinput).unwrap();
                       let mut reader = BufReader::new(file);
                       let mut line = String::new();
                       let mut linenum: u64 = 0;
                       loop {
                          match reader.read_line(&mut line) {
                             Ok(bytes_read) => {
                                 // EOF: save last file address to restart from this address for next run
                                 if bytes_read == 0 {
                                     break;
                                 }
                                 linenum = linenum + 1;
                             }
                             Err(_err) => {
                                 self.msg_value = "error reading mega ".to_string();
                                 self.mess_color = Color::from([1.0, 0.0, 0.0]);
                                 bolok = false;   
                                 break;
                             }
                          };
                       }
                       if bolok {
                           self.rows_num = linenum;
                           self.mess_color = Color::from([0.0, 1.0, 0.0]);
                           self.msg_value = "got mega ls file and retrieved its number of rows".to_string();
                       } 
                   } else {
                       self.mess_color = Color::from([1.0, 0.0, 0.0]);
                       self.msg_value = format!("mega ls file does not exist: {}", newinput);
                   }
               } else {
                   self.mess_color = Color::from([1.0, 0.0, 0.0]);
               }
               Command::none()
           }
            Message::ExecPressed => {
               let (errcode, errstr) = execpress(self.mega_file.clone(), self.rows_num.clone());
               self.msg_value = errstr.to_string();
               if errcode == 0 {
                   self.mess_color = Color::from([0.0, 1.0, 0.0]);
                   Command::perform(Execx::execit(self.mega_file.clone(), self.rows_num.clone(), self.tx_send.clone()), Message::ExecxFound)
               } else {
                   self.mess_color = Color::from([1.0, 0.0, 0.0]);
                   Command::none()
               }
            }
            Message::ExecxFound(Ok(exx)) => {
               self.msg_value = exx.errval.clone();
               if exx.errcd == 0 {
                   self.mess_color = Color::from([0.0, 1.0, 0.0]);
               } else {
                   self.mess_color = Color::from([1.0, 0.0, 0.0]);
               }
               Command::none()
            }
            Message::ExecxFound(Err(_error)) => {
               self.msg_value = "error in copyx copyit routine".to_string();
               self.mess_color = Color::from([1.0, 0.0, 0.0]);
               Command::none()
            }
            Message::ProgressPressed => {
                   self.do_progress = true;
                   Command::perform(Progstart::pstart(), Message::ProgRtn)
            }
            Message::ProgRtn(Ok(_prx)) => {
              if self.do_progress {
                let mut inputval  = " ".to_string();
                let mut bgotmesg = false;
                while let Ok(Some(input)) = self.rx_receive.try_next() {
                   inputval = input;
                   bgotmesg = true;
                }
                if bgotmesg {
                    let progvec: Vec<&str> = inputval[0..].split("|").collect();
                    let lenpg1 = progvec.len();
                    if lenpg1 == 3 {
                        let prog1 = progvec[0].clone().to_string();
                        if prog1 == "Progress" {
                            let num_int: i32 = progvec[1].clone().parse().unwrap_or(-9999);
                            if num_int == -9999 {
                                println!("progress numeric not numeric: {}", inputval);
                            } else {
                                let dem_int: i32 = progvec[2].clone().parse().unwrap_or(-9999);
                                if dem_int == -9999 {
                                    println!("progress numeric not numeric: {}", inputval);
                                } else {
                                    self.progval = 100.0 * (num_int as f32 / dem_int as f32);
                                    self.msg_value = format!("Convert progress: {}", self.progval);
                                    self.mess_color = Color::from([0.0, 0.0, 1.0]);
                                }
                            }
                        } else {
                            println!("message not progress: {}", inputval);
                        }
                    } else {
                        println!("message not progress: {}", inputval);
                    }
                }             
                Command::perform(Progstart::pstart(), Message::ProgRtn)
              } else {
                Command::none()
              }
            }
            Message::ProgRtn(Err(_error)) => {
                self.msg_value = "error in Progstart::pstart routine".to_string();
                self.mess_color = Color::from([1.0, 0.0, 0.0]);
               Command::none()
            }

        }
    }

    fn view(&self) -> Element<Message> {
        column![
            row![text("Message:").size(20),
                 text(&self.msg_value).size(30).style(*&self.mess_color),
            ].align_items(Alignment::Center).spacing(10).padding(10),
            row![button("fclones dry run input file Button").on_press(Message::MegaPressed).style(theme::Button::Secondary),
                 text(&self.mega_file).size(20).width(1000)
            ].align_items(Alignment::Center).spacing(10).padding(10),
            row![text(format!("number of rows: {}", self.rows_num)).size(20), horizontal_space(100),
            ].align_items(Alignment::Center).spacing(10).padding(10),
            row![horizontal_space(200),
                 button("Exec Button").on_press(Message::ExecPressed).style(theme::Button::Secondary),
            ].align_items(Alignment::Center).spacing(10).padding(10),
            row![button("Start Progress Button").on_press(Message::ProgressPressed),
                 progress_bar(0.0..=100.0,self.progval),
                 text(format!("{}%", &self.progval)).size(30),
            ].align_items(Alignment::Center).spacing(5).padding(10),
         ]
        .padding(5)
        .align_items(Alignment::Start)
        .into()
    }

    fn theme(&self) -> Theme {
//       Theme::Light
          Theme::custom(theme::Palette {
                        background: Color::from_rgb8(240, 240, 240),
                        text: Color::BLACK,
                        primary: Color::from_rgb8(230, 230, 230),
                        success: Color::from_rgb(0.0, 1.0, 0.0),
                        danger: Color::from_rgb(1.0, 0.0, 0.0),
                    })
               
    }
}

#[derive(Debug, Clone)]
struct Execx {
    errcd: u32,
    errval: String,
}
impl Execx {
//    const TOTAL: u16 = 807;

    async fn execit(mega_file: String, rows_num: u64, tx_send: mpsc::UnboundedSender<String>,) -> Result<Execx, Error> {
     let mut errstring  = "tess of exec ".to_string();
     let mut errcode: u32 = 0;
     if Path::new(&mega_file).exists() {
         if rows_num < 2 {
             errcode = 1;
             errstring = "The number of rows is less than 2".to_string();
         } else {
             let targetfullname = format!("{}__tmp3", mega_file);
             let file = File::open(mega_file).unwrap(); 
             let mut reader = BufReader::new(file);
             let mut targetfile = File::create(targetfullname.clone()).unwrap();
             let mut bolok = true;
             let mut line = String::new();
             let mut linenum = 0;
             let mut echonum = 0;
             loop {
                 match reader.read_line(&mut line) {
                      Ok(bytes_read) => {
                         if bytes_read == 0 {
                             break;
                         }
                         linenum = linenum + 1;
                         if !bolok {
                             break;
                         }
                         if line.starts_with("mv ") {
                             let fromfile: String;
                             if line.contains("'") {
                                 let vqfiles: Vec<&str> = line.split("'").collect();
                                 let lenvqfiles = vqfiles.len();
                                 if lenvqfiles < 4 {
                                     println!("quoted need 4 parms ({}): {}", lenvqfiles, line);
                                     errcode = 2;
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
                                     errcode = 3;
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
                                     errcode = 4;
                                     errstring = "a line in the file does not have /MEGA/".to_string();
                                     bolok = false;
                                     break;
                                 } else {
                                     let lcurrpos = fromfile.find("/MEGA/").unwrap();
                                     let lfilest = lcurrpos + 6;
                                     let megafullname = fromfile.get(lfilest..).unwrap().to_string();
                                     let stroutput = format!("mega-rm '{}'", megafullname);
                                     writeln!(&mut targetfile, "{}", stroutput).unwrap();
                                     echonum = echonum + 1;
                                     if echonum > 19 {
                                        echonum = 0;
                                        let echoout = format!(r#"echo "processing {} of {}" "#, linenum, rows_num);
                                        writeln!(&mut targetfile, "{}", echoout).unwrap();
                                     }
                                 }
                             }
                         } 
                         let msgx = format!("Progress|{}|{}", linenum, rows_num);
                         tx_send.unbounded_send(msgx).unwrap();
                         if linenum > rows_num {
                             break;
                         }
                         line.clear();
                      }
                      Err(_err) => {
                           errstring = "error reading mega-ls file: do file i and iconv".to_string();
                           errcode = 1;
                           bolok = false;   
                           break;
                      }
                 };
             }
             if bolok {
                 errstring = "processed the file".to_string();
             } 
         }             
     } else {
         errstring = "the fclone dry run file does not exist".to_string();
         errcode = 6;
     }
     Ok(Execx {
            errcd: errcode,
            errval: errstring,
        })
    }
}
#[derive(Debug, Clone)]
pub enum Error {
//    APIError,
//    LanguageError,
}

// loop thru by sleeping for 5 seconds
#[derive(Debug, Clone)]
pub struct Progstart {
//    errcolor: Color,
//    errval: String,
}

impl Progstart {

    pub async fn pstart() -> Result<Progstart, Error> {
//     let errstring  = " ".to_string();
//     let colorx = Color::from([0.0, 0.0, 0.0]);
     sleep(timeDuration::from_secs(5));
     Ok(Progstart {
//            errcolor: colorx,
//            errval: errstring,
        })
    }
}
