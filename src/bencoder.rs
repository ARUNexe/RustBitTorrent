use std::io::{self};

#[allow(dead_code)]
pub enum BValue {
    Text(Vec<u8>),
    Number(i64),
    Lists(Vec<BValue>),
    Dicts(Vec<(Vec<u8>,BValue)>),
}

#[allow(dead_code)]
impl BValue{

    pub fn get_text(&self) -> Result<String, io::Error> {
        let string_text = match self {
            BValue::Text(data) => {
                let data_string = match str::from_utf8(&data) {
                    Ok(data) => data.to_string(),
                    Err(_) => return Err(io::Error::new(io::ErrorKind::Unsupported, "Error while paarsing anounce url string")),
                };
                data_string
            },
            _ => return Err(io::Error::new(io::ErrorKind::Unsupported, "Error while paarsing anounce url not a string")),
        };
        Ok(string_text)
    }
    pub fn get_number(&self) -> Result<i64, io::Error>{
        let number = match self {
            BValue::Number(data) => {
                data
            },
            _ => return Err(io::Error::new(io::ErrorKind::Unsupported, "Error while paarsing number")),
        };
        Ok(*number)
    }
    pub fn get_bytes(&self) -> Result<Vec<u8>,io::Error>{
        match self {
            BValue::Text(data) => {
                Ok(data.clone())
            }
            _ => return Err(io::Error::new(io::ErrorKind::Unsupported, "Error while paarsing number")),
        }
    }

    pub fn encode(&self) -> Result<Vec<u8>,io::Error> {
        let mut result_bytes: Vec<u8> = Vec::new();
        match self {
            BValue::Dicts(dict) =>{
                result_bytes.push(b'd');
                for dict_values in dict{
                    let mut dict_item_key = dict_values.0.clone();
                    let dict_item_value = &dict_values.1;

                    let dict_item_key_str =  match str::from_utf8(&dict_item_key) {
                        Ok(strs) => strs,
                        Err(_) => return Err(io::Error::new(io::ErrorKind::Unsupported, "Error while paarsing dict key string")),
                    };
                    let dict_item_key_len = dict_item_key_str.len();
                    let mut dict_item_key_len = dict_item_key_len.to_string().into_bytes();
                    result_bytes.append(&mut dict_item_key_len);
                    result_bytes.push(b':');
                    result_bytes.append(&mut dict_item_key);
                    let mut value_bytes =  dict_item_value.encode()?;
                    result_bytes.append(&mut value_bytes);
                }
                result_bytes.push(b'e');
            },
            BValue::Text(text) => {
                let mut text = text.clone();
                let text_string_len = text.len();
                let mut text_string_len = text_string_len.to_string().into_bytes();
                result_bytes.append(&mut text_string_len);
                result_bytes.push(b':');
                result_bytes.append(&mut text);
            },
            BValue::Lists(list) => {
                result_bytes.push(b'l');
                for list_values in list{
                    let mut value_bytes =  list_values.encode()?;
                    result_bytes.append(&mut value_bytes);
                }
                result_bytes.push(b'e');
            }
            BValue::Number(num ) => {
                result_bytes.push(b'i');
                let mut num_bytes = num.to_string().into_bytes();
                result_bytes.append(&mut num_bytes);
                result_bytes.push(b'e');
            }
        };
        Ok(result_bytes)
    }

    pub fn decode(bytes: &[u8], pos: &mut usize ) -> Result<Self,io::Error> {
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
            b'0'..=b'9' => {
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
                            println!("Error parsing number: {}", valfuck);
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
                    let entry = Self::decode(bytes, pos);
                    match entry {
                        Ok(val) => {
                            lists.push(val);
                        }
                        Err(_) => {
                            return Err(io::Error::new(io::ErrorKind::Unsupported, "List parsing failed 1"));
                        }
                    };
                };
                *pos = *pos + 1; // for e
                return Ok(BValue::Lists(lists))
            }
            b'd' => {
                let mut dicts: Vec<(Vec<u8>,BValue)>  = Vec::new(); 
                *pos = *pos + 1; // for l
                while bytes[*pos] != b'e' {
                    let key_entry = Self::decode(bytes, pos);
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
                            return Err(io::Error::new(io::ErrorKind::Unsupported, "Dict parsing failed 2"));
                        }
                    }
                    let val_entry = Self::decode(bytes, pos);
                    let dict_value;
                    match val_entry {
                        Ok(val) => {
                            dict_value = val;
                        },
                        Err(_) => {
                            return Err(io::Error::new(io::ErrorKind::Unsupported, "Dict parsing failed 3"));
                        }
                    }
                    let key_bytes = dict_key;
                    dicts.push((key_bytes,dict_value));
                }
                *pos = *pos + 1; // for e
                return Ok(BValue::Dicts(dicts))
            }
            _ => {
                println!("Rest of the bytes is {}",bytes[*pos]);
                return Err( io::Error::new(io::ErrorKind::Unsupported, "No kind of message found"));
            },
        };
    }   

    pub fn printer(&self) {
        match self {
            BValue::Text(val) => {
                let string_text = match str::from_utf8(val) {
                    Ok(text) => text,
                    Err(_) => "hex data",
                };
                print!("'{}'",string_text);
            },
            BValue::Number(val) => {
                print!("{} ",*val);
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
                    let key = set.0.clone();
                    let value = &set.1; 
                    print!(" {} :",str::from_utf8(&key).unwrap());
                    value.printer();
                    println!("");
                }
                print!(" }} ");
            },
        }
    }
}

