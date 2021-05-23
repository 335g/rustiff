use crate::decode::Decoder;

pub struct Reader<R> {
    decoder: Decoder<R>,
}
