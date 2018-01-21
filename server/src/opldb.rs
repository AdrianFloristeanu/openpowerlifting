//! An in-memory data store for OpenPowerlifting data.
//!
//! Because our data is read-only at runtime, we can lay out data structures
//! better than a "real" database like SQLite3 or PostreSQL. Additionally,
//! by storing all the data in formats native to Rust, we avoid copy overhead.

use csv;
use serde;
use serde::de::{self, Visitor};

use std::error::Error;
use std::mem;
use std::fmt;
use std::str::FromStr;

pub use opldb_enums::*;

/// Deserializes a f32 value from the CSV source,
/// defaulting to 0.0 if the empty string is encountered.
fn deserialize_f32_with_default<'de, D>(deserializer: D) -> Result<f32, D::Error>
    where D: serde::Deserializer<'de>
{
    struct F32StrVisitor;

    impl<'de> Visitor<'de> for F32StrVisitor {
        type Value = f32;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("f32 or the empty string")
        }

        fn visit_str<E>(self, value: &str) -> Result<f32, E>
            where E: de::Error
        {
            if value.is_empty() {
                return Ok(0.0);
            }
            f32::from_str(value).map_err(E::custom)
        }
    }

    deserializer.deserialize_str(F32StrVisitor)
}

/// The definition of a Lifter in the database.
#[derive(Deserialize)]
pub struct Lifter {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Username")]
    pub username: String,
    #[serde(rename = "Instagram")]
    pub instagram: Option<String>,
}

/// The definition of a Meet in the database.
#[derive(Deserialize)]
pub struct Meet {
    #[serde(rename = "MeetPath")]
    pub path: String,
    #[serde(rename = "Federation")]
    pub federation: Federation,
    #[serde(rename = "Date")]
    pub date: String,
    #[serde(rename = "MeetCountry")]
    pub country: String,
    #[serde(rename = "MeetState")]
    pub state: Option<String>,
    #[serde(rename = "MeetTown")]
    pub town: Option<String>,
    #[serde(rename = "MeetName")]
    pub name: String,
}

/// The definition of an Entry in the database.
#[derive(Deserialize)]
pub struct Entry {
    #[serde(rename = "MeetID")]
    pub meet_id: u32,
    #[serde(rename = "LifterID")]
    pub lifter_id: u32,
    #[serde(rename = "Sex")]
    pub sex: Sex,
    #[serde(rename = "Event")]
    pub event: Option<String>,
    #[serde(rename = "Equipment")]
    pub equipment: Equipment,
    #[serde(rename = "Age")]
    pub age: Option<f32>,
    #[serde(rename = "Division")]
    pub division: Option<String>,
    #[serde(rename = "BodyweightKg")]
    #[serde(deserialize_with = "deserialize_f32_with_default")]
    pub bodyweightkg: f32,
    #[serde(rename = "WeightClassKg")]
    pub weightclasskg: Option<String>,
    #[serde(rename = "Squat1Kg")]
    #[serde(deserialize_with = "deserialize_f32_with_default")]
    pub squat1kg: f32,
    #[serde(rename = "Squat2Kg")]
    #[serde(deserialize_with = "deserialize_f32_with_default")]
    pub squat2kg: f32,
    #[serde(rename = "Squat3Kg")]
    #[serde(deserialize_with = "deserialize_f32_with_default")]
    pub squat3kg: f32,
    #[serde(rename = "Squat4Kg")]
    #[serde(deserialize_with = "deserialize_f32_with_default")]
    pub squat4kg: f32,
    #[serde(rename = "BestSquatKg")]
    #[serde(deserialize_with = "deserialize_f32_with_default")]
    pub bestsquatkg: f32,
    #[serde(rename = "Bench1Kg")]
    #[serde(deserialize_with = "deserialize_f32_with_default")]
    pub bench1kg: f32,
    #[serde(rename = "Bench2Kg")]
    #[serde(deserialize_with = "deserialize_f32_with_default")]
    pub bench2kg: f32,
    #[serde(rename = "Bench3Kg")]
    #[serde(deserialize_with = "deserialize_f32_with_default")]
    pub bench3kg: f32,
    #[serde(rename = "Bench4Kg")]
    #[serde(deserialize_with = "deserialize_f32_with_default")]
    pub bench4kg: f32,
    #[serde(rename = "BestBenchKg")]
    #[serde(deserialize_with = "deserialize_f32_with_default")]
    pub bestbenchkg: f32,
    #[serde(rename = "Deadlift1Kg")]
    #[serde(deserialize_with = "deserialize_f32_with_default")]
    pub deadlift1kg: f32,
    #[serde(rename = "Deadlift2Kg")]
    #[serde(deserialize_with = "deserialize_f32_with_default")]
    pub deadlift2kg: f32,
    #[serde(rename = "Deadlift3Kg")]
    #[serde(deserialize_with = "deserialize_f32_with_default")]
    pub deadlift3kg: f32,
    #[serde(rename = "Deadlift4Kg")]
    #[serde(deserialize_with = "deserialize_f32_with_default")]
    pub deadlift4kg: f32,
    #[serde(rename = "BestDeadliftKg")]
    #[serde(deserialize_with = "deserialize_f32_with_default")]
    pub bestdeadliftkg: f32,
    #[serde(rename = "TotalKg")]
    #[serde(deserialize_with = "deserialize_f32_with_default")]
    pub totalkg: f32,
    #[serde(rename = "Place")]
    pub place: Option<String>,
    #[serde(rename = "Wilks")]
    #[serde(deserialize_with = "deserialize_f32_with_default")]
    pub wilks: f32,
    #[serde(rename = "McCulloch")]
    #[serde(deserialize_with = "deserialize_f32_with_default")]
    pub mcculloch: f32,
}

/// The collection of data stores that constitute the complete dataset.
pub struct OplDb {
    /// The LifterID is implicit in the backing vector, as the index.
    /// The order of the lifters is arbitrary.
    lifters: Vec<Lifter>,

    /// The MeetID is implicit in the backing vector, as the index.
    /// The order of the meets is arbitrary.
    meets: Vec<Meet>,

    /// The EntryID is implicit in the backing vector, as the index.
    /// The order of the entries is by increasing lifter_id.
    /// Within the entries of a single lifter_id, the order is arbitrary.
    entries: Vec<Entry>,
}

/// A list of indices into the OplDB's vector of Entries.
///
/// Accessing the Entry vector through indices allows effective
/// creation of sublists. Union and Intersection operations allow
/// for simple and extremely efficient construction of complex views.
///
/// The indices are in the same sort order as the Vec<Entry> list,
/// by lifter_id.
pub struct EntryFilter<'a> {
    /// The index list should not outlive the database itself.
    opldb: &'a OplDb,

    /// A list of indices into the OplDb's Entry vector,
    /// sorted and curated in some specific order.
    // TODO: Make non-pub.
    pub indices: Vec<u32>,
}

/// Reads the `lifters.csv` file into a Vec<Lifter>.
fn import_lifters_csv(file: &str) -> Result<Vec<Lifter>, Box<Error>> {
    let mut vec = Vec::with_capacity(140_000);

    let mut rdr = csv::Reader::from_path(file)?;
    for lifter in rdr.deserialize() {
        let lifter: Lifter = lifter?;
        vec.push(lifter);
    }

    vec.shrink_to_fit();
    Ok(vec)
}

/// Reads the `meet.csv` file into a Vec<Meet>.
fn import_meets_csv(file: &str) -> Result<Vec<Meet>, Box<Error>> {
    let mut vec = Vec::with_capacity(10_000);

    let mut rdr = csv::Reader::from_path(file)?;
    for meet in rdr.deserialize() {
        let meet: Meet = meet?;
        vec.push(meet);
    }

    vec.shrink_to_fit();
    Ok(vec)
}

/// Reads the `entries.csv` file into a Vec<Entry>.
fn import_entries_csv(file: &str) -> Result<Vec<Entry>, Box<Error>> {
    let mut vec = Vec::with_capacity(450_000);

    let mut rdr = csv::Reader::from_path(file)?;
    for entry in rdr.deserialize() {
        let entry: Entry = entry?;
        vec.push(entry);
    }

    // The entries database is sorted by lifter_id.
    // This invariant allows for extremely efficient lifter-uniqueness
    // filtering without constructing additional data structures.
    vec.sort_unstable_by_key(|e| e.lifter_id);

    vec.shrink_to_fit();
    Ok(vec)
}

impl OplDb {
    pub fn from_csv(lifters_csv: &str, meets_csv: &str, entries_csv: &str)
        -> Result<OplDb, Box<Error>>
    {
        let lifters = import_lifters_csv(lifters_csv)?;
        let meets = import_meets_csv(meets_csv)?;
        let entries = import_entries_csv(entries_csv)?;
        Ok(OplDb { lifters, meets, entries })
    }

    pub fn size_bytes(&self) -> usize {
        let lifters_size = mem::size_of::<Lifter>() * self.lifters.len();
        let meets_size = mem::size_of::<Meet>() * self.meets.len();
        let entries_size = mem::size_of::<Entry>() * self.entries.len();
        let struct_size = mem::size_of::<OplDb>();
        lifters_size + meets_size + entries_size + struct_size
    }

    pub fn get_lifter(&self, n: u32) -> &Lifter {
        &self.lifters[n as usize]
    }

    pub fn get_meet(&self, n: u32) -> &Meet {
        &self.meets[n as usize]
    }

    pub fn get_entry(&self, n: u32) -> &Entry {
        &self.entries[n as usize]
    }

    pub fn filter_entries<F>(&self, filter: F) -> EntryFilter
        where F: Fn(&Entry) -> bool
    {
        let mut vec = Vec::new();
        for i in 0 .. self.entries.len() {
            if filter(&self.entries[i]) {
                vec.push(i as u32);
            }
        }
        vec.shrink_to_fit();
        EntryFilter { opldb: &self, indices: vec }
    }
}
