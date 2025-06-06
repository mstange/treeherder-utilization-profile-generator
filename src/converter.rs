use std::time::SystemTime;

use fxprof_processed_profile::{
    CategoryHandle, CpuDelta, FrameFlags, FrameHandle, Profile, ReferenceTimestamp,
    SamplingInterval, SourceLocation, StringHandle, SubcategoryHandle, ThreadHandle, Timestamp,
};
use rustc_hash::FxHashMap;
use smallvec::SmallVec;

use super::category_matcher::{CategoryMatcher, CategoryOrSubcategory};
use crate::Row;

pub struct Converter {
    profile: Profile,
    thread: ThreadHandle,
    category_matcher: CategoryMatcher,
    file_path_s: StringHandle,
    category_by_job_name: FxHashMap<StringHandle, SubcategoryHandle>,
    accumulated_seconds: u64,
}

impl Converter {
    pub fn new(
        path: &str,
        category_matchers: &[(&'static [&'static str], CategoryOrSubcategory)],
    ) -> Self {
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
        let category_matcher = CategoryMatcher::new(&mut profile, category_matchers);

        let file_path_s = profile.handle_for_string(path);

        let accumulated_seconds = 0;
        let category_by_job_name = FxHashMap::default();
        Self {
            profile,
            thread,
            category_matcher,
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
        let repo_frame = Self::label_frame(profile, thread, CategoryHandle::OTHER, repository_name);
        let platform_frame = Self::label_frame(profile, thread, CategoryHandle::OTHER, platform);
        let job_group_frame =
            Self::label_frame(profile, thread, CategoryHandle::OTHER, job_group_symbol);
        let job_name_s = profile.handle_for_string(job_type_name);
        let category = *self
            .category_by_job_name
            .entry(job_name_s)
            .or_insert_with(|| {
                let job_name_fragments: SmallVec<[&str; 10]> =
                    job_type_name.split(&['-', '/']).collect();
                self.category_matcher.get(&job_name_fragments)
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

    pub fn finish(self) -> Profile {
        self.profile
    }
}
