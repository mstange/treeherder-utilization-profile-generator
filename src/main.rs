use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::Path,
    time::SystemTime,
};

use fxprof_processed_profile::{
    Category, CategoryColor, CategoryHandle, CpuDelta, FrameFlags, FrameHandle, Profile,
    ReferenceTimestamp, SamplingInterval, SourceLocation, StringHandle, SubcategoryHandle,
    ThreadHandle, Timestamp,
};
use rustc_hash::FxHashMap;
use serde_derive::*;

#[derive(Debug, Deserialize, Eq, PartialEq)]
struct Row<'a> {
    repository_name: &'a str,
    job_type_name: &'a str,
    platform: &'a str,
    job_group_symbol: &'a str,
    duration: u64,
}

fn main() {
    let path = std::env::args().nth(1).expect("Missing CSV path");
    let file = File::open(&path).unwrap();
    let reader = BufReader::new(file);
    let mut reader = csv::Reader::from_reader(reader);
    let mut raw_record = csv::StringRecord::new();
    let headers = reader.headers().unwrap().clone();

    let mut converter = Converter::new(&path);
    let mut line_number = 2;

    while reader.read_record(&mut raw_record).unwrap() {
        let row: Row = match raw_record.deserialize(Some(&headers)) {
            Ok(row) => row,
            Err(e) => panic!("Error parsing line {line_number}: {e}"),
        };
        converter.process_row(&row, line_number);
        line_number += 1;
    }

    let filename = Path::new(&path).file_name().unwrap().to_string_lossy();
    let out_filename = format!("{filename}-profile.json");
    let out_path = Path::new(&path).with_file_name(out_filename);
    let writer = File::create(out_path).unwrap();
    let writer = BufWriter::new(writer);

    serde_json::to_writer(writer, &converter.profile).unwrap();
}

struct Converter {
    profile: Profile,
    thread: ThreadHandle,
    repo_category: CategoryHandle,
    #[allow(dead_code)]
    chrome_category: CategoryHandle,
    #[allow(dead_code)]
    fenix_category: CategoryHandle,
    #[allow(dead_code)]
    gva_category: CategoryHandle,
    #[allow(dead_code)]
    refbrow_category: CategoryHandle,
    tp6_category: CategoryHandle,
    tp6m_category: SubcategoryHandle,
    startup_category: CategoryHandle,
    resource_category: CategoryHandle,
    speedometer_category: CategoryHandle,
    file_path_s: StringHandle,
    category_by_job_name: FxHashMap<StringHandle, SubcategoryHandle>,
    accumulated_seconds: u64,
}

impl Converter {
    pub fn new(path: &str) -> Self {
        let mut profile = Profile::new(
            "Utilization",
            ReferenceTimestamp::from_system_time(SystemTime::now()),
            SamplingInterval::from_millis(1000), // one sample = 1 second
        );
        let process = profile.add_process(
            "Utilization",
            0,
            Timestamp::from_millis_since_reference(0.0),
        );
        let thread = profile.add_thread(
            process,
            0,
            Timestamp::from_millis_since_reference(0.0),
            true,
        );
        let repo_category = CategoryHandle::OTHER; //profile.handle_for_category(Category("Repository", CategoryColor::Green));
        // let job_category = profile.handle_for_category(Category("Job", CategoryColor::Blue));
        let tp6_category = profile.handle_for_category(Category("Tp6", CategoryColor::Yellow));
        let tp6m_category = profile.handle_for_subcategory(tp6_category, "tp6m");
        let chrome_category =
            profile.handle_for_category(Category("Chrome", CategoryColor::Yellow));
        let startup_category =
            profile.handle_for_category(Category("Startup", CategoryColor::Blue));
        let resource_category =
            profile.handle_for_category(Category("Resource", CategoryColor::Purple));
        let speedometer_category =
            profile.handle_for_category(Category("Speedometer", CategoryColor::Red));
        let fenix_category = profile.handle_for_category(Category("Fenix", CategoryColor::Green));
        let gva_category =
            profile.handle_for_category(Category("GeckoView example", CategoryColor::Blue));
        let refbrow_category =
            profile.handle_for_category(Category("Reference browser", CategoryColor::Blue));
        let file_path_s = profile.handle_for_string(path);

        let accumulated_seconds = 0;
        let category_by_job_name = FxHashMap::default();
        Self {
            profile,
            thread,
            repo_category,
            chrome_category,
            fenix_category,
            gva_category,
            refbrow_category,
            tp6_category,
            tp6m_category,
            startup_category,
            resource_category,
            speedometer_category,
            file_path_s,
            category_by_job_name,
            accumulated_seconds,
        }
    }

    fn label_frame(
        profile: &mut Profile,
        thread: ThreadHandle,
        category: CategoryHandle,
        label: &str,
    ) -> FrameHandle {
        let label_handle = profile.handle_for_string(label);
        profile.handle_for_frame_with_label(thread, label_handle, category, FrameFlags::empty())
    }

    pub fn process_row(&mut self, row: &Row, line_number: u32) {
        let thread = self.thread;
        let profile = &mut self.profile;
        let Row {
            repository_name,
            job_type_name,
            platform,
            job_group_symbol,
            duration,
        } = row;
        let repo_frame = Self::label_frame(profile, thread, self.repo_category, repository_name);
        let platform_frame = Self::label_frame(profile, thread, CategoryHandle::OTHER, platform);
        let job_group_frame =
            Self::label_frame(profile, thread, CategoryHandle::OTHER, job_group_symbol);
        let job_name_s = profile.handle_for_string(job_type_name);
        let category = *self
            .category_by_job_name
            .entry(job_name_s)
            .or_insert_with(|| {
                // if job_type_name.contains("-chrome") {
                //     self.chrome_category
                // } else if job_type_name.contains("-fenix") {
                //     self.fenix_category
                // } else if job_type_name.contains("-refbrow") {
                //     self.refbrow_category
                // } else if job_type_name.contains("-geckoview") {
                //     self.gva_category
                // } else {
                //     CategoryHandle::OTHER
                // }
                if job_type_name.contains("-tp6m") {
                    self.tp6m_category
                } else if job_type_name.contains("-tp6") {
                    self.tp6_category.into()
                } else if job_type_name.contains("-speedometer") {
                    self.speedometer_category.into()
                } else if job_type_name.contains("-startup") {
                    self.startup_category.into()
                } else if job_type_name.contains("-resource") {
                    self.resource_category.into()
                } else {
                    CategoryHandle::OTHER.into()
                }
            });
        let job_frame = profile.handle_for_frame_with_label_and_source_location(
            thread,
            job_name_s,
            SourceLocation {
                file_path: Some(self.file_path_s),
                line: Some(line_number),
                col: None,
            },
            category,
            FrameFlags::empty(),
        );
        let mut stack = [repo_frame, platform_frame, job_group_frame, job_frame].into_iter();
        let stack = profile.handle_for_stack_frames(thread, move |_| stack.next());
        let start = self.accumulated_seconds;
        self.accumulated_seconds += *duration;
        let end = self.accumulated_seconds;
        profile.add_sample(
            thread,
            Timestamp::from_millis_since_reference(start as f64 * 1000.0),
            stack,
            CpuDelta::ZERO,
            i32::try_from(*duration).expect("duration in seconds overflowed i32"),
        );
        profile.add_sample(
            thread,
            Timestamp::from_millis_since_reference(end as f64 * 1000.0),
            stack,
            CpuDelta::from_millis(*duration as f64 * 1000.0),
            0,
        );
    }
}
