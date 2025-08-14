use clap::Args;

#[derive(Args, Debug, Clone)]
pub struct WalkOptions {
    /// Follow symbolic links when traversing and copying
    #[arg(short = 'L', long = "follow")]
    pub follow_symlinks: bool,

    /// Include hidden files
    #[arg(short = 'H', long = "hidden")]
    pub include_hidden: bool,

    /// Do not respect ignore files
    #[arg(short = 'I', long = "no-ignore")]
    pub no_ignore: bool,
}
