use ike::prelude::*;

struct Data {
    count: u32,
}

fn counter(data: &mut Data) -> impl View<Data> + use<> {
    center(button(
        label(format!("Count {}", data.count)),
        |data: &mut Data| data.count += 1,
    ))
}

fn ui(data: &mut Data) -> impl Effect<Data> + use<> {
    window(counter(data))
}

#[ike::main]
fn main() {
    let mut data = Data { count: 0 };

    App::new().run(&mut data, ui);
}
