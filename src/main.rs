use crate::bdecoder::BValue;

mod bdecoder;

fn main() {

    // let test_value = "d4:infod6:lengthi12345e4:name8:test.txt12:piece lengthi16384eee";
    // let mut pos: usize = 0;
    // let bvalue: BValue = BValue::serialize(test_value,&mut pos).expect("msg");
    // println!("\n\n");
    // bvalue.printer();


    let test_value = "d4:infod4:name4:Test6:lengthi1000ee4:tagsl3:one3:twoee";
    let mut pos: usize = 0;
    let bvalue: BValue = BValue::serialize(test_value, &mut pos).expect("msg");
    println!("\n\n");
    bvalue.printer();
}
