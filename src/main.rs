use crate::bdecoder::BValue;

mod bdecoder;

fn main() {

    // let test_value = "d4:infod6:lengthi12345e4:name8:test.txt12:piece lengthi16384eee";
    // let mut pos: usize = 0;
    // let bvalue: BValue = BValue::serialize(test_value,&mut pos).expect("msg");
    // println!("\n\n");
    // bvalue.printer();


    let test_value: &[u8] = b"d4:name5:Aruns3:agei24e5:peersl6:peer01i6881e6:peer02i6882ee4:infod4:name4:test6:lengthi1024e6:piecesl20:abcdefghijklmnopqrst20:uvwxyzabcdefghijklmneeee";
    let mut pos: usize = 0;
    let bvalue: BValue = BValue::serialize(test_value, &mut pos).expect("msg");
    println!("\n\n");
    bvalue.printer();
}
