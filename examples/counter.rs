use ike::prelude::*;

struct Data {
    count: u32,
}

fn counter(data: &Data) -> impl View<Data> + use<> {
    center(
        transform(button(
            label(format!("count {}", data.count)),
            |data: &mut Data| data.count += 1,
        ))
        .rotation(data.count as f32)
        .transition(Transition::linear(0.5)),
    )
}

fn ui(data: &Data) -> impl Effect<Data> + use<> {
    window(counter(data))
}

fn main() {
    let mut data = Data { count: 0 };

    App::new().run(&mut data, ui).unwrap();
}
