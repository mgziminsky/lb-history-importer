use lb_importer_derive::IntoPayload;

#[derive(IntoPayload)]
#[payload(track = test: String)]
#[payload(invalid = other: String)]
struct MultipleOuter {
    test: String,
    other: String,
}

fn main() {}
