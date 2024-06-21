use iced::widget::{button, column, row, text, checkbox, horizontal_space, radio, progress_bar, Space};
use iced::{Alignment, Element, Command, Application, Settings, Color, Size};
use iced::theme::{self, Theme};
use iced::executor;
use iced::window;
use iced_futures::futures;
use futures::channel::mpsc;
extern crate chrono;
use std::path::{Path, PathBuf};
use std::io::{Write, BufRead, BufReader};
use std::fs::File;
use std::time::Duration as timeDuration;
use std::time::Instant as timeInstant;
use chrono::Local;
use std::thread::sleep;
use rusqlite::{Connection, Result};

mod get_winsize;
mod inputpress;
mod diroutpress;
mod execpress;
mod connectdb;
use get_winsize::get_winsize;
use inputpress::inputpress;
use diroutpress::diroutpress;
use execpress::execpress;
use connectdb::connectdb;
#[derive(Debug)]
struct Outpt {
    name: String,
}
#[derive(Debug)]
struct Bkup {
      rowid: u64,
      refname: String,
      filename: String,
      dirname: String,
      filesize: u64,
      filedate: String,
      md5sum: Option<String>,
      locations: Option<String>,
      notes: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RunChoice {
    TST,
    UPD,
}

impl Default for RunChoice {
    fn default() -> Self {
        RunChoice::TST
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HourChoice {
    DIF,
    EQL,
}

impl Default for HourChoice {
    fn default() -> Self {
        HourChoice::EQL
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DateChoice {
    DIF,
    EQL,
}

impl Default for DateChoice {
    fn default() -> Self {
        DateChoice::EQL
    }
}

pub fn main() -> iced::Result {

     let mut widthxx: f32 = 1350.0;
     let mut heightxx: f32 = 750.0;
     let (errcode, errstring, widtho, heighto) = get_winsize();
     if errcode == 0 {
         widthxx = widtho as f32 - 20.0;
         heightxx = heighto as f32 - 75.0;
         println!("{}", errstring);
     } else {
         println!("**ERROR {} get_winsize: {}", errcode, errstring);
     }

     Dbupdate::run(Settings {
        window: window::Settings {
            size: Size::new(widthxx, heightxx),
            ..window::Settings::default()
        },
        ..Settings::default()
     })
}

struct Dbupdate {
    db_value: String,
    ref_value: String,
    mess_color: Color,
    msg_value: String,
    runchoice_value: RunChoice,
    hourchoice_value: HourChoice,
    datechoice_value: DateChoice,
    rows_num: u64,
    bk_value: String,
    exclude_value: String,
    outdir_value: String,
    haveexclude: bool,
    do_progress: bool,
    progval: f32,
    dbconn: Connection,
    tx_send: mpsc::UnboundedSender<String>,
    rx_receive: mpsc::UnboundedReceiver<String>,
}

#[derive(Debug, Clone)]
enum Message {
    DBPressed,
    BKPressed,
    EXPressed,
    OutdirPressed,
    ExecPressed,
    CheckExclude(bool),
    RunRadioSelected(RunChoice),
    HourRadioSelected(HourChoice),
    DateRadioSelected(DateChoice),
    ExecxFound(Result<Execx, Error>),
    ProgressPressed,
    ProgRtn(Result<Progstart, Error>),
}

impl Application for Dbupdate {
    type Message = Message;
    type Theme = Theme;
    type Flags = ();
    type Executor = executor::Default;
    fn new(_flags: Self::Flags) -> (Dbupdate, iced::Command<Message>) {
        let (tx_send, rx_receive) = mpsc::unbounded();
        ( Self { db_value: "--".to_string(), ref_value: "--".to_string(), msg_value: "no message".to_string(),
               rows_num: 0, mess_color: Color::from([0.0, 1.0, 0.0]), bk_value: "--".to_string(), 
               exclude_value: "--".to_string(), outdir_value: "--".to_string(), haveexclude: false,
               do_progress: false, progval: 0.0, tx_send, rx_receive, runchoice_value: RunChoice::TST,
               hourchoice_value: HourChoice::EQL, datechoice_value: DateChoice::EQL, dbconn: Connection::open_in_memory().unwrap(),
 
          },
          Command::none()
        )
    }

    fn title(&self) -> String {
        String::from("bklist update to database-- iced")
    }

    fn update(&mut self, message: Message) -> Command<Message>  {
        match message {
            Message::BKPressed => {
               self.runchoice_value = RunChoice::TST;
               self.hourchoice_value = HourChoice::EQL;
               self.datechoice_value = DateChoice::EQL;
               let mut inputstr: String = self.bk_value.clone();
               if !Path::new(&inputstr).exists() {
                   if Path::new(&self.db_value).exists() {
                       inputstr = self.db_value.clone();
                   }
               }
               let (errcode, errstr, newinput) = inputpress(inputstr);
               self.msg_value = errstr.to_string();
               if errcode == 0 {
                   if Path::new(&newinput).exists() {
                       self.mess_color = Color::from([0.0, 1.0, 0.0]);
                       self.bk_value = newinput.to_string();
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
                                 let vecline: Vec<&str> = line.split("|").collect();
                                 if linenum == 1 {
                                     self.ref_value = vecline[4].to_string();
                                     if !(vecline.len() == 6) {
                                         self.msg_value = "bklist does not have 6 items per line separated by |".to_string();
                                         self.mess_color = Color::from([1.0, 0.0, 0.0]);
                                         bolok = false;   
                                         break;
                                     }
                                 } else {
                                        if self.ref_value != vecline[4] {
                                            self.msg_value = format!("bklist has more than 1 reference name {} and {}", self.ref_value, vecline[4]);
                                            self.mess_color = Color::from([1.0, 0.0, 0.0]);
                                            bolok = false;   
                                            break;
                                        }
                                 }
                             }
                             Err(_err) => {
                                 self.msg_value = "error reading bklist ".to_string();
                                 self.mess_color = Color::from([1.0, 0.0, 0.0]);
                                 bolok = false;   
                                 break;
                             }
                          };
                       }
                       if bolok {
                           self.rows_num = linenum;
                           self.mess_color = Color::from([0.0, 1.0, 0.0]);
                           self.msg_value = format!("got bklist with reference name of {} and retrieved its number of rows", self.ref_value);
                       } 
                   } else {
                       self.mess_color = Color::from([1.0, 0.0, 0.0]);
                       self.msg_value = format!("bklist file does not exist: {}", newinput);
                   }
               } else {
                   self.mess_color = Color::from([1.0, 0.0, 0.0]);
               }
               Command::none()
           }
            Message::DBPressed => {
               let mut inputstr: String = self.db_value.clone();
               self.runchoice_value = RunChoice::TST;
               self.hourchoice_value = HourChoice::EQL;
               self.datechoice_value = DateChoice::EQL;
               if !Path::new(&inputstr).exists() {
                   if Path::new(&self.bk_value).exists() {
                       inputstr = self.bk_value.clone();
                   }
               }
               let (errcode, errstr, newinput) = inputpress(inputstr);
               self.msg_value = errstr.to_string();
               if errcode == 0 {
                   let conn = Connection::open(newinput.clone()).unwrap();
                   if let Err(e) = connectdb(&conn) {
                       self.msg_value = format!("data base error: {}", e);
                       self.mess_color = Color::from([1.0, 0.0, 0.0]);
                   } else {
                       self.dbconn = conn;
                       self.db_value = newinput.to_string();
                       self.mess_color = Color::from([0.0, 1.0, 0.0]);
                   }
               } else {
                   self.mess_color = Color::from([1.0, 0.0, 0.0]);
               }
               Command::none()
            }
            Message::EXPressed => {
               if !self.haveexclude {
                   self.msg_value = "The checkbox is not check for having exclude file".to_string();
                   self.mess_color = Color::from([1.0, 0.0, 0.0]);
               } else {
                   let mut inputstr: String = self.exclude_value.clone();
                   self.runchoice_value = RunChoice::TST;
                   self.hourchoice_value = HourChoice::EQL;
                   self.datechoice_value = DateChoice::EQL;
                   if !Path::new(&inputstr).exists() {
                       if Path::new(&self.bk_value).exists() {
                           inputstr = self.bk_value.clone();
                       }
                   }
                   let (errcode, errstr, newinput) = inputpress(inputstr);
                   self.msg_value = errstr.to_string();
                   if errcode == 0 {
                       self.exclude_value = newinput.to_string();
                       self.mess_color = Color::from([0.0, 1.0, 0.0]);
                   } else {
                       self.mess_color = Color::from([1.0, 0.0, 0.0]);
                   }
               }
               Command::none()
            }
            Message::OutdirPressed => {
               let mut indirstr: String = self.outdir_value.clone();
               self.runchoice_value = RunChoice::TST;
               self.hourchoice_value = HourChoice::EQL;
               self.datechoice_value = DateChoice::EQL;
               if !Path::new(&indirstr).exists() {
                   if Path::new(&self.bk_value).exists() {
                       let getpath = PathBuf::from(&self.bk_value);
                       let getdir = getpath.parent().unwrap();
                       indirstr = getdir.to_str().unwrap().to_string();
                   }
               }
               let (errcode, errstr, newdir) = diroutpress(indirstr);
               self.msg_value = errstr.to_string();
               if errcode == 0 {
                   self.outdir_value = newdir.to_string();
                   self.mess_color = Color::from([0.0, 1.0, 0.0]);
               } else {
                   self.mess_color = Color::from([1.0, 0.0, 0.0]);
               }
               Command::none()
            }
             Message::CheckExclude(completed) => {
                if !completed {
                    self.exclude_value = "--".to_string();
                }
                self.haveexclude = completed;
                Command::none()
            }       
             Message::RunRadioSelected(xchoice) => {
                        let strx = match xchoice {
                        RunChoice::TST => "choice tst selected",
                        RunChoice::UPD => "choice update selected" };
                       self.runchoice_value = xchoice;
                       self.msg_value = strx.to_string();
                       Command::none()
           }
             Message::HourRadioSelected(xchoice) => {
                        let strx = match xchoice {
                        HourChoice::DIF => "choice hour different selected",
                        HourChoice::EQL => "choice hour equal selected" };
                       self.hourchoice_value = xchoice;
                       self.msg_value = strx.to_string();
                       Command::none()
           }
             Message::DateRadioSelected(xchoice) => {
                        let strx = match xchoice {
                        DateChoice::DIF => "choice date different selected",
                        DateChoice::EQL => "choice date equal selected", };
                       self.datechoice_value = xchoice;
                       self.msg_value = strx.to_string();
                       Command::none()
           }

            Message::ExecPressed => {
               let mut bolok = true;
               if self.haveexclude {
                   if self.exclude_value == "--" {
                       self.msg_value = "Have exclude file check box check, but no exclude file".to_string();
                       self.mess_color = Color::from([1.0, 0.0, 0.0]);
                       bolok = false;
                   }
               }        
               if bolok {     
                   let (errcode, errstr) = execpress(&self.dbconn, self.bk_value.clone(), self.rows_num.clone(), self.outdir_value.clone(), self.exclude_value.clone(), self.ref_value.clone());
                   self.msg_value = errstr.to_string();
                   if errcode == 0 {
                       self.mess_color = Color::from([0.0, 1.0, 0.0]);
                   } else {
                       self.mess_color = Color::from([1.0, 0.0, 0.0]);
                       bolok = false;
                   }
               }
               if !bolok {
                    Command::none()
               } else {
                    let tstrun: bool = match self.runchoice_value {
                        RunChoice::TST => true,
                        RunChoice::UPD => false };
                    let hourmust: bool = match self.hourchoice_value {
                        HourChoice::DIF => false,
                        HourChoice::EQL => true };
                    let datemust: bool = match self.datechoice_value {
                        DateChoice::DIF => false,
                        DateChoice::EQL => true };
 
                    Command::perform(Execx::execit(self.db_value.clone(), self.bk_value.clone(), self.ref_value.clone(), self.rows_num.clone(), self.outdir_value.clone(), self.exclude_value.clone(), tstrun, hourmust, datemust, self.tx_send.clone()), Message::ExecxFound) 
               }
               
            }
            Message::ExecxFound(Ok(exx)) => {
              self.msg_value = exx.errval.clone();
              self.progval = 100.0;
              self.do_progress = false;
               if exx.errcd == 0 {
                   self.mess_color = Color::from([0.0, 1.0, 0.0]);
               } else {
                   self.mess_color = Color::from([1.0, 0.0, 0.0]);
               }
               Command::none()
            }
            Message::ExecxFound(Err(_error)) => {
               self.msg_value = "error in Execx execit routine".to_string();
               self.do_progress = false;
               self.mess_color = Color::from([1.0, 0.0, 0.0]);
               Command::none()
            }
            Message::ProgressPressed => {
                   self.do_progress = true;
                   Command::perform(Progstart::pstart(0), Message::ProgRtn)
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
                    if lenpg1 == 4 {
                        let prog1 = progvec[0].to_string();
                        if prog1 == "Progress" {
                            let num_int: i32 = progvec[1].parse().unwrap_or(-9999);
                            if num_int == -9999 {
                                println!("progress numeric not numeric: {}", inputval);
                            } else {
                                let dem_int: i32 = progvec[2].parse().unwrap_or(-9999);
                                if dem_int == -9999 {
                                    println!("progress numeric not numeric: {}", inputval);
                                } else {
                                    self.progval = 100.0 * (num_int as f32 / dem_int as f32);
                                    if self.runchoice_value == RunChoice::TST {
                                        self.msg_value = format!("Test run progress: {} of {}  {}", num_int, dem_int, progvec[3]);
                                    } else { 
                                        self.msg_value = format!("Update run progress: {} of {}  {}", num_int, dem_int, progvec[3]);
                                    }
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
                Command::perform(Progstart::pstart(5), Message::ProgRtn)
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
            row![button("sqlite database Button").on_press(Message::DBPressed),
                 text(&self.db_value).size(20).width(1000)
            ].align_items(Alignment::Center).spacing(10).padding(10),
            row![button("bklist file Button").on_press(Message::BKPressed),
                 text(&self.bk_value).size(20).width(1000)
            ].align_items(Alignment::Center).spacing(10).padding(10),
            row![text(format!("number of rows: {}", self.rows_num)).size(20), Space::with_width(100),
            ].align_items(Alignment::Center).spacing(10).padding(10),
            row![checkbox("Have Exclude file", self.haveexclude).on_toggle(Message::CheckExclude), button("Exclude List file Button").on_press(Message::EXPressed),
                 text(&self.exclude_value).size(20).width(1000)
            ].align_items(Alignment::Center).spacing(10).padding(10),
            row![button("Output Directory Button").on_press(Message::OutdirPressed),
                 text(&self.outdir_value).size(20).width(1000)
            ].align_items(Alignment::Center).spacing(10).padding(10),
            row![radio(
                         "Hour Must Be Equal",
                         HourChoice::EQL,
                         Some(self.hourchoice_value.clone()),
                         Message::HourRadioSelected,
                ).size(15), radio(
                         "Hour Can Be Different",
                         HourChoice::DIF,
                         Some(self.hourchoice_value.clone()),
                         Message::HourRadioSelected,
                ).size(15), Space::with_width(100), radio(
                         "Date Must Be Equal",
                         DateChoice::EQL,
                         Some(self.datechoice_value.clone()),
                         Message::DateRadioSelected,
                ).size(15), radio(
                         "Date Can Be Different",
                         DateChoice::DIF,
                         Some(self.datechoice_value.clone()),
                         Message::DateRadioSelected,
                ).size(15),
            ].spacing(80).padding(1),
            row![horizontal_space(),radio(
                         "Test Run",
                         RunChoice::TST,
                         Some(self.runchoice_value.clone()),
                         Message::RunRadioSelected,
                ).size(15), radio(
                         "Update Run",
                         RunChoice::UPD,
                         Some(self.runchoice_value.clone()),
                         Message::RunRadioSelected,
                ).size(15),
                 button("Exec Button").on_press(Message::ExecPressed),
            ].align_items(Alignment::Center).spacing(10).padding(10),
            row![button("Start Progress Button").on_press(Message::ProgressPressed),
                 progress_bar(0.0..=100.0,self.progval),
                 text(format!("{:.2}%", &self.progval)).size(30),
            ].align_items(Alignment::Center).spacing(5).padding(10),
         ]
        .padding(10)
        .align_items(Alignment::Start)
        .into()
    }

    fn theme(&self) -> Theme {
//       Theme::Dark
//       Theme::Light
       Theme::Dracula
//       Theme::Nord
//       Theme::SolarizedLight
//       Theme::SolarizedDark
//       Theme::GruvboxLight
//       Theme::GruvboxDark
//       Theme::CatppuccinLatte
//       Theme::CatppuccinFrappe
//       Theme::CatppuccinMacchiato
//       Theme::CatppuccinMocha
//       Theme::TokyoNight
//       Theme::TokyoNightStorm
//       Theme::TokyoNightLight
//       Theme::KanagawaWave
//       Theme::KanagawaDragon
//       Theme::KanagawaLotus
//       Theme::Moonfly
//       Theme::Nightfly
//       Theme::Oxocarbon
/*          Theme::custom(theme::Palette {
                        background: Color::from_rgb8(240, 240, 240),
                        text: Color::BLACK,
                        primary: Color::from_rgb8(230, 230, 230),
                        success: Color::from_rgb(0.0, 1.0, 0.0),
                        danger: Color::from_rgb(1.0, 0.0, 0.0),
                    })
*/               
    }
}

#[derive(Debug, Clone)]
struct Execx {
    errcd: u32,
    errval: String,
}

impl Execx {

    async fn execit(db_value: String, bk_value: String, ref_value: String, rows_num: u64, outdir: String, excludefile: String, tstrun: bool, hourmust: bool, datemust: bool, tx_send: mpsc::UnboundedSender<String>,) -> Result<Execx, Error> {
     let mut errstring  = "test of exec ".to_string();
     let mut errcode: u32 = 0;
     let mut bolok = true;
     let mut vecexclude: Vec<String> = Vec::new();
     let mut bexclusion = false;
     let mut outseq: u32 = 1;
// if exclusion list load the file (which contains directories to be exluded) into a vector string.
     if Path::new(&excludefile).exists() {
         let fileex = File::open(excludefile.clone()).unwrap();
         let mut readerex = BufReader::new(fileex);
         let mut lineex = String::new();
         let mut lineexnum: u64 = 0;
         loop {
               match readerex.read_line(&mut lineex) {
                  Ok(bytes_read) => {
                  // EOF: save last file address to restart from this address for next run
                      if bytes_read == 0 {
                          break;
                      }
                      lineexnum = lineexnum + 1;
                      let excl: String = lineex.trim().to_string();
                      vecexclude.push(excl);
                      lineex.clear();
                  }
                   Err(err) => {
                      errstring = format!("error of {} reading exclude list ", err);
                      errcode = 1;
                      bolok = false;   
                      break;
                  }
               };
         }
             if lineexnum < 1 {
                 errstring  = format!("exclusion file {} has no records", excludefile);
                 errcode = 2;
                 bolok = false;
             } else {
                 bexclusion = true;
             }
     }
     let conn = Connection::open(db_value).unwrap();
     if bolok {
         if let Err(e) = connectdb(&conn) {
                 errstring  = format!("error in opening db of {} ", e);
                 errcode = 21;
                 bolok = false;
         }
     }
     if bolok {
         let mut excludout: String = format!("{}/excluded{:02}.excout", outdir, outseq);
         let mut noentriesout: String = format!("{}/noentries{:02}.neout", outdir, outseq);
         let mut errout: String = format!("{}/generrors{:02}.errout", outdir, outseq);
         loop {
               if Path::new(&errout).exists() {
                   outseq = outseq + 1;
                   excludout = format!("{}/excluded{:02}.excout", outdir, outseq);
                   noentriesout = format!("{}/noentries{:02}.neout", outdir, outseq);
                   errout = format!("{}/generrors{:02}.errout", outdir, outseq);
               } else {
                   break;
               }
         }          
         let mut excludeop = File::create(excludout).unwrap();
         let mut noop = File::create(noentriesout).unwrap();
         let mut errop = File::create(errout).unwrap();
         let filex = File::open(bk_value).unwrap();
         let mut readerx = BufReader::new(filex);
         let mut linex = String::new();
         let mut linenumx: u64 = 0;
         let mut numeq1: u64 = 0;
         let start_time = timeInstant::now();
//            get list of reference name entries
         match conn.prepare("SELECT  rowid, refname, filename, dirname, filesize, filedate, md5sum
                FROM blubackup
                WHERE refname = :ref") {
                   Ok(mut stmt) => {
                       match stmt.query_map(&[(":ref", &ref_value)], |row| {
                          Ok(Bkup {
                                   rowid: row.get(0)?,
                                   refname: row.get(1)?,
                                   filename: row.get(2)?,
                                   dirname: row.get(3)?,
                                   filesize: row.get(4)?,
                                   filedate: row.get(5)?,
                                   md5sum: row.get(6)?,
                                   locations: row.get(6)?,
                                   notes: row.get(6)?,
                          })
                       }) {
                          Ok(bk_iter) => {
                             loop {
                                 match readerx.read_line(&mut linex) {
                                    Ok(bytes_read) => {
                                        if bytes_read == 0 {
                                            break;
                                        }
                                        linenumx = linenumx + 1;
                                        let vecline: Vec<&str> = linex.split("|").collect();
                                        let inptfilenm: String = vecline[0].trim().to_string();
                                        let inptdirnm: String = vecline[3].to_string();              
                                        let mut bnotex = true;
//   check if excluded
                                        if bexclusion {
                                            for strexcl in &vecexclude {
                                                 if inptdirnm.contains(strexcl) {
                                                     bnotex = false;
                                                     let stroutput = format!("{}|{}", inptdirnm, linex);
                                                     writeln!(&mut excludeop, "{}", stroutput).unwrap();
                                                     break;
                                                 }
                                            }
                                        }
                                        if bnotex {
                                            let mut numentries = 0;
                                            let mut bothequal = 0;
                                            let mut bolsgoodhr = false;
                                            for bk in bk_iter {
                                                 let bki = bk.unwrap();
                                                 let dbfilename = bki.filename.to_string();
                                                 let dbdirnm = bki.dirname.to_string();
                                                 if dbfilename == inptfilenm {
                                                     if dbdirnm != inptdirnm {
                                                         writeln!(&mut errop, "for file {} directories do not match db:{} input:{}", dbfilename, dbdirnm, inptdirnm).unwrap();
                                                     } else {
                                                         let dbfilesize = bki.filesize;
                                                         let inptsizi64: i64 = vecline[1].parse().unwrap_or(-9999);
                                                         let mut bolsizeeq = false;
                                                         if inptsizi64 == -9999 {
                                                             writeln!(&mut errop, "size of {} is invalid for {}", vecline[1], inptfilenm).unwrap();
                                                         } else {
                                                             if dbfilesize != inptsizi64 as u64 {
                                                                 writeln!(&mut errop, "file:{} sizes are not equal db:{} input:{}", inptfilenm, dbfilesize, inptsizi64).unwrap();
                                                             } else {
                                                                 let dbfiledate = bki.filedate.to_string();
                                                                 if dbfiledate == vecline[2] {
                                                                     if bki.md5sum == None {
                                                                         writeln!(&mut errop, "md5sum is null for {}", inptfilenm).unwrap();
                                                                         if !tstrun {
                                                                             let strrow = format!("{}", bki.rowid);
                                                                             let strrowx = strrow.as_str();
                                                                             match conn.prepare("UPDATE blubackup SET md5sum = :md5 WHERE rowid = :rwd") {
                                                                                 Ok (mut upit) => {
                                                                                      match upit.query_map(&[(":md5", &vecline[5]),(":rwd", &strrowx)], |row| {
                                                                                         Ok(Outpt {
                                                                                            name: row.get(0)?,
                                                                                         })
                                                                                      }) {
                                                                                         Ok(upit_iter) => {
                                                                                             for uupit in upit_iter {
                                                                                                  let uuupit = uupit.unwrap();
                                                                                                  writeln!(&mut errop, "update listing output {:?}, row {}", uuupit.name, bki.rowid).unwrap();
                                                                                             }
                                                                                         }
                                                                                         Err(err) => {
                                                                                             println!("sql call aa update error {}", err);
                                                                                         }
                                                                                      }
                                                                                 }
                                                                                 Err(err) => {
                                                                                     println!("sql call ab update error {}", err);
                                                                                 }
                                                                             }
                                                                         } else {
                                                                             writeln!(&mut errop, "test run - update {}, row {}", inptfilenm, bki.rowid).unwrap();
                                                                         }
                                                                     } else {
                                                                         let strbkimd5sum = bki.md5sum.as_ref().unwrap();
//                                                                     if bki.md5sum.as_ref() == Some(&vecline[5].to_string()) {
                                                                         if strbkimd5sum == &vecline[5].to_string() {
                                                                             writeln!(&mut errop, "md5sum same {} for {}", vecline[5], inptfilenm).unwrap();
                                                                         } else {
//                                                                     writeln!(&mut errop, "md5sum differ is db {:?} and input {} for {}", bki.md5sum, vecline[5], inptfilenm).unwrap();
                                                                             writeln!(&mut errop, "md5sum differ is db {} and input {} for {}", strbkimd5sum, vecline[5], inptfilenm).unwrap();
                                                                         }
                                                                     }
                                                                 } else {
          // if filedate not equal
                                                                     let dbdate: String = dbfiledate[0..10].to_string();
 //                                               let dbhr: String = filedate[11..13].to_string();
                                                                     let dbminsec: String = dbfiledate[13..].to_string();
                                                                     let indate: String = vecline[2][0..10].to_string();
 //                                               let inhr: String = vecline[2][11..13].to_string();
                                                                     let inminsec: String = vecline[2][13..].to_string();
                                                                     let mut bolonlyhr = false;
                                                                     if dbminsec == inminsec {
                                                                         if (dbdate == indate) || !datemust {
                                                                             bolonlyhr = true;
                                                                         }
                                                                     }
                                                                     if bki.md5sum == None {
                                                                         if bolonlyhr {
                                                                             if !hourmust {
                                                                                 bolsgoodhr = true;
                                                                                 writeln!(&mut errop, "md5sum is null for {} but hr diff ok", inptfilenm).unwrap();
                                                                                 if !tstrun {
                                                                                     let strrow = format!("{}", bki.rowid);
                                                                                     let strrowx = strrow.as_str();
                                                                                     match conn.prepare("UPDATE blubackup SET md5sum = :md5 WHERE rowid = :rwd") {
                                                                                        Ok (mut upit) => {
                                                                                             match upit.query_map(&[(":md5", &vecline[5]),(":rwd", &strrowx)], |row| {
                                                                                                Ok(Outpt {
                                                                                                    name: row.get(0)?,
                                                                                                })
                                                                                             }) {
                                                                                                Ok(upit_iter) => {
                                                                                                    for uupit in upit_iter {
                                                                                                       let uuupit = uupit.unwrap();
                                                                                                       writeln!(&mut errop, "update listing output {:?}, row {}", uuupit.name, bki.rowid).unwrap();
                                                                                                    }
                                                                                                }
                                                                                                Err(err) => {
                                                                                                    println!("sql call ac update error {}", err);
                                                                                                }
                                                                                             }
                                                                                        }
                                                                                        Err(err) => {
                                                                                            println!("sql call ad update error {}", err);
                                                                                        }
                                                                                     }
                                                                                 } else {
                                                                                     writeln!(&mut errop, "test run - update {}, row {}",inptfilenm, bki.rowid).unwrap();
                                                                                 }
                                                                             } else {
                                                                                 writeln!(&mut errop, "input size equal no md5sum but only hr different date db {} not equal to input {} for {} ",dbfiledate, vecline[2], inptfilenm).unwrap();
                                                                             }
                                                                         } else {
                                                                                 writeln!(&mut errop, "input size equal no md5sum but date db {} not equal to input {} for {}",dbfiledate, vecline[2], inptfilenm).unwrap();
                                                                         }
                                                                     } else {
                                                                         if bki.md5sum.as_ref() == Some(&vecline[5].to_string()) {
                                                                             if bolonlyhr {
                                                                                 writeln!(&mut errop, "only hr different date db {} not equal to input {} but size & md5sum same for {}",dbfiledate, vecline[2], inptfilenm).unwrap();
                                                                             } else {
                                                                                 writeln!(&mut errop, "date db {} not equal to input {} but size & md5sum same for {}",dbfiledate, vecline[2], inptfilenm).unwrap();
                                                                             }
                                                                         }
                                                                     }
                                                                 }
                                                             }
                                                         }
                                                         break;
                                                     }
                                                 }
                                            } // end for
                                        }
                                        linex.clear();
                                    }
                                    Err(err) => {
                                       errstring = format!("error of {} reading input file ", err);
                                       errcode = 1;
                                       bolok = false;   
                                       break;
                                    }
                                 }
                             } // end loop
                          }
                          Err(err) => {
                             println!("sql call ba update error {}", err);
                          }
                       }
                   }
                   Err(err) => {
                       println!("sql call bb update error {}", err);
                   }
         }
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

    pub async fn pstart(numsecs: u64) -> Result<Progstart, Error> {
//     let errstring  = " ".to_string();
//     let colorx = Color::from([0.0, 1.0, 0.0]);
     if numsecs > 0 {     
         sleep(timeDuration::from_secs(numsecs));
     }
     Ok(Progstart {
//            errcolor: colorx,
//            errval: errstring,
        })
    }
}
