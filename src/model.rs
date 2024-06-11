#[allow(unused)]
#[derive(PartialEq, Debug)]
pub enum Type {
    SimpleStr(String), // +
    SimpleErr, // -
    Integer, // :
    BulkStr, // $
    Array, // *
    Null, // _
    Bool, // #
    Double, // ,
    BigNum, // (
    BulkErr, // !
    VerbatimStr, // =
    Map, // %
    Set, // ~
    Pushes, // >
}


