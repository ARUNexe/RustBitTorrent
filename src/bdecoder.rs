use core::num;
use std::collections::HashMap;
use std::io::{self, ErrorKind};
use std::ops::Add;
use std::str::FromStr;



pub enum BValue {
    Text(String),
    Number(i64),
    Lists(Vec<BValue>),
    Dicts(Vec<(Vec<u8>,BValue)>),
}


impl BValue{
    
    pub fn serialize(bytes: &[u8], pos: &mut usize ) -> Result<Self,io::Error> {
        match bytes[*pos] {
            b'i' => {
                *pos = *pos + 1; // i
                let mut str_intpool: Vec<u8> = Vec::new();
                let num: i64;
                while bytes[*pos] != b'e' {
                    str_intpool.push(bytes[*pos]);
                    *pos = *pos + 1; // ints iterating
                }
                *pos = *pos + 1; // ints iterating
                
                let number: i64 = match str::from_utf8(&str_intpool) { 
                    Ok(val) => match val.parse::<i64>() {
                        Ok(val2) => val2,
                        Err(_) => return Err(io::Error::new(io::ErrorKind::Unsupported, "Number parsing failed")), 
                        
                    },
                    Err(_) => return Err(io::Error::new(io::ErrorKind::Unsupported, "Number parsing failed")),
                };

                return Ok(BValue::Number(number));

            },
            b'0'..b'9' => {
                println!("Got into this");
                let mut str_intpool: Vec<u8> = Vec::new();
                while bytes[*pos] != b':' {
                    str_intpool.push(bytes[*pos]);
                    *pos = *pos + 1; // ints iterating
                }
                *pos = *pos + 1; // for :
                
                let number: i64 = match str::from_utf8(&str_intpool) { 
                    Ok(val) => match val.parse::<i64>() {
                        Ok(val2) => val2,
                        Err(valfuck) => {
                            println!("Fucker is {}",valfuck);
                            return Err(io::Error::new(io::ErrorKind::Unsupported, " 1 Number parsing failed"));
                        },
                    },
                    Err(_) => return Err(io::Error::new(io::ErrorKind::Unsupported, "2 Number parsing failed")),
                };
            
                let chars = &bytes[*pos..*pos+number as usize];
                let chars = match str::from_utf8(chars){
                    Ok(val) => val.to_string(),
                    Err(_) => return Err(io::Error::new(io::ErrorKind::Unsupported, "3 List parsing failed")),
                };

                *pos = *pos + number as usize;
                return Ok(BValue::Text(chars));
            },
            b'l' => {
                let mut lists = Vec::new();
                *pos = *pos + 1; // for l
                while bytes[*pos] != b'e' {
                    let entry = Self::serialize(bytes, pos);
                    match entry {
                        Ok(val) => {
                            lists.push(val);

                        }
                        Err(_) => {
                            return Err(io::Error::new(io::ErrorKind::Unsupported, "List parsing failed"));
                        }
                    }
                }
                *pos = *pos + 1; // for e
                return Ok(BValue::Lists(lists))
            }
            b'd' => {
                let mut dicts: Vec<(Vec<u8>,BValue)>  = Vec::new(); 
                *pos = *pos + 1; // for l
                while bytes[*pos] != b'e' {
                    let key_entry = Self::serialize(bytes, pos);
                    let dict_key;
                    match key_entry {
                        Ok(val) => {
                            match val {
                                BValue::Text(txt) => dict_key = txt,
                                _ => return Err(io::Error::new(io::ErrorKind::Unsupported, "Key is a non integer value")),
                            };
                        },
                        Err(_) => {
                            return Err(io::Error::new(io::ErrorKind::Unsupported, "List parsing failed"));
                        }
                    }
                    let val_entry = Self::serialize(bytes, pos);
                    let dict_value;
                    match val_entry {
                        Ok(val) => {
                            dict_value = val;
                        },
                        Err(_) => {
                            return Err(io::Error::new(io::ErrorKind::Unsupported, "List parsing failed"));
                        }
                    }
                    let key_bytes = dict_key.as_bytes().to_vec();
                    dicts.push((key_bytes,dict_value));
                }
                *pos = *pos + 1; // for e
                return Ok(BValue::Dicts(dicts))
            }
            _ => {
                return Err( io::Error::new(io::ErrorKind::Unsupported, "No kind of message found"));
            },
        };
    }   


    pub fn printer(self) {
        match self {
            BValue::Text(val) => {
                print!("'{}'",val);
            },
            BValue::Number(val) => {
                print!(" {} ",val);
            },
            BValue::Lists(val) => {
                print!(" [ ");
                for i in val{
                    i.printer();
                }
                print!(" ] ");
            },
            BValue::Dicts(val) => {
                print!(" {{ ");
                println!("");

                for set in val{
                    let key = set.0;
                    let value = set.1; 
                    print!(" {} :",str::from_utf8(&key).unwrap());
                    value.printer();
                    println!("");

                }
                print!(" }} ");
            },
            _ => {
                println!("Unknown value printer fucked");
            }
        }
    }
}

