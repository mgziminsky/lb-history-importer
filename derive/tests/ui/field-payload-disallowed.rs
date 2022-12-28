use lb_importer_derive::IntoPayload;

#[derive(IntoPayload)]
struct PayloadFieldError {
    #[payload]
    track: String,
    other: String,
}

fn main() {}
