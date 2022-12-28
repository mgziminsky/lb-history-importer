use lb_importer_derive::IntoPayload;

#[derive(IntoPayload)]
struct MultipleError {
    #[track]
    track1: String,
    #[track]
    track2: String,
    other: String,
}

fn main() {}
