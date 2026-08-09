use std::io::{self};



pub enum BValue {
    Text(Vec<u8>),
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
                let chars = chars.to_vec();
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
                                BValue::Text(txt) => {
                                    dict_key = txt
                                },
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
                    let key_bytes = dict_key;
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
                let string_text = str::from_utf8(&val).expect("Error while printing v8 as string");
                print!("'{}'",string_text);
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
        }
    }
}

