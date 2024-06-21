use std::path::Path;
use std::io::{BufRead, BufReader};
use std::fs;
use std::fs::File;
//use connectdb::connectdb;
use rusqlite::Connection;
use crate::connectdb;
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

pub fn execpress (conn: &Connection, hd_value: String, rows_num: u64, outdir: String, excludefile: String, ref_value:String) -> (u32, String) {
     let mut errcode: u32 = 0;
     let mut errstring: String = "all good and now process execution".to_string();
     let mut bolok = true;
     if let Err(e) = connectdb(&conn) {
         errstring = format!("data base error: {}", e);
         errcode = 1;
         bolok = false;
     } else {
         // get list of all tables in database
         let strval = "SELECT name FROM sqlite_master WHERE type = \"table\" ";
         match conn.prepare(strval) {
            Ok(mut ss) => {
               match ss.query_map([], |row| {
                Ok(Outpt {
                    name: row.get(0)?,
                })
              }) {
                Ok(ss_iter) => {
                    let mut numtables = 0;
                    let mut tablena: String = "---".to_string();
                    for si in ss_iter {
                         numtables = numtables + 1;
                         let sii = si.unwrap();
                         tablena = sii.name.to_string();
                    }
                    // check to see if blubackup is the only table
                    if numtables == 0 {
                        errstring  = format!("no tables in database: tablena: {}", tablena);
                        errcode = 1;
                        bolok = false;
                    } else if !(numtables == 1) {  
                        errstring  = format!("{} tables in database: last tablena: {}", numtables, tablena);
                        errcode = 2;
                        bolok = false;
                    } else {
                        if !(tablena == "blubackup") {
                            errstring  = format!("invalid table of {}", tablena);
                            errcode = 3;
                            bolok = false;
                        } else {
                            match conn.prepare("SELECT GROUP_CONCAT(NAME,',') FROM PRAGMA_TABLE_INFO('blubackup')") {
                               Ok(mut ssx) => {
                                   match ssx.query_map([], |row| {
                                Ok(Outpt {
                                     name: row.get(0)?,
                                })
                              }) {
                                Ok(ssx_iter) => {
                                    for six in ssx_iter {
                                        let _siix = six.unwrap();
//                                        println!("column listing output {:?}", siix.name);
                                   }
                                }
                                Err(err) => {
                                    errstring  = format!("Error doing sql select group {:?}", err);
                                    errcode = 4;
                                    bolok = false; 
                                }
                              };
                               }
                               Err(err) => {
                                   errstring  = format!("Error doing sql select group {:?}", err);
                                   errcode = 5;
                                   bolok = false;
                               } 
                            }        
                         }
                    }                     
                }
                Err(err) => {
                    errstring  = format!("Error doing sql select group {:?}", err);
                    errcode = 6;
                    bolok = false;

                }
              }
            }
            Err(err) => {
                errstring  = format!("Error doing sql select name {:?}", err);
                errcode = 7;
                bolok = false;
            } 
         };
     }






     if bolok {
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
                             if bk_iter.count() < 1 {
                                 errstring  = format!("No database entries for reference name {}", ref_value);
                                 errcode = 8;
                                 bolok = false;
                             }
                          }
                          Err(err) => {
                             errstring  = format!("Error doing sql select reference name aa {:?}", err);
                             errcode = 9;
                             bolok = false;
                         }
                       }
                   }
                   Err(err) => {
                      println!("Error doing sql select reference name bb {:?}", err);
                      errcode = 10;
                      bolok = false;   
                   }
         }
     }
     if bolok {
         if !Path::new(&hd_value).exists() {
             errstring = "HD file does not exist".to_string();
             errcode = 4;
         } else {
             let file = File::open(hd_value).unwrap();
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
                       if linenum == 1 {
                           let vecline: Vec<&str> = line.split("|").collect();
                           if !(vecline.len() == 6) {
                               errstring = "HD list does not have 6 items per line separated by |".to_string();
                               errcode = 11;
                               bolok = false;   
                               break;
                           }
                       }
                   }
                   Err(_err) => {
                       errstring = "error reading HD list ".to_string();
                       errcode = 12;
                       bolok = false;   
                       break;
                   }
                };
             }
             if bolok {
                 if !(linenum == rows_num) {
                     errstring = format!("HD rows do not match displayed: {}  counted: {}", rows_num, linenum);
                     errcode = 13;
                     bolok = false;   
                 }
             }
         }
     }
     if bolok {
         if !Path::new(&outdir).exists() {
              errstring = "Output directory does not exist".to_string();
              errcode = 14;
              bolok = false;  
         }
     }
     if bolok {
         if !(excludefile == "--") {
             if !Path::new(&excludefile).exists() {
                 errstring = "Exclude file does not exist".to_string();
                 errcode = 16;
//                 bolok = false;
             } else {
                 let file = File::open(excludefile).unwrap();
                        let mut reader = BufReader::new(file);
                        let mut line = String::new();
                        let mut linenumx: u64 = 0;
                        loop {
                           match reader.read_line(&mut line) {
                             Ok(bytes_read) => {
                                  // EOF: save last file address to restart from this address for next run
                                 if bytes_read == 0 {
                                     break;
                                 }
                                 linenumx = linenumx + 1;
                             }
                             Err(_err) => {
                                  errstring = "error reading exclude list ".to_string();
                                  errcode = 17;
                                  bolok = false;   
                                  break;
                             }
                           };
                        }
                        if bolok {
                            if linenumx < 1 {
                                errstring = "error reading exclude list ".to_string();
                                errcode = 18;
//                                bolok = false;
                            }
                        }
                    }
                }
             }
     (errcode, errstring)
}

