use edsdk::Sdk;

fn main() {
    println!("Retrieve cameras");
    let sdk = Sdk::init().unwrap();

    for c in sdk.get_camera_list().unwrap() {
        println!("Camera: {}", c.description().unwrap());
        let image = c.set_flash().take_shot().unwrap();

        image.save_to("/home/azl/canon-auto-shots").unwrap();
    }
}
