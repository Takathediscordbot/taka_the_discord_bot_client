pub mod lb;
pub mod psq;
pub mod rlb;
pub mod sq;
#[cfg(feature = "html_server_image_generation")]
pub mod teto;
#[cfg(feature = "html_server_image_generation")]
pub mod tetra_command;
pub mod ts;
pub mod vs;
pub mod vsr;
pub mod vst;

pub fn get_descriptions() -> Box<[(Box<str>, Box<str>)]> {
    [
        ("lb".into(), "get a leaderboard of stats".into()),
        ("rlb".into(), "get a leaderboard of stats in the reverse order".into()),
        ("vst".into(), "compare the stats of two users".into()),
        ("vs".into(), "get a graph from tetrio stats, from a user, from the average of a rank or from the stats of a recent tetra league game".into()),
        ("vsr".into(), "get a graph from tetrio stats, from a user, from the average of a rank or from the stats of a recent tetra league game relative to the highest stat".into()),
        ("psq".into(), "get a graph representing the playstyle from tetrio stats, from a user, from the average of a rank or from the stats of a recent tetra league game".into()),
        ("sq".into(), "get a graph representing the main characteristics from tetrio stats, from a user, from the average of a rank or from the stats of a recent tetra league game".into()),
        #[cfg(feature = "html_server_image_generation")]
        ("teto".into(), "get the tetrio profile of a user".into()),
        #[cfg(feature = "html_server_image_generation")]
        ("tetra".into(), "get a recent tetra league game of a user".into()),
        ("ts".into(), "get the stats from tetrio stats, from a user, from the average of a rank or from the stats of a recent tetra league game formatted in a nice way.".into())
    ].into()
}
