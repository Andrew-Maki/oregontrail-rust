//! The trail itself: landmarks, mileages, terrain and the two famous forks.
//!
//! Mileages follow the classic game's table (Independence to Oregon City,
//! roughly 1,870-1,930 miles depending on which branches you take).

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Terrain {
    Plains,
    Highlands,
    Mountains,
    Desert,
}

impl Terrain {
    pub fn name(self) -> &'static str {
        match self {
            Terrain::Plains => "plains",
            Terrain::Highlands => "high plains",
            Terrain::Mountains => "mountains",
            Terrain::Desert => "high desert",
        }
    }

    /// How much the ground slows the wagon.
    pub fn travel_factor(self) -> f64 {
        match self {
            Terrain::Plains => 1.0,
            Terrain::Highlands => 0.92,
            Terrain::Mountains => 0.72,
            Terrain::Desert => 0.86,
        }
    }

    /// Degrees Fahrenheit relative to the same month on the plains.
    pub fn temp_offset(self) -> i32 {
        match self {
            Terrain::Plains => 0,
            Terrain::Highlands => -8,
            Terrain::Mountains => -17,
            Terrain::Desert => 4,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct RiverSpec {
    /// Typical depth in feet before weather is taken into account.
    pub base_depth: f64,
    pub width_ft: u32,
    pub has_ferry: bool,
    /// Cost of the ferry, in cents.
    pub ferry_cost: i64,
    pub has_guide: bool,
    /// Sets of clothing demanded by the local guide.
    pub guide_clothing: u32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Fork {
    /// South Pass: strike out for the Green River, or detour to Fort Bridger.
    SouthPass,
    /// Blue Mountains: push on to The Dalles, or swing by Fort Walla Walla.
    BlueMountains,
    /// The Dalles: float the Columbia, or pay for the Barlow Toll Road.
    Dalles,
}

#[derive(Clone, Copy, Debug)]
pub enum Kind {
    Start,
    /// A landmark you can look at and rest beside.
    Point,
    /// A fort or trading post, where you can buy supplies.
    Fort {
        /// Price multiplier relative to Independence.
        markup: f64,
    },
    River(RiverSpec),
    Finish,
}

#[derive(Clone, Debug)]
pub struct Landmark {
    pub name: &'static str,
    /// Miles from the previous landmark.
    pub miles_from_prev: u32,
    pub terrain: Terrain,
    pub kind: Kind,
    pub fork: Option<Fork>,
    pub blurb: &'static str,
}

impl Landmark {
    fn new(
        name: &'static str,
        miles_from_prev: u32,
        terrain: Terrain,
        kind: Kind,
        blurb: &'static str,
    ) -> Self {
        Landmark {
            name,
            miles_from_prev,
            terrain,
            kind,
            fork: None,
            blurb,
        }
    }

    fn with_fork(mut self, fork: Fork) -> Self {
        self.fork = Some(fork);
        self
    }

    pub fn is_fort(&self) -> bool {
        matches!(self.kind, Kind::Fort { .. })
    }

    pub fn markup(&self) -> f64 {
        match self.kind {
            Kind::Fort { markup } => markup,
            _ => 1.0,
        }
    }
}

/// Independence, Missouri through South Pass -- the stretch everybody walks.
pub fn opening_route() -> Vec<Landmark> {
    vec![
        Landmark::new(
            "Independence, Missouri",
            0,
            Terrain::Plains,
            Kind::Start,
            "The jumping-off place. Wagons gather here each spring, waiting for \
             the prairie grass to grow tall enough to feed the oxen.",
        ),
        Landmark::new(
            "Kansas River Crossing",
            102,
            Terrain::Plains,
            Kind::River(RiverSpec {
                base_depth: 2.2,
                width_ft: 620,
                has_ferry: true,
                ferry_cost: 500,
                has_guide: true,
                guide_clothing: 2,
            }),
            "The Kansas River runs wide and brown through groves of cottonwood. \
             A ferryman has set up shop on the near bank.",
        ),
        Landmark::new(
            "Big Blue River Crossing",
            83,
            Terrain::Plains,
            Kind::River(RiverSpec {
                base_depth: 1.8,
                width_ft: 530,
                has_ferry: false,
                ferry_cost: 0,
                has_guide: true,
                guide_clothing: 2,
            }),
            "A clear, quick little river with a gravel bottom. There is no ferry \
             here -- wagons must take their chances.",
        ),
        Landmark::new(
            "Fort Kearney",
            119,
            Terrain::Plains,
            Kind::Fort { markup: 1.25 },
            "An army post on the Platte, built to guard the emigrant road. \
             Soldiers, Pawnee traders and a great many wagon ruts.",
        ),
        Landmark::new(
            "Chimney Rock",
            250,
            Terrain::Plains,
            Kind::Point,
            "A great spire of clay and sandstone rising five hundred feet from \
             the valley floor. Emigrants carve their names into the base.",
        ),
        Landmark::new(
            "Fort Laramie",
            86,
            Terrain::Highlands,
            Kind::Fort { markup: 1.5 },
            "A fur-trading post of adobe walls, now the last real outfitting \
             stop before the mountains. Prices here are steep.",
        ),
        Landmark::new(
            "Independence Rock",
            190,
            Terrain::Highlands,
            Kind::Point,
            "The Register of the Desert -- a granite whaleback covered in the \
             names of thousands. Reach it by the Fourth of July and you are on \
             schedule.",
        ),
        Landmark::new(
            "South Pass",
            102,
            Terrain::Mountains,
            Kind::Point,
            "The broad saddle through the Rocky Mountains, so gentle you can \
             scarcely tell you have crossed the Continental Divide. The waters \
             now run to the Pacific.",
        )
        .with_fork(Fork::SouthPass),
    ]
}

/// After South Pass. `via_bridger` adds the Fort Bridger detour.
pub fn after_south_pass(via_bridger: bool) -> Vec<Landmark> {
    let green_river = |miles: u32| {
        Landmark::new(
            "Green River Crossing",
            miles,
            Terrain::Highlands,
            Kind::River(RiverSpec {
                base_depth: 5.6,
                width_ft: 1100,
                has_ferry: true,
                ferry_cost: 1200,
                has_guide: true,
                guide_clothing: 3,
            }),
            "The Green runs deep, cold and swift between cut banks. This is the \
             most dangerous crossing on the whole trail.",
        )
    };

    let soda_springs = Landmark::new(
        "Soda Springs",
        112,
        Terrain::Highlands,
        Kind::Point,
        "Dozens of bubbling mineral springs hiss out of the rock. The water \
         tastes of soda and tingles on the tongue.",
    );

    if via_bridger {
        vec![
            Landmark::new(
                "Fort Bridger",
                57,
                Terrain::Highlands,
                Kind::Fort { markup: 1.6 },
                "Jim Bridger's trading post: a pair of log huts and a stockade, \
                 but a blacksmith, fresh oxen and a welcome rest.",
            ),
            green_river(150),
            soda_springs,
        ]
    } else {
        vec![green_river(57), soda_springs]
    }
}

/// Soda Springs through the Blue Mountains -- common to both branches.
pub fn middle_route() -> Vec<Landmark> {
    vec![
        Landmark::new(
            "Fort Hall",
            58,
            Terrain::Highlands,
            Kind::Fort { markup: 1.75 },
            "Here the California Trail splits away to the south. Those bound \
             for Oregon are warned the hardest country is still ahead.",
        ),
        Landmark::new(
            "Snake River Crossing",
            182,
            Terrain::Desert,
            Kind::River(RiverSpec {
                base_depth: 3.1,
                width_ft: 1000,
                has_ferry: false,
                ferry_cost: 0,
                has_guide: true,
                guide_clothing: 3,
            }),
            "The Snake has carved itself a canyon of black rock. Shoshone \
             guides will swim your wagon across for a price in clothing.",
        ),
        Landmark::new(
            "Fort Boise",
            104,
            Terrain::Desert,
            Kind::Fort { markup: 2.0 },
            "A Hudson's Bay Company post among the willows. Everything costs \
             twice what it did in Missouri, and is worth it.",
        ),
        Landmark::new(
            "Blue Mountains",
            160,
            Terrain::Mountains,
            Kind::Point,
            "Timbered ridges stacked one behind another. Wagons must be let \
             down the steep pitches on ropes.",
        )
        .with_fork(Fork::BlueMountains),
    ]
}

/// After the Blue Mountains. `via_walla_walla` is slightly longer but has a fort.
pub fn after_blue_mountains(via_walla_walla: bool) -> Vec<Landmark> {
    let dalles = |miles: u32| {
        Landmark::new(
            "The Dalles",
            miles,
            Terrain::Mountains,
            Kind::Point,
            "The end of the wagon road. Below, the Columbia River narrows and \
             roars through a chute of basalt.",
        )
        .with_fork(Fork::Dalles)
    };

    if via_walla_walla {
        vec![
            Landmark::new(
                "Fort Walla Walla",
                60,
                Terrain::Mountains,
                Kind::Fort { markup: 2.1 },
                "A small post where the Walla Walla meets the Columbia. Canoes \
                 and salmon and a chance to buy flour.",
            ),
            dalles(120),
        ]
    } else {
        vec![dalles(168)]
    }
}

/// The final leg out of The Dalles.
pub fn after_dalles(barlow_road: bool) -> Vec<Landmark> {
    if barlow_road {
        vec![Landmark::new(
            "Willamette Valley",
            125,
            Terrain::Mountains,
            Kind::Finish,
            "Oregon City, at the falls of the Willamette. Green bottomland, \
             tall timber, and free land for the taking.",
        )]
    } else {
        vec![Landmark::new(
            "Willamette Valley",
            100,
            Terrain::Mountains,
            Kind::Finish,
            "Oregon City, at the falls of the Willamette. Green bottomland, \
             tall timber, and free land for the taking.",
        )]
    }
}

/// Cost of the Barlow Toll Road, in cents.
pub const BARLOW_TOLL: i64 = 800;
