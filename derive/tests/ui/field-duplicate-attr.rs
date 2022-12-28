use lb_importer_derive::IntoPayload;

#[derive(IntoPayload)]
struct DuplicateError {
    #[track]
    #[track]
    track: String,
    other: String,
}

fn main() {}
