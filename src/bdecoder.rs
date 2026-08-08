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
    
    pub fn serialize(characters: &str, pos: &mut usize ) -> Result<Self,io::Error> {
        match characters.chars().nth(*pos) {
            Some('i') => {
                *pos = *pos + 1; // i
                let str_numbers = characters.split_at(*pos).1.split_once('e');
                let mut numbers= "";
                match str_numbers {
                    Some(value) => {numbers = value.0;},
                    None => return Err(io::Error::new(io::ErrorKind::Unsupported, "Number parsing failed")),
                };
                let num: i64 = match i64::from_str(numbers) {
                    Ok(num) => num,
                    Err(_) => return Err(io::Error::new(io::ErrorKind::Unsupported, "Number parsing failed")),
                };
                *pos = *pos + 1 + numbers.len(); // num + e
                return Ok(BValue::Number(num));

            },
            Some('0'..'9') => {
                let size_str = characters.split_at(*pos).1.split_once(':').unwrap().0;
                let size = match u64::from_str(size_str) {
                    Ok(num) => num,
                    Err(_) => return Err(io::Error::new(io::ErrorKind::Unsupported, "Number parsing failed for Text length")),
                };
                *pos = *pos + size_str.len() + 1;
                let words = characters.split_at(*pos).1.split_at(size as usize).0;
                *pos = *pos + words.len();
                return Ok(BValue::Text(words.to_string()));
            },
            Some('l') => {
                let mut lists = Vec::new();
                *pos = *pos + 1; // for l
                while characters.chars().nth(*pos) != Some('e') {
                    let entry = Self::serialize(characters, pos);
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
            Some('d') => {
                let mut dicts: Vec<(Vec<u8>,BValue)>  = Vec::new(); 
                *pos = *pos + 1; // for l
                while characters.chars().nth(*pos) != Some('e') {
                    let key_entry = Self::serialize(characters, pos);
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
                    let val_entry = Self::serialize(characters, pos);
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
                println!("Got some shit");
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

