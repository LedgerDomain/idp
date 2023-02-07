use anyhow::Result;
use idp_proto::PlumHeadSeal;

#[derive(Debug, PartialEq)]
pub enum FragmentQueryResult<'a> {
    // This returns a Plum via its PlumHeadSeal.
    // TODO: Refactor to use Box<dyn Any> later, since it will be useful to be able to return
    // particular data types from queries, such as number of elements in a data structure, or
    // other things like content size.
    // TODO: Figure out how to be able to return this by reference
    Value(PlumHeadSeal),
    ForwardQueryTo {
        // TODO: Figure out how to be able to return this by reference
        target: PlumHeadSeal,
        rest_of_query_str: &'a str,
    },
}

// TODO: Consider adding a generic form of query which expects to return a particular
// type T and fails otherwise.  Or a weaker form where the TypeId of the terminal Value
// is specified and checked.
pub trait FragmentQueryable<'a> {
    fn fragment_query_single_segment(
        &self,
        self_plum_head_seal: &PlumHeadSeal,
        query_str: &'a str,
    ) -> Result<FragmentQueryResult<'a>>;
}
