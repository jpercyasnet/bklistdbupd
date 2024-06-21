//use std::io::BufReader;
//use std::fs::File;
//use std::path::Path;
//use std::error::Error;
use rusqlite::{ Connection, Result};

#[derive(Debug)]
struct Outpt {
    name: String,
}

pub fn connectdb(conn: &Connection) -> Result<(), rusqlite::Error> {
    let strval = "SELECT name FROM sqlite_master WHERE type = \"table\" ";
    let mut ss = conn.prepare(strval)?;
    let ss_iter = ss.query_map([], |row| {
         Ok(Outpt {
             name: row.get(0)?,
         })
      })?;
    let mut numtables = 0;
    let mut tablena: String = "---".to_string();
    for si in ss_iter {
         numtables = numtables + 1;
         let sii = si.unwrap();
         tablena = sii.name.to_string();
    }
    if numtables == 0 {
//        println!("no tables in database");
        return Err(rusqlite::Error::InvalidParameterName("no tables in database".to_string()));
//        bolok = false;
    } else if !(numtables == 1) {
        println!("{} tables in database, last table is: {}", numtables, tablena);
        let errstr1 = format!("{} tables in database, last table is: {}", numtables, tablena);
        return Err(rusqlite::Error::InvalidParameterName(errstr1.to_string()));
//        bolok = false;
    } else {
        if !(tablena == "blubackup") {
            println!("invalid table of {}", tablena);
            let errstr2 = format!("invalid table of {}", tablena);
            return Err(rusqlite::Error::InvalidParameterName(errstr2.to_string()));
//            bolok = false;
        } else {
            let mut ssx = conn.prepare("SELECT GROUP_CONCAT(NAME,',') FROM PRAGMA_TABLE_INFO('blubackup')")?;
            let ssx_iter = ssx.query_map([], |row| {
                 Ok(Outpt {
                      name: row.get(0)?,
                 })
              })?;
            let mut numlist = 0;
            let mut collist: String = "---".to_string();
            for six in ssx_iter {
                 numlist = numlist + 1;
                 let siix = six.unwrap();
                 collist = siix.name.to_string();
//                 println!("column listing output {:?}", siix.name);
            }
            
            if numlist == 0 {
                println!("no columns for table blubackup in database");
                return Err(rusqlite::Error::InvalidParameterName("no columns for table blubackup in database".to_string()));
//                bolok = false;
            } else if !(numlist == 1) {
                println!("{} column list in database, last column list is: {}", numlist, collist);
                let errstr3 = format!("{} column list in database, last column list is: {}", numlist, collist);
                return Err(rusqlite::Error::InvalidParameterName(errstr3.to_string()));
//                bolok = false;
            } else {
                if !(collist == "refname,filename,dirname,filesize,filedate,md5sum,locations,notes") {
                    println!("column list of {} instead of refname,filename,dirname,filesize,filedate,md5sum,locations,notes", collist);
                    let errstr4 = format!("column list of {} instead of refname,filename,dirname,filesize,filedate,md5sum,locations,notes", collist);
                    return Err(rusqlite::Error::InvalidParameterName(errstr4.to_string()));
//                   bolok = false;
                }
            }
        }
    }
    Ok(())
}
