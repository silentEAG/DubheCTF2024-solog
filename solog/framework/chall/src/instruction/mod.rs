mod add_collaborator;
mod add_comment;
mod create_post;
mod edit_comment;
mod clap;

const POST_SUFFIX: &[u8] = b"post";
const COMMENT_SUFFIX: &[u8] = b"comment";

pub use self::add_collaborator::instruction as add_collaborator;
pub use self::add_comment::instruction as add_comment;
pub use self::create_post::instruction as create_post;
pub use self::edit_comment::instruction as edit_comment;
pub use self::clap::instruction as clap;