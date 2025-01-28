use std::time::Duration;

use edsdk::Sdk;

fn main() {
    println!("Retrieve cameras");
    let sdk = Sdk::init().unwrap();

    for c in sdk.get_camera_list().unwrap() {
        println!("Camera: {} ", c.description().unwrap(),);

        let c = c.live_view().unwrap();
        let lv_stream = c.live_view_stream().unwrap();

        let take_ph = 500;
        for (ix, _image) in lv_stream.take(take_ph).enumerate() {
            if 250 == ix {
                let shot = c.take_shot().unwrap().get_blob().unwrap();
                println!("shot: {}", shot.size());
            }

            println!("Got image #{ix}");
        }
    }
}
